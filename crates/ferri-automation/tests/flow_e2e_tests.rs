use ferri_automation::flow::{parse_flow_file, Update, JobStatus, StepStatus};
use ferri_automation::orchestrator::FlowOrchestrator;
use std::collections::HashMap;
use std::fs;
use std::io;
use tempfile::tempdir;

/// Execute a flow and collect all updates
fn execute_flow(flow_yaml: &str) -> Result<ExecutionSummary, String> {
    let dir = tempdir().map_err(|e| e.to_string())?;
    let base_path = dir.path();
    let flow_file = base_path.join("test-flow.yml");

    // Write the flow file
    fs::write(&flow_file, flow_yaml).map_err(|e| e.to_string())?;

    // Parse the flow
    let flow_doc = parse_flow_file(&flow_file).map_err(|e| e.to_string())?;

    // Create channel for receiving updates
    let (tx, rx) = crossbeam_channel::unbounded();

    // Create orchestrator
    let orchestrator = FlowOrchestrator::new(
        flow_doc,
        &base_path.to_path_buf(),
        tx,
        HashMap::new(),
    );

    // Spawn execution thread
    let execution_handle = std::thread::spawn(move || orchestrator.execute());

    // Collect all updates
    let mut summary = ExecutionSummary::new();

    for update in rx {
        match update {
            Update::Job(job_update) => {
                summary.job_updates.push(job_update.clone());
                summary.final_job_status.insert(
                    job_update.job_id.clone(),
                    job_update.status.clone()
                );
            }
            Update::Step(step_update) => {
                summary.step_updates.push(step_update.clone());
                let key = (step_update.job_id.clone(), step_update.step_name.clone());
                summary.final_step_status.insert(key, step_update.status.clone());

                // Store step output if available
                if let Some(output) = step_update.output {
                    let output_key = (step_update.job_id.clone(), step_update.step_name.clone());
                    summary.step_outputs.insert(output_key, output);
                }
            }
        }
    }

    // Wait for execution to complete
    match execution_handle.join() {
        Ok(result) => {
            summary.execution_result = Some(result);
            Ok(summary)
        }
        Err(_) => Err("Execution thread panicked".to_string())
    }
}

/// Summary of flow execution for testing
#[derive(Debug)]
struct ExecutionSummary {
    job_updates: Vec<ferri_automation::flow::JobUpdate>,
    step_updates: Vec<ferri_automation::flow::StepUpdate>,
    final_job_status: HashMap<String, JobStatus>,
    final_step_status: HashMap<(String, String), StepStatus>,
    step_outputs: HashMap<(String, String), String>,
    execution_result: Option<Result<(), io::Error>>,
}

impl ExecutionSummary {
    fn new() -> Self {
        Self {
            job_updates: Vec::new(),
            step_updates: Vec::new(),
            final_job_status: HashMap::new(),
            final_step_status: HashMap::new(),
            step_outputs: HashMap::new(),
            execution_result: None,
        }
    }

    fn job_succeeded(&self, job_id: &str) -> bool {
        matches!(
            self.final_job_status.get(job_id),
            Some(JobStatus::Succeeded)
        )
    }

    fn job_failed(&self, job_id: &str) -> bool {
        matches!(
            self.final_job_status.get(job_id),
            Some(JobStatus::Failed(_))
        )
    }

    fn step_completed(&self, job_id: &str, step_name: &str) -> bool {
        matches!(
            self.final_step_status.get(&(job_id.to_string(), step_name.to_string())),
            Some(StepStatus::Completed)
        )
    }

    fn step_failed(&self, job_id: &str, step_name: &str) -> bool {
        matches!(
            self.final_step_status.get(&(job_id.to_string(), step_name.to_string())),
            Some(StepStatus::Failed(_))
        )
    }

    fn get_step_output(&self, job_id: &str, step_name: &str) -> Option<&String> {
        self.step_outputs.get(&(job_id.to_string(), step_name.to_string()))
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_simple_single_job_execution() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: simple-test
spec:
  jobs:
    hello:
      runs-on: ubuntu-latest
      steps:
        - name: say-hello
          run: echo "Hello from Ferri!"
        - name: print-date
          run: date
"#;

    let summary = execute_flow(flow_yaml).expect("Flow should execute successfully");

    assert!(summary.job_succeeded("hello"), "Job 'hello' should succeed");
    assert!(summary.step_completed("hello", "say-hello"), "Step 'say-hello' should complete");
    assert!(summary.step_completed("hello", "print-date"), "Step 'print-date' should complete");
    assert!(summary.execution_result.unwrap().is_ok(), "Overall execution should succeed");
}

#[test]
fn test_multi_job_with_dependencies() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: multi-job-test
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      steps:
        - name: build-step
          run: echo "Building..."

    test:
      runs-on: ubuntu-latest
      needs: [build]
      steps:
        - name: test-step
          run: echo "Testing..."

    deploy:
      runs-on: ubuntu-latest
      needs: [test]
      steps:
        - name: deploy-step
          run: echo "Deploying..."
"#;

    let summary = execute_flow(flow_yaml).expect("Flow should execute successfully");

    assert!(summary.job_succeeded("build"), "Job 'build' should succeed");
    assert!(summary.job_succeeded("test"), "Job 'test' should succeed");
    assert!(summary.job_succeeded("deploy"), "Job 'deploy' should succeed");
    assert!(summary.execution_result.unwrap().is_ok(), "Overall execution should succeed");
}

#[test]
fn test_job_failure_propagation() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: failure-test
spec:
  jobs:
    will-fail:
      runs-on: ubuntu-latest
      steps:
        - name: failing-step
          run: exit 1
"#;

    let summary = execute_flow(flow_yaml).expect("Flow execution should complete");

    assert!(summary.job_failed("will-fail"), "Job 'will-fail' should fail");
    assert!(summary.step_failed("will-fail", "failing-step"), "Step 'failing-step' should fail");
    assert!(summary.execution_result.unwrap().is_err(), "Overall execution should fail");
}

#[test]
fn test_dependency_failure_skips_downstream() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: dependency-failure-test
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      steps:
        - name: fail-build
          run: exit 1

    test:
      runs-on: ubuntu-latest
      needs: [build]
      steps:
        - name: should-not-run
          run: echo "This should not execute"
"#;

    let summary = execute_flow(flow_yaml).expect("Flow execution should complete");

    assert!(summary.job_failed("build"), "Job 'build' should fail");
    // The 'test' job should either not appear in final_job_status or be marked as skipped/failed
    // Depending on orchestrator implementation, it may not run at all
    assert!(!summary.job_succeeded("test"), "Job 'test' should not succeed");
}

#[test]
fn test_multiple_steps_in_job() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: multi-step-test
spec:
  jobs:
    process:
      runs-on: ubuntu-latest
      steps:
        - name: step1
          run: echo "Step 1"
        - name: step2
          run: echo "Step 2"
        - name: step3
          run: echo "Step 3"
        - name: step4
          run: echo "Step 4"
        - name: step5
          run: echo "Step 5"
"#;

    let summary = execute_flow(flow_yaml).expect("Flow should execute successfully");

    assert!(summary.job_succeeded("process"), "Job 'process' should succeed");
    assert!(summary.step_completed("process", "step1"), "Step 1 should complete");
    assert!(summary.step_completed("process", "step2"), "Step 2 should complete");
    assert!(summary.step_completed("process", "step3"), "Step 3 should complete");
    assert!(summary.step_completed("process", "step4"), "Step 4 should complete");
    assert!(summary.step_completed("process", "step5"), "Step 5 should complete");
}

#[test]
fn test_step_failure_stops_job() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: step-failure-test
spec:
  jobs:
    process:
      runs-on: ubuntu-latest
      steps:
        - name: step1
          run: echo "Step 1 runs"
        - name: step2
          run: exit 1
        - name: step3
          run: echo "Step 3 should not run"
"#;

    let summary = execute_flow(flow_yaml).expect("Flow execution should complete");

    assert!(summary.job_failed("process"), "Job 'process' should fail");
    assert!(summary.step_completed("process", "step1"), "Step 1 should complete");
    assert!(summary.step_failed("process", "step2"), "Step 2 should fail");
    // Step 3 should not run after step 2 fails
    assert!(!summary.step_completed("process", "step3"), "Step 3 should not complete");
}

#[test]
fn test_empty_command_output() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: empty-output-test
spec:
  jobs:
    silent:
      runs-on: ubuntu-latest
      steps:
        - name: no-output
          run: "true"
"#;

    let summary = execute_flow(flow_yaml).expect("Flow should execute successfully");

    assert!(summary.job_succeeded("silent"), "Job 'silent' should succeed");
    assert!(summary.step_completed("silent", "no-output"), "Step 'no-output' should complete");
}

#[test]
fn test_long_running_command() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: long-running-test
spec:
  jobs:
    slow:
      runs-on: ubuntu-latest
      steps:
        - name: sleep-and-print
          run: sleep 1 && echo "Done sleeping"
"#;

    let summary = execute_flow(flow_yaml).expect("Flow should execute successfully");

    assert!(summary.job_succeeded("slow"), "Job 'slow' should succeed");
    assert!(summary.step_completed("slow", "sleep-and-print"), "Step should complete after sleep");
}

#[test]
fn test_multiline_shell_script() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: multiline-test
spec:
  jobs:
    script:
      runs-on: ubuntu-latest
      steps:
        - name: multiline-script
          run: |
            echo "Line 1"
            echo "Line 2"
            echo "Line 3"
"#;

    let summary = execute_flow(flow_yaml).expect("Flow should execute successfully");

    assert!(summary.job_succeeded("script"), "Job 'script' should succeed");
    assert!(summary.step_completed("script", "multiline-script"), "Multiline script should complete");
}
