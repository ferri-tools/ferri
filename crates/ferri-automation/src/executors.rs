use std::collections::HashMap;
use std::io;
use std::path::Path;
use crate::flow::{Job, Update, StepUpdate, StepStatus};
use std::process::{Command, Stdio};
use std::thread::{self, JoinHandle};
use crossbeam_channel::Sender;

pub trait Executor {
    fn execute(
        &self,
        job_id: &str,
        job: &Job,
        base_path: &Path,
        secrets: &HashMap<String, String>,
        update_sender: Sender<Update>,
    ) -> io::Result<ExecutionHandle>;
}

#[derive(Debug)]
pub struct ExecutionHandle(pub JoinHandle<io::Result<()>>);

pub struct ExecutorRegistry {
    executors: HashMap<String, Box<dyn Executor + Send + Sync>>,
}

impl ExecutorRegistry {
    pub fn new() -> Self {
        let mut executors: HashMap<String, Box<dyn Executor + Send + Sync>> =HashMap::new();
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
        update_sender: Sender<Update>,
    ) -> io::Result<ExecutionHandle> {
        let job_id = job_id.to_string();
        let job = job.clone();
        let base_path = base_path.to_path_buf();
        let handle = thread::spawn(move || {
            for (step_idx, step) in job.steps.iter().enumerate() {
                let step_name = step.name.clone().unwrap_or_else(|| format!("step-{}", step_idx));

                update_sender.send(Update::Step(StepUpdate {
                    job_id: job_id.clone(),
                    step_name: step_name.clone(),
                    status: StepStatus::Running,
                    output: None,
                })).unwrap();

                if let Some(run_command) = &step.run {
                    let mut cmd = Command::new("sh");
                    cmd.arg("-c").arg(run_command);
                    cmd.current_dir(&base_path);
                    cmd.stdout(Stdio::piped());
                    cmd.stderr(Stdio::piped());

                    let mut child = cmd.spawn().unwrap();
                    let stdout = child.stdout.take().unwrap();
                    let stderr = child.stderr.take().unwrap();

                    let stdout_handle = thread::spawn(move || {
                        use std::io::BufRead;
                        let reader = std::io::BufReader::new(stdout);
                        reader.lines().for_each(|line| {
                            println!("{}", line.unwrap());
                        });
                    });

                    let stderr_handle = thread::spawn(move || {
                        use std::io::BufRead;
                        let reader = std::io::BufReader::new(stderr);
                        reader.lines().for_each(|line| {
                            eprintln!("{}", line.unwrap());
                        });
                    });

                    stdout_handle.join().unwrap();
                    stderr_handle.join().unwrap();

                    let status = child.wait().unwrap();

                    if status.success() {
                        update_sender.send(Update::Step(StepUpdate {
                            job_id: job_id.clone(),
                            step_name: step_name.clone(),
                            status: StepStatus::Completed,
                            output: None,
                        })).unwrap();
                    } else {
                        let err_msg = format!("Step {} failed", step_name);
                        update_sender.send(Update::Step(StepUpdate {
                            job_id: job_id.clone(),
                            step_name: step_name.clone(),
                            status: StepStatus::Failed(err_msg.clone()),
                            output: None,
                        })).unwrap();
                        return Err(io::Error::new(io::ErrorKind::Other, err_msg));
                    }
                }
            }
            Ok(())
        });

        Ok(ExecutionHandle(handle))
    }
}