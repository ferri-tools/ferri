use ferri_automation::jobs::JobInstance;
use std::io;

pub fn run(_jobs: Vec<JobInstance>) -> io::Result<()> {
    // This function is no longer used, but we're keeping it to avoid breaking changes.
    // It will be removed in a future version.
    Ok(())
}
