use ferri_automation::flow::parse_flow_file;
use ferri_automation::orchestrator::FlowOrchestrator;
use std::collections::HashMap;
use std::fs;
use std::io;
use tempfile::tempdir;

/// Execute a flow and collect the final result
fn execute_flow(flow_yaml: &str) -> Result<ExecutionSummary, String> {
    let dir = tempdir().map_err(|e| e.to_string())?;
    let base_path = dir.path();
    let flow_file = base_path.join("test-flow.yml");

    fs::write(&flow_file, flow_yaml).map_err(|e| e.to_string())?;
    let flow_doc = parse_flow_file(&flow_file).map_err(|e| e.to_string())?;

    // The new orchestrator does not require a channel for this kind of test
    let orchestrator = FlowOrchestrator::new(
        flow_doc,
        base_path,
        HashMap::new(),
    );

    // Execute the flow directly and synchronously
    let execution_result = orchestrator.execute();

    let mut summary = ExecutionSummary::new();
    summary.execution_result = Some(execution_result);
    Ok(summary)
}

use std::path::PathBuf;

/// Simplified summary of flow execution for testing
#[derive(Debug)]
struct ExecutionSummary {
    execution_result: Option<Result<PathBuf, io::Error>>,
}

impl ExecutionSummary {
    fn new() -> Self {
        Self {
            execution_result: None,
        }
    }

    fn succeeded(&self) -> bool {
        matches!(self.execution_result, Some(Ok(_)))
    }

    fn failed(&self) -> bool {
        matches!(self.execution_result, Some(Err(_)))
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
      runs_on: ubuntu-latest
      steps:
        - name: say-hello
          run: echo "Hello from Ferri!"
"#;
    let summary = execute_flow(flow_yaml).expect("Flow should execute");
    assert!(summary.succeeded(), "Flow should succeed");
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
      runs_on: ubuntu-latest
      steps:
        - name: build-step
          run: echo "Building..."
    test:
      runs_on: ubuntu-latest
      needs: [build]
      steps:
        - name: test-step
          run: echo "Testing..."
"#;
    let summary = execute_flow(flow_yaml).expect("Flow should execute");
    assert!(summary.succeeded(), "Flow should succeed");
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
      runs_on: ubuntu-latest
      steps:
        - name: failing-step
          run: exit 1
"#;
    let summary = execute_flow(flow_yaml).expect("Flow should execute");
    assert!(summary.failed(), "Flow should fail");
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
      runs_on: ubuntu-latest
      steps:
        - name: fail-build
          run: exit 1
    test:
      runs_on: ubuntu-latest
      needs: [build]
      steps:
        - name: should-not-run
          run: echo "This should not execute"
"#;
    let summary = execute_flow(flow_yaml).expect("Flow should execute");
    assert!(summary.failed(), "Flow should fail");
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
      runs_on: ubuntu-latest
      steps:
        - name: step1
          run: echo "Step 1"
        - name: step2
          run: echo "Step 2"
"#;
    let summary = execute_flow(flow_yaml).expect("Flow should execute");
    assert!(summary.succeeded(), "Flow should succeed");
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
      runs_on: ubuntu-latest
      steps:
        - name: step1
          run: echo "Step 1 runs"
        - name: step2
          run: exit 1
        - name: step3
          run: echo "Step 3 should not run"
"#;
    let summary = execute_flow(flow_yaml).expect("Flow should execute");
    assert!(summary.failed(), "Flow should fail");
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
      runs_on: ubuntu-latest
      steps:
        - name: no-output
          run: "true"
"#;
    let summary = execute_flow(flow_yaml).expect("Flow should execute");
    assert!(summary.succeeded(), "Flow should succeed");
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
      runs_on: ubuntu-latest
      steps:
        - name: sleep-and-print
          run: sleep 1 && echo "Done sleeping"
"#;
    let summary = execute_flow(flow_yaml).expect("Flow should execute");
    assert!(summary.succeeded(), "Flow should succeed");
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
      runs_on: ubuntu-latest
      steps:
        - name: multiline-script
          run: |
            echo "Line 1"
            echo "Line 2"
"#;
    let summary = execute_flow(flow_yaml).expect("Flow should execute");
    assert!(summary.succeeded(), "Flow should succeed");
}
