use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
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

fn read_jobs(base_path: &Path) -> std::io::Result<Vec<Job>> {
    let jobs_file = get_jobs_file_path(base_path);
    if !jobs_file.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(jobs_file)?;
    serde_json::from_str(&content).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))
}

fn write_jobs(base_path: &Path, jobs: &[Job]) -> std::io::Result<()> {
    let jobs_file = get_jobs_file_path(base_path);
    let content =
        serde_json::to_string_pretty(jobs).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;
    fs::write(jobs_file, content)
}

use std::collections::HashMap;
use std::os::unix::process::CommandExt;

pub fn submit_job(
    base_path: &Path,
    mut command: Command,
    secrets: HashMap<String, String>,
) -> std::io::Result<Job> {
    let job_id = generate_job_id();
    let ferri_dir = base_path.join(".ferri");
    let jobs_dir = ferri_dir.join("jobs");
    let job_dir = jobs_dir.join(&job_id);
    fs::create_dir_all(&job_dir)?;

    let stdout_path = job_dir.join("stdout.log");
    let stderr_path = job_dir.join("stderr.log");

    let stdout_file = fs::File::create(stdout_path)?;
    let stderr_file = fs::File::create(stderr_path)?;

    // Construct the command string for logging before we move `command`
    let command_string = format!(
        "{} {}",
        command.get_program().to_string_lossy(),
        command
            .get_args()
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );

    command.envs(secrets);
    command.stdout(Stdio::from(stdout_file));
    command.stderr(Stdio::from(stderr_file));

    unsafe {
        command.pre_exec(|| {
            nix::unistd::setpgid(nix::unistd::Pid::from_raw(0), nix::unistd::Pid::from_raw(0))
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            Ok(())
        });
    }

    let child = command.spawn()?;
    let pid = Some(child.id());
    let pgid = pid; // In a new process group, pgid is the same as pid

    let new_job = Job {
        id: job_id.clone(),
        command: command_string,
        status: "Running".to_string(),
        pid,
        pgid,
    };

    let mut jobs = read_jobs(base_path)?;
    jobs.push(new_job.clone());
    write_jobs(base_path, &jobs)?;

    Ok(new_job)
}

pub fn list_jobs(base_path: &Path) -> std::io::Result<Vec<Job>> {
    let mut jobs = read_jobs(base_path)?;
    let mut needs_write = false;
    let mut s = System::new_all();
    s.refresh_processes();

    for job in jobs.iter_mut() {
        if job.status == "Running" {
            if let Some(pid) = job.pid {
                if s.process(Pid::from(pid as usize)).is_none() {
                    let stderr_path = base_path
                        .join(".ferri/jobs")
                        .join(&job.id)
                        .join("stderr.log");
                    let stderr_content = fs::read_to_string(stderr_path).unwrap_or_default();

                    if stderr_content.trim().is_empty() {
                        job.status = "Completed".to_string();
                    } else {
                        job.status = "Failed".to_string();
                    }
                    needs_write = true;
                }
            } else {
                job.status = "Failed".to_string();
                needs_write = true;
            }
        }
    }

    if needs_write {
        write_jobs(base_path, &jobs)?;
    }

    Ok(jobs)
}

pub fn get_job_output(base_path: &Path, job_id: &str) -> std::io::Result<String> {
    let jobs = read_jobs(base_path)?;
    let job = jobs.iter().find(|j| j.id == job_id).ok_or_else(|| {
        std::io::Error::new(ErrorKind::NotFound, format!("Job '{}' not found.", job_id))
    })?;

    let stdout_path = base_path
        .join(".ferri/jobs")
        .join(&job.id)
        .join("stdout.log");

    if !stdout_path.exists() {
        return Err(std::io::Error::new(
            ErrorKind::NotFound,
            "Job output not found.",
        ));
    }

    fs::read_to_string(stdout_path)
}

pub fn kill_job(base_path: &Path, job_id: &str) -> std::io::Result<()> {
    let mut jobs = read_jobs(base_path)?;
    let job_index = jobs.iter().position(|j| j.id == job_id).ok_or_else(|| {
        std::io::Error::new(ErrorKind::NotFound, format!("Job '{}' not found.", job_id))
    })?;

    let job = &mut jobs[job_index];

    if job.status != "Running" {
        return Err(std::io::Error::new(
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
            Err(e) => Err(std::io::Error::new(
                ErrorKind::Other,
                format!("Failed to kill process group {}: {}", pgid, e),
            )),
        }
    } else {
        Err(std::io::Error::new(
            ErrorKind::Other,
            "Job does not have a process group ID.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_submit_job_creates_job_and_files() {
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        let command = vec!["echo".to_string(), "hello".to_string()];

        fs::create_dir_all(base_path.join(".ferri")).unwrap();

        let job = submit_job(base_path, &command).unwrap();

        assert!(job.id.starts_with("job-"));
        assert_eq!(job.command, "echo hello");
        assert_eq!(job.status, "Running");
        assert!(job.pid.is_some());

        let jobs_file = get_jobs_file_path(base_path);
        assert!(jobs_file.exists());

        let jobs = read_jobs(base_path).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, job.id);

        let job_dir = base_path.join(".ferri/jobs").join(&job.id);
        assert!(job_dir.exists());
        assert!(job_dir.join("stdout.log").exists());
        assert!(job_dir.join("stderr.log").exists());
    }

    #[test]
    fn test_list_jobs_updates_status() {
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        let command = vec!["sleep".to_string(), "0.1".to_string()];

        fs::create_dir_all(base_path.join(".ferri")).unwrap();

        let job = submit_job(base_path, &command).unwrap();
        assert_eq!(job.status, "Running");

        let jobs_running = list_jobs(base_path).unwrap();
        assert_eq!(jobs_running.len(), 1);
        assert_eq!(jobs_running[0].status, "Running");

        std::thread::sleep(std::time::Duration::from_millis(200));

        let jobs_completed = list_jobs(base_path).unwrap();
        assert_eq!(jobs_completed.len(), 1);
        assert_eq!(jobs_completed[0].status, "Completed");
    }

    #[test]
    fn test_get_job_output() {
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        let command = vec!["echo".to_string(), "hello job".to_string()];

        fs::create_dir_all(base_path.join(".ferri")).unwrap();

        // Submit a job and get its ID.
        let job = submit_job(base_path, &command).unwrap();

        // Wait for the job to complete.
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Yank the output with the correct ID.
        let output = get_job_output(base_path, &job.id).unwrap();
        assert_eq!(output.trim(), "hello job");

        // Try to yank with an invalid ID.
        let result = get_job_output(base_path, "job-invalid");
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().kind(), ErrorKind::NotFound);
    }
}