//! Core logic for managing asynchronous background jobs.

use chrono::{DateTime, Utc};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use sysinfo::{System};
use crate::execute::PreparedCommand;

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

/// Monitors a spawned child process in a background thread.
fn monitor_job(
    base_path: PathBuf,
    job_id: String,
    mut child: std::process::Child,
    output_file: Option<PathBuf>,
) {
    thread::spawn(move || {
        eprintln!("DEBUG: Monitor thread started for job {}", job_id);
        let result = (|| -> io::Result<()> {
            let job_dir = base_path.join(".ferri/jobs").join(&job_id);
            let stdout_path = job_dir.join("stdout.log");
            let stderr_path = job_dir.join("stderr.log");
            let exit_code_path = job_dir.join("exit_code.log");

            eprintln!("DEBUG: Waiting for child process to complete");
            let status = child.wait()?;  // Just wait, don't try to read pipes
            eprintln!("DEBUG: Child completed with status: {:?}", status);

            let exit_code = status.code().unwrap_or(1);

            // Read the output from the files that were written during execution
            let stdout_content = fs::read(&stdout_path).unwrap_or_default();
            let stderr_content = fs::read(&stderr_path).unwrap_or_default();

            // Copy to user-specified output file if provided
            if let Some(path) = output_file {
                let _ = fs::write(path, &stdout_content);
            }

            eprintln!("DEBUG: Writing exit_code.log to {:?}", exit_code_path);
            fs::write(&exit_code_path, exit_code.to_string())?;
            eprintln!("DEBUG: Exit code written successfully");

            // Update job status in jobs.json
            let mut jobs = read_jobs(&base_path)?;
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                job.status = if exit_code == 0 {
                    "Completed".to_string()
                } else {
                    "Failed".to_string()
                };

                // Add error preview for failed jobs
                if exit_code != 0 {
                    let error_content = String::from_utf8_lossy(&stderr_content).to_string();
                    let preview: String = error_content.chars().take(200).collect();
                    job.error_preview = Some(format!("Exit Code: {}. {}", exit_code, preview));
                }

                write_jobs(&base_path, &jobs)?;
            }
            Ok(())
        })();

        // If there's an error, log it to a debug file
        if let Err(e) = result {
            eprintln!("DEBUG: Monitor thread error: {}", e);
            let error_log = base_path.join(".ferri/jobs").join(&job_id).join("thread_error.log");
            let _ = fs::write(error_log, format!("Monitor thread error: {}", e));
        }
    });
}

/// Spawns a local command with stdout/stderr redirected to files
fn spawn_local_command(
    command: &mut Command,
    secrets: HashMap<String, String>,
    input_data: Option<Vec<u8>>,
    stdout_path: &Path,
    stderr_path: &Path,
) -> io::Result<std::process::Child> {
    eprintln!("DEBUG: spawn_local_command called");
    command.envs(secrets);

    // Redirect stdout/stderr directly to files (no pipes = no threading issues)
    let stdout_file = fs::File::create(stdout_path)?;
    let stderr_file = fs::File::create(stderr_path)?;
    command.stdout(Stdio::from(stdout_file));
    command.stderr(Stdio::from(stderr_file));

    if input_data.is_some() {
        command.stdin(Stdio::piped());
    } else {
        command.stdin(Stdio::null());
    }

    // Only use pre_exec on Linux - it can hang on macOS when called from threads
    #[cfg(target_os = "linux")]
    unsafe {
        command.pre_exec(|| {
            nix::unistd::setpgid(nix::unistd::Pid::from_raw(0), nix::unistd::Pid::from_raw(0))?;
            Ok(())
        });
    }

    eprintln!("DEBUG: About to spawn command");
    let mut child = command.spawn()?;
    eprintln!("DEBUG: Command spawned, pid: {}", child.id());

    // Write stdin if provided
    if let Some(data) = input_data {
        if let Some(mut stdin) = child.stdin.take() {
            thread::spawn(move || {
                let _ = stdin.write_all(&data);
            });
        }
    }

    Ok(child)
}

fn execute_remote_command(
    request: reqwest::blocking::RequestBuilder,
) -> io::Result<Vec<u8>> {
    let response = request.send().map_err(|e| io::Error::new(ErrorKind::Other, e))?;
    let status = response.status();
    let body = response.text().map_err(|e| io::Error::new(ErrorKind::Other, e))?;

    if !status.is_success() {
        return Err(io::Error::new(
            ErrorKind::Other,
            format!("API Error ({}): {}", status, body),
        ));
    }

    let mut text_content = String::new();
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        let response_chunks = if let Some(array) = json.as_array() { array.to_vec() } else { vec![json] };
        for chunk in response_chunks {
            if let Some(candidates) = chunk.get("candidates").and_then(|c| c.as_array()) {
                for candidate in candidates {
                    if let Some(parts) = candidate.get("content").and_then(|c| c.get("parts")).and_then(|p| p.as_array()) {
                        for part in parts {
                            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                text_content.push_str(text);
                            }
                        }
                    }
                }
            }
        }
    } else {
        text_content = body;
    }

    Ok(text_content.into_bytes())
}


pub fn submit_job(
    base_path: &Path,
    prepared_command: PreparedCommand,
    secrets: HashMap<String, String>,
    original_command: &[String],
    input_data: Option<Vec<u8>>,
    output_file: Option<PathBuf>,
) -> io::Result<Job> {
    let job_id = generate_job_id();
    let job_dir = base_path.join(".ferri/jobs").join(&job_id);
    fs::create_dir_all(&job_dir)?;

    let stdout_path = job_dir.join("stdout.log");
    let stderr_path = job_dir.join("stderr.log");

    eprintln!("DEBUG: submit_job - spawning command on main thread");

    // Spawn the command on the MAIN thread (critical for macOS compatibility)
    let child = match prepared_command {
        PreparedCommand::Local(mut command, stdin_data) => {
            let input_bytes = stdin_data.map(|s| s.into_bytes());
            spawn_local_command(&mut command, secrets, input_bytes, &stdout_path, &stderr_path)?
        }
        PreparedCommand::Remote(_request) => {
            return Err(io::Error::new(
                ErrorKind::Other,
                "Remote commands not yet supported in background jobs"
            ));
        }
    };

    let pid = child.id();
    eprintln!("DEBUG: Process spawned with PID: {}", pid);

    let new_job = Job {
        id: job_id.clone(),
        command: original_command.join(" "),
        status: "Running".to_string(),
        pid: Some(pid),
        pgid: None,
        start_time: Utc::now(),
        error_preview: None,
    };

    let mut jobs = read_jobs(base_path).unwrap_or_else(|_| Vec::new());
    jobs.push(new_job.clone());
    write_jobs(base_path, &jobs)?;

    // Now move the waiting/monitoring to a background thread
    monitor_job(base_path.to_path_buf(), job_id, child, output_file);

    Ok(new_job)
}


pub fn list_jobs(base_path: &Path) -> io::Result<Vec<Job>> {
    let mut jobs = read_jobs(base_path)?;
    let mut needs_write = false;
    let _s = System::new_all();

    for job in jobs.iter_mut() {
        if job.status == "Running" {
            let exit_code_path = base_path.join(".ferri/jobs").join(&job.id).join("exit_code.log");
            if exit_code_path.exists() {
                let exit_code_content = fs::read_to_string(exit_code_path).unwrap_or_default();
                let exit_code = exit_code_content.trim().parse::<i32>().unwrap_or(1);

                if exit_code == 0 {
                    job.status = "Completed".to_string();
                } else {
                    job.status = "Failed".to_string();
                    let stdout_path = base_path.join(".ferri/jobs").join(&job.id).join("stdout.log");
                    let error_content = fs::read_to_string(stdout_path).unwrap_or_default();
                    let preview: String = error_content.chars().take(200).collect();
                    job.error_preview = Some(format!("Exit Code: {}. {}", exit_code, preview));
                }
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
    let _job = jobs.iter().find(|j| j.id == job_id).ok_or_else(|| {
        io::Error::new(ErrorKind::NotFound, format!("Job '{}' not found.", job_id))
    })?;

    let job_dir = base_path.join(".ferri/jobs").join(job_id);
    let stdout_path = job_dir.join("stdout.log");

    fs::read_to_string(stdout_path).or_else(|_| Ok("Job produced no output or is still running.".to_string()))
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
            "Cannot kill a remote job.",
        ))
    }
}