use chrono::{DateTime, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use sysinfo::{Pid, System};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Job {
    pub id: String,
    pub command: String,
    pub status: String,
    pub pid: Option<u32>,
    pub pgid: Option<u32>,
    pub start_time: DateTime<Utc>,
    pub error_preview: Option<String>,
}

fn generate_job_id() -> String {
    let random_part: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    format!("job-{}", random_part.to_lowercase())
}

fn get_jobs_file_path(base_path: &Path) -> PathBuf {
    base_path.join(".ferri").join("jobs.json")
}

fn read_jobs(base_path: &Path) -> io::Result<Vec<Job>> {
    let jobs_file = get_jobs_file_path(base_path);
    if !jobs_file.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&jobs_file)?;
    if content.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(&content).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))
}

fn write_jobs(base_path: &Path, jobs: &[Job]) -> io::Result<()> {
    let jobs_file = get_jobs_file_path(base_path);
    let content =
        serde_json::to_string_pretty(jobs).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
    fs::write(jobs_file, content)
}

pub fn submit_job(
    base_path: &Path,
    command: &mut Command,
    secrets: HashMap<String, String>,
    original_command: &[String],
) -> io::Result<Job> {
    let job_id = generate_job_id();
    let ferri_dir = base_path.join(".ferri");
    let jobs_dir = ferri_dir.join("jobs");
    let job_dir = jobs_dir.join(&job_id);
    fs::create_dir_all(&job_dir)?;

    let stdout_path = job_dir.join("stdout.log");
    let stderr_path = job_dir.join("stderr.log");
    let exit_code_path = job_dir.join("exit_code.log");

    // Manually construct the command string, quoting arguments
    let program = command.get_program().to_string_lossy();
    let args_vec: Vec<String> = command
        .get_args()
        .map(|s| s.to_string_lossy().to_string())
        .collect();
    let quoted_args: Vec<String> = args_vec
        .iter()
        .map(|arg| shell_words::quote(arg).to_string())
        .collect();
    let executable_command = format!("{} {}", program, quoted_args.join(" "));

    // Create a wrapper script to execute the command and capture its exit code
    let wrapper_script = format!(
        "{} > {} 2> /dev/null; echo $? > {}",
        executable_command,
        stdout_path.to_string_lossy(),
        exit_code_path.to_string_lossy()
    );


    let mut shell_command = Command::new("sh");
    shell_command.stdin(Stdio::null());
    shell_command.arg("-c").arg(wrapper_script);
    for (key, value) in secrets {
        shell_command.env(key, value);
    }

    // The new process needs to be in its own process group to be killable
    unsafe {
        shell_command.pre_exec(|| {
            nix::unistd::setpgid(nix::unistd::Pid::from_raw(0), nix::unistd::Pid::from_raw(0))?;
            Ok(())
        });
    }

    let child = shell_command.spawn()?;
    let pid = Some(child.id());
    let pgid = pid; // In a new process group, pgid is the same as pid

    let new_job = Job {
        id: job_id.clone(),
        command: original_command.join(" "),
        status: "Running".to_string(),
        pid,
        pgid,
        start_time: Utc::now(),
        error_preview: None,
    };

    let mut jobs = read_jobs(base_path)?;
    jobs.push(new_job.clone());
    write_jobs(base_path, &jobs)?;

    Ok(new_job)
}

pub fn list_jobs(base_path: &Path) -> io::Result<Vec<Job>> {
    let mut jobs = read_jobs(base_path)?;
    let mut needs_write = false;
    let mut s = System::new_all();
    s.refresh_processes();

    for job in jobs.iter_mut() {
        if job.status == "Running" {
            if let Some(pid) = job.pid {
                if s.process(Pid::from(pid as usize)).is_none() {
                    // Process is gone, determine status from exit code
                    let exit_code_path = base_path
                        .join(".ferri/jobs")
                        .join(&job.id)
                        .join("exit_code.log");

                    let exit_code_content = fs::read_to_string(exit_code_path).unwrap_or_default();
                    let exit_code = exit_code_content.trim().parse::<i32>().unwrap_or(1); // Default to failure

                    if exit_code == 0 {
                        job.status = "Completed".to_string();
                    } else {
                        job.status = "Failed".to_string();
                        let stderr_path = base_path
                            .join(".ferri/jobs")
                            .join(&job.id)
                            .join("stderr.log");
                        let stderr_content = fs::read_to_string(stderr_path).unwrap_or_default();
                        if !stderr_content.is_empty() {
                            let preview: String = stderr_content.chars().take(200).collect();
                            job.error_preview = Some(format!("Exit Code: {}. {}", exit_code, preview));
                        } else {
                            job.error_preview = Some(format!("Job failed with exit code: {}", exit_code));
                        }
                    }
                    needs_write = true;
                }
            } else {
                // Job was created without a PID, mark as failed.
                job.status = "Failed".to_string();
                job.error_preview = Some("Job failed to spawn.".to_string());
                needs_write = true;
            }
        }
    }

    if needs_write {
        write_jobs(base_path, &jobs)?;
    }

    Ok(jobs)
}

pub fn get_job_output(base_path: &Path, job_id: &str) -> io::Result<String> {
    let jobs = read_jobs(base_path)?;
    let job = jobs.iter().find(|j| j.id == job_id).ok_or_else(|| {
        io::Error::new(ErrorKind::NotFound, format!("Job '{}' not found.", job_id))
    })?;

    let job_dir = base_path.join(".ferri/jobs").join(&job.id);
    let stdout_path = job_dir.join("stdout.log");
    let stderr_path = job_dir.join("stderr.log");

    let stdout_content = fs::read_to_string(stdout_path).unwrap_or_default();
    let stderr_content = fs::read_to_string(stderr_path).unwrap_or_default();

    if !stderr_content.trim().is_empty() {
        Ok(format!("Error Log:\n---\n{}", stderr_content))
    } else if !stdout_content.trim().is_empty() {
        Ok(stdout_content)
    } else {
        Ok("Job produced no output.".to_string())
    }
}

pub fn kill_job(base_path: &Path, job_id: &str) -> io::Result<()> {
    let mut jobs = read_jobs(base_path)?;
    let job_index = jobs.iter().position(|j| j.id == job_id).ok_or_else(|| {
        io::Error::new(ErrorKind::NotFound, format!("Job '{}' not found.", job_id))
    })?;

    let job = &mut jobs[job_index];

    if job.status != "Running" {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("Job '{}' is not running.", job_id),
        ));
    }

    if let Some(pgid) = job.pgid {
        let pgid_to_kill = nix::unistd::Pid::from_raw(pgid as i32);
        match nix::sys::signal::killpg(pgid_to_kill, nix::sys::signal::Signal::SIGTERM) {
            Ok(_) => {
                job.status = "Terminated".to_string();
                write_jobs(base_path, &jobs)?;
                Ok(())
            }
            Err(e) => Err(io::Error::new(
                ErrorKind::Other,
                format!("Failed to kill process group {}: {}", pgid, e),
            )),
        }
    } else {
        Err(io::Error::new(
            ErrorKind::Other,
            "Job does not have a process group ID.",
        ))
    }
}
