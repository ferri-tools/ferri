use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Job {
    pub id: String,
    pub command: String,
    pub status: String,
    pub pid: Option<u32>,
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

pub fn submit_job(base_path: &Path, command_args: &[String]) -> std::io::Result<Job> {
    if command_args.is_empty() {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "Command cannot be empty.",
        ));
    }

    let job_id = generate_job_id();
    let ferri_dir = base_path.join(".ferri");
    let jobs_dir = ferri_dir.join("jobs");
    let job_dir = jobs_dir.join(&job_id);
    fs::create_dir_all(&job_dir)?;

    let stdout_path = job_dir.join("stdout.log");
    let stderr_path = job_dir.join("stderr.log");

    let stdout_file = fs::File::create(stdout_path)?;
    let stderr_file = fs::File::create(stderr_path)?;

    let mut command = Command::new(&command_args[0]);
    command.args(&command_args[1..]);

    // Detach the process from the parent.
    // On Unix, this is the default behavior. For cross-platform compatibility,
    // one might use libraries, but for now, standard library behavior is sufficient.
    command.stdout(Stdio::from(stdout_file));
    command.stderr(Stdio::from(stderr_file));

    let child = command.spawn()?;
    let pid = Some(child.id());

    let new_job = Job {
        id: job_id.clone(),
        command: command_args.join(" "),
        status: "Running".to_string(),
        pid,
    };

    let mut jobs = read_jobs(base_path)?;
    jobs.push(new_job.clone());
    write_jobs(base_path, &jobs)?;

    Ok(new_job)
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

        // Initialize the .ferri directory
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
}
