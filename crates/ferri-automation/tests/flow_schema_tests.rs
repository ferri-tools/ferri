use ferri_automation::flow::parse_flow_file;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_parse_valid_flow() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: test-flow
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      steps:
        - name: Checkout
          run: echo "Checking out code"
        - name: Build
          run: echo "Building project"
"#;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test-flow.yml");
    fs::write(&file_path, flow_yaml).unwrap();

    let result = parse_flow_file(&file_path);
    assert!(result.is_ok(), "Should parse valid flow");

    let flow = result.unwrap();
    assert_eq!(flow.api_version, "ferri.flow/v1alpha1");
    assert_eq!(flow.kind, "Flow");
    assert_eq!(flow.metadata.name, "test-flow");
    assert_eq!(flow.spec.jobs.len(), 1);
}

#[test]
fn test_parse_flow_with_dependencies() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: multi-job-flow
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      steps:
        - run: echo "Building"
    test:
      runs-on: ubuntu-latest
      needs: [build]
      steps:
        - run: echo "Testing"
    deploy:
      runs-on: ubuntu-latest
      needs: [test]
      steps:
        - run: echo "Deploying"
"#;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("multi-job.yml");
    fs::write(&file_path, flow_yaml).unwrap();

    let result = parse_flow_file(&file_path);
    assert!(result.is_ok(), "Should parse flow with dependencies");

    let flow = result.unwrap();
    assert_eq!(flow.spec.jobs.len(), 3);

    let test_job = flow.spec.jobs.get("test").unwrap();
    assert_eq!(test_job.needs.as_ref().unwrap(), &vec!["build"]);

    let deploy_job = flow.spec.jobs.get("deploy").unwrap();
    assert_eq!(deploy_job.needs.as_ref().unwrap(), &vec!["test"]);
}

#[test]
fn test_parse_flow_with_inputs() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: parameterized-flow
spec:
  inputs:
    environment:
      type: string
      description: "Deployment environment"
      default: "staging"
    parallel:
      type: boolean
      default: false
  jobs:
    deploy:
      runs-on: ubuntu-latest
      steps:
        - run: echo "Deploying to ${{ ctx.inputs.environment }}"
"#;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("param-flow.yml");
    fs::write(&file_path, flow_yaml).unwrap();

    let result = parse_flow_file(&file_path);
    assert!(result.is_ok(), "Should parse flow with inputs");

    let flow = result.unwrap();
    let inputs = flow.spec.inputs.as_ref().unwrap();
    assert_eq!(inputs.len(), 2);
    assert!(inputs.contains_key("environment"));
    assert!(inputs.contains_key("parallel"));
}

#[test]
fn test_parse_flow_with_workspaces() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: workspace-flow
spec:
  workspaces:
    - name: source
    - name: artifacts
  jobs:
    build:
      runs-on: ubuntu-latest
      steps:
        - name: Build
          run: echo "Building"
          workspaces:
            - name: source
              mountPath: /workspace/source
              readOnly: true
            - name: artifacts
              mountPath: /workspace/artifacts
"#;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("workspace-flow.yml");
    fs::write(&file_path, flow_yaml).unwrap();

    let result = parse_flow_file(&file_path);
    assert!(result.is_ok(), "Should parse flow with workspaces");

    let flow = result.unwrap();
    let workspaces = flow.spec.workspaces.as_ref().unwrap();
    assert_eq!(workspaces.len(), 2);

    let build_job = flow.spec.jobs.get("build").unwrap();
    let step_workspaces = build_job.steps[0].workspaces.as_ref().unwrap();
    assert_eq!(step_workspaces.len(), 2);
    assert_eq!(step_workspaces[0].mount_path, "/workspace/source");
    assert!(step_workspaces[0].read_only);
}

// --- Validation Tests ---

#[test]
fn test_invalid_kind() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: InvalidKind
metadata:
  name: test
spec:
  jobs:
    test:
      runs-on: ubuntu-latest
      steps:
        - run: echo "test"
"#;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("invalid-kind.yml");
    fs::write(&file_path, flow_yaml).unwrap();

    let result = parse_flow_file(&file_path);
    assert!(result.is_err(), "Should reject invalid kind");
}

#[test]
fn test_empty_jobs() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: empty-flow
spec:
  jobs: {}
"#;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("empty-jobs.yml");
    fs::write(&file_path, flow_yaml).unwrap();

    let result = parse_flow_file(&file_path);
    assert!(result.is_err(), "Should reject flow with no jobs");
}

#[test]
fn test_job_with_no_steps() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: no-steps-flow
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      steps: []
"#;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("no-steps.yml");
    fs::write(&file_path, flow_yaml).unwrap();

    let result = parse_flow_file(&file_path);
    assert!(result.is_err(), "Should reject job with no steps");
}

#[test]
fn test_step_with_both_run_and_uses() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: invalid-step-flow
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      steps:
        - run: echo "test"
          uses: actions/checkout@v4
"#;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("both-run-uses.yml");
    fs::write(&file_path, flow_yaml).unwrap();

    let result = parse_flow_file(&file_path);
    assert!(result.is_err(), "Should reject step with both 'run' and 'uses'");
}

#[test]
fn test_step_with_neither_run_nor_uses() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: invalid-step-flow
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      steps:
        - name: Invalid Step
"#;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("neither-run-uses.yml");
    fs::write(&file_path, flow_yaml).unwrap();

    let result = parse_flow_file(&file_path);
    assert!(result.is_err(), "Should reject step without 'run' or 'uses'");
}

#[test]
fn test_nonexistent_dependency() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: bad-deps-flow
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      needs: [nonexistent]
      steps:
        - run: echo "test"
"#;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("bad-deps.yml");
    fs::write(&file_path, flow_yaml).unwrap();

    let result = parse_flow_file(&file_path);
    assert!(result.is_err(), "Should reject job depending on non-existent job");
}

#[test]
fn test_self_dependency() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: self-dep-flow
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      needs: [build]
      steps:
        - run: echo "test"
"#;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("self-dep.yml");
    fs::write(&file_path, flow_yaml).unwrap();

    let result = parse_flow_file(&file_path);
    assert!(result.is_err(), "Should reject job depending on itself");
}

#[test]
fn test_nonexistent_workspace_reference() {
    let flow_yaml = r#"
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: bad-workspace-flow
spec:
  workspaces:
    - name: source
  jobs:
    build:
      runs-on: ubuntu-latest
      steps:
        - run: echo "test"
          workspaces:
            - name: nonexistent
              mountPath: /workspace
"#;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("bad-workspace.yml");
    fs::write(&file_path, flow_yaml).unwrap();

    let result = parse_flow_file(&file_path);
    assert!(result.is_err(), "Should reject reference to non-existent workspace");
}
