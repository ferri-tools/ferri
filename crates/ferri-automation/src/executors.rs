use crate::flow::{Job, OutputUpdate, StepStatus, StepUpdate, Update};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

pub trait Executor {
    fn execute(
        &self,
        job_id: &str,
        job: &Job,
        base_path: &Path,
        secrets: &HashMap<String, String>,
        writer: Arc<Mutex<io::BufWriter<fs::File>>>,
    ) -> io::Result<ExecutionHandle>;
}

#[derive(Debug)]
pub struct ExecutionHandle(pub thread::JoinHandle<io::Result<()>>);

pub struct ExecutorRegistry {
    executors: HashMap<String, Box<dyn Executor + Send + Sync>>,
}

impl Default for ExecutorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutorRegistry {
    pub fn new() -> Self {
        let mut executors: HashMap<String, Box<dyn Executor + Send + Sync>> = HashMap::new();
        executors.insert("process".to_string(), Box::new(ProcessExecutor));
        Self { executors }
    }

    pub fn get(&self, name: &str) -> Option<&(dyn Executor + Send + Sync)> {
        self.executors.get(name).map(|e| e.as_ref())
    }
}

struct ProcessExecutor;

impl Executor for ProcessExecutor {
    fn execute(
        &self,
        job_id: &str,
        job: &Job,
        base_path: &Path,
        _secrets: &HashMap<String, String>,
        writer: Arc<Mutex<io::BufWriter<fs::File>>>,
    ) -> io::Result<ExecutionHandle> {
        let job_id = job_id.to_string();
        let job = job.clone();
        let base_path = base_path.to_path_buf();

        let handle = thread::spawn(move || {
            for (step_index, step) in job.steps.iter().enumerate() {
                if let Some(run_command) = &step.run {
                    // Log Step Running
                    let step_update = Update::Step(StepUpdate {
                        job_id: job_id.clone(),
                        step_index,
                        status: StepStatus::Running,
                    });
                    let mut writer_lock = writer.lock().unwrap();
                    writeln!(
                        writer_lock,
                        "{}",
                        serde_json::to_string(&step_update).unwrap()
                    )?;
                    writer_lock.flush()?;
                    drop(writer_lock);

                    let mut cmd = Command::new("sh");
                    cmd.arg("-c")
                        .arg(run_command)
                        .current_dir(&base_path)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());

                    let mut child = cmd.spawn()?;

                    let stdout = child.stdout.take().unwrap();
                    let stderr = child.stderr.take().unwrap();

                    let stdout_job_id = job_id.clone();
                    let stdout_writer = Arc::clone(&writer);
                    let stdout_handle = thread::spawn(move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines() {
                            let update = Update::Output(OutputUpdate {
                                job_id: stdout_job_id.clone(),
                                step_index,
                                line: line.unwrap_or_default(),
                            });
                            let mut writer_lock = stdout_writer.lock().unwrap();
                            writeln!(writer_lock, "{}", serde_json::to_string(&update).unwrap())
                                .unwrap();
                            writer_lock.flush().unwrap();
                        }
                    });

                    let stderr_job_id = job_id.clone();
                    let stderr_writer = Arc::clone(&writer);
                    let stderr_handle = thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines() {
                            let update = Update::Output(OutputUpdate {
                                job_id: stderr_job_id.clone(),
                                step_index,
                                line: line.unwrap_or_default(),
                            });
                            let mut writer_lock = stderr_writer.lock().unwrap();
                            writeln!(writer_lock, "{}", serde_json::to_string(&update).unwrap())
                                .unwrap();
                            writer_lock.flush().unwrap();
                        }
                    });

                    stdout_handle.join().unwrap();
                    stderr_handle.join().unwrap();

                    let status = child.wait()?;

                    let final_status = if status.success() {
                        StepStatus::Completed
                    } else {
                        let err_msg = format!(
                            "Step failed with exit code {}",
                            status.code().unwrap_or(1)
                        );
                        StepStatus::Failed(err_msg)
                    };

                    let step_update = Update::Step(StepUpdate {
                        job_id: job_id.clone(),
                        step_index,
                        status: final_status.clone(),
                    });
                    let mut writer_lock = writer.lock().unwrap();
                    writeln!(
                        writer_lock,
                        "{}",
                        serde_json::to_string(&step_update).unwrap()
                    )?;
                    writer_lock.flush()?;

                    if !status.success() {
                        return Err(io::Error::other(
                            format!("Step {} failed", step_index),
                        ));
                    }
                }
            }
            Ok(())
        });

        Ok(ExecutionHandle(handle))
    }
}