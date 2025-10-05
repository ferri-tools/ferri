# L2 Flow Schema Manual Test Plan

**Feature:** New ferri-flow.yml schema implementation (Issue #15)
**Layer:** L2 - Workflow Automation
**Status:** Ready for testing
**Prerequisites:** ferri built from `feature/flow-schema-reimplementation` branch

---

## Overview

This test plan validates the new ferri-flow.yml schema specification, including:
- Schema parsing and validation
- Expression evaluation (`${{ }}` syntax)
- Job dependency resolution
- Input parameters
- Workspace definitions
- Parallel job execution

---

## Setup

```bash
# 1. Ensure you're on the feature branch
git checkout feature/flow-schema-reimplementation

# 2. Build and install ferri
cargo install --path ferri-cli --force

# 3. Create a test directory
mkdir -p /tmp/ferri-flow-test
cd /tmp/ferri-flow-test

# 4. Initialize ferri
ferri init
```

---

## Test Suite 1: Schema Validation

### Test 1.1: Valid Flow - Basic Structure

**Objective:** Verify that a minimal valid flow parses successfully.

```bash
# Create minimal-flow.yml
cat > minimal-flow.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: minimal-test
spec:
  jobs:
    test-job:
      runs-on: ubuntu-latest
      steps:
        - run: echo "Hello from minimal flow"
EOF

# Run show command
ferri flow show minimal-flow.yml
```

**Expected Result:**
- ✅ Command succeeds
- ✅ Displays: "Flow: minimal-test"
- ✅ Displays: "API Version: ferri.flow/v1alpha1"
- ✅ Shows "test-job" with 1 step

---

### Test 1.2: Invalid Kind

**Objective:** Verify validation rejects invalid `kind` field.

```bash
cat > invalid-kind.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Workflow
metadata:
  name: bad-kind
spec:
  jobs:
    test:
      runs-on: ubuntu-latest
      steps:
        - run: echo "test"
EOF

ferri flow show invalid-kind.yml
```

**Expected Result:**
- ❌ Command fails
- ✅ Error message mentions "Invalid kind 'Workflow', expected 'Flow'"

---

### Test 1.3: Empty Jobs

**Objective:** Verify validation rejects flows with no jobs.

```bash
cat > empty-jobs.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: empty
spec:
  jobs: {}
EOF

ferri flow show empty-jobs.yml
```

**Expected Result:**
- ❌ Command fails
- ✅ Error message mentions "Flow must define at least one job"

---

### Test 1.4: Job with No Steps

**Objective:** Verify validation rejects jobs without steps.

```bash
cat > no-steps.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: no-steps
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      steps: []
EOF

ferri flow show no-steps.yml
```

**Expected Result:**
- ❌ Command fails
- ✅ Error message mentions "Job 'build' must have at least one step"

---

### Test 1.5: Step with Both run and uses

**Objective:** Verify validation rejects steps with both `run` and `uses`.

```bash
cat > both-run-uses.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: invalid-step
spec:
  jobs:
    test:
      runs-on: ubuntu-latest
      steps:
        - run: echo "test"
          uses: actions/checkout@v4
EOF

ferri flow show both-run-uses.yml
```

**Expected Result:**
- ❌ Command fails
- ✅ Error message mentions "cannot have both 'run' and 'uses'"

---

### Test 1.6: Step with Neither run nor uses

**Objective:** Verify validation rejects steps without `run` or `uses`.

```bash
cat > neither-run-uses.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: invalid-step
spec:
  jobs:
    test:
      runs-on: ubuntu-latest
      steps:
        - name: Invalid Step
EOF

ferri flow show neither-run-uses.yml
```

**Expected Result:**
- ❌ Command fails
- ✅ Error message mentions "must have either 'run' or 'uses'"

---

## Test Suite 2: Job Dependencies

### Test 2.1: Simple Linear Dependencies

**Objective:** Verify dependency ordering for linear workflow.

```bash
cat > linear-flow.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: linear-test
spec:
  jobs:
    first:
      runs-on: ubuntu-latest
      steps:
        - run: echo "First job"

    second:
      runs-on: ubuntu-latest
      needs: [first]
      steps:
        - run: echo "Second job"

    third:
      runs-on: ubuntu-latest
      needs: [second]
      steps:
        - run: echo "Third job"
EOF

ferri flow show linear-flow.yml
```

**Expected Result:**
- ✅ Command succeeds
- ✅ Shows "second" depends on "first"
- ✅ Shows "third" depends on "second"

---

### Test 2.2: Parallel Jobs

**Objective:** Verify display of jobs that can run in parallel.

```bash
cat > parallel-flow.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: parallel-test
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      steps:
        - run: echo "Building..."

    test:
      runs-on: ubuntu-latest
      steps:
        - run: echo "Testing..."

    deploy:
      runs-on: ubuntu-latest
      needs: [build, test]
      steps:
        - run: echo "Deploying..."
EOF

ferri flow show parallel-flow.yml
```

**Expected Result:**
- ✅ Command succeeds
- ✅ "build" and "test" have no dependencies (can run in parallel)
- ✅ "deploy" depends on both "build, test"

---

### Test 2.3: Non-existent Dependency

**Objective:** Verify validation catches references to non-existent jobs.

```bash
cat > bad-dep.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: bad-dependency
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      needs: [nonexistent]
      steps:
        - run: echo "test"
EOF

ferri flow show bad-dep.yml
```

**Expected Result:**
- ❌ Command fails
- ✅ Error message mentions "depends on non-existent job 'nonexistent'"

---

### Test 2.4: Self Dependency

**Objective:** Verify validation rejects jobs that depend on themselves.

```bash
cat > self-dep.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: self-dependency
spec:
  jobs:
    build:
      runs-on: ubuntu-latest
      needs: [build]
      steps:
        - run: echo "test"
EOF

ferri flow show self-dep.yml
```

**Expected Result:**
- ❌ Command fails
- ✅ Error message mentions "cannot depend on itself"

---

## Test Suite 3: Input Parameters

### Test 3.1: Flow with Inputs

**Objective:** Verify flows with input parameters parse correctly.

```bash
cat > input-flow.yml <<'EOF'
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
    timeout:
      type: number
      default: 300
  jobs:
    deploy:
      runs-on: ubuntu-latest
      steps:
        - run: echo "Deploying to ${{ ctx.inputs.environment }}"
EOF

ferri flow show input-flow.yml
```

**Expected Result:**
- ✅ Command succeeds
- ✅ Flow parses without errors
- ✅ No validation errors for expression syntax

---

### Test 3.2: Expression Syntax in Steps

**Objective:** Verify expression syntax is recognized in step definitions.

```bash
cat > expression-flow.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: expression-test
spec:
  inputs:
    message:
      type: string
      default: "Hello World"
  jobs:
    greet:
      runs-on: ubuntu-latest
      steps:
        - name: Print greeting
          run: echo "${{ ctx.inputs.message }}"

        - name: Use in env
          run: echo "Message is $MSG"
          env:
            MSG: "${{ ctx.inputs.message }}"
EOF

ferri flow show expression-flow.yml
```

**Expected Result:**
- ✅ Command succeeds
- ✅ No parsing errors for expression syntax

---

## Test Suite 4: Workspaces

### Test 4.1: Flow with Workspaces

**Objective:** Verify workspace definitions are parsed correctly.

```bash
cat > workspace-flow.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: workspace-test
spec:
  workspaces:
    - name: source-code
    - name: build-artifacts
  jobs:
    build:
      runs-on: ubuntu-latest
      steps:
        - name: Compile
          run: echo "Building..."
          workspaces:
            - name: source-code
              mountPath: /workspace/src
              readOnly: true
            - name: build-artifacts
              mountPath: /workspace/dist
EOF

ferri flow show workspace-flow.yml
```

**Expected Result:**
- ✅ Command succeeds
- ✅ No validation errors

---

### Test 4.2: Invalid Workspace Reference

**Objective:** Verify validation catches references to undefined workspaces.

```bash
cat > bad-workspace.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: bad-workspace
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
EOF

ferri flow show bad-workspace.yml
```

**Expected Result:**
- ❌ Command fails
- ✅ Error message mentions "non-existent workspace 'nonexistent'"

---

## Test Suite 5: Complete Example Flow

### Test 5.1: Example Pipeline

**Objective:** Verify the included example flow works correctly.

```bash
# Use the example from the repo
ferri flow show ../../examples/example-flow.yml
```

**Expected Result:**
- ✅ Command succeeds
- ✅ Shows flow name: "example-pipeline"
- ✅ Shows 4 jobs: checkout, build, test, deploy
- ✅ Displays correct dependencies:
  - checkout: no dependencies
  - build: depends on checkout
  - test: depends on checkout
  - deploy: depends on build, test

---

### Test 5.2: Complex Multi-Job Flow

**Objective:** Test a realistic CI/CD pipeline.

```bash
cat > ci-pipeline.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: ci-pipeline
  labels:
    team: platform
    env: dev
spec:
  inputs:
    branch:
      type: string
      default: "main"
  jobs:
    checkout:
      name: "Checkout Code"
      runs-on: ubuntu-latest
      steps:
        - run: echo "Cloning branch ${{ ctx.inputs.branch }}"

    lint:
      name: "Lint Code"
      runs-on: ubuntu-latest
      needs: [checkout]
      steps:
        - run: echo "Running linter..."
        - run: echo "Linting complete"

    test:
      name: "Run Tests"
      runs-on: ubuntu-latest
      needs: [checkout]
      steps:
        - run: echo "Running unit tests..."
        - run: echo "Running integration tests..."

    build:
      name: "Build Application"
      runs-on: ubuntu-latest
      needs: [lint, test]
      steps:
        - run: echo "Compiling application..."
        - run: echo "Creating artifacts..."

    security-scan:
      name: "Security Scan"
      runs-on: ubuntu-latest
      needs: [build]
      steps:
        - run: echo "Running security scan..."

    deploy:
      name: "Deploy to Staging"
      runs-on: ubuntu-latest
      needs: [security-scan]
      steps:
        - run: echo "Deploying to staging environment..."
EOF

ferri flow show ci-pipeline.yml
```

**Expected Result:**
- ✅ Command succeeds
- ✅ Shows 6 jobs with correct dependency tree
- ✅ Identifies parallel execution opportunities:
  - lint and test can run in parallel (both depend only on checkout)
  - build waits for both lint and test

---

## Test Suite 6: Backward Compatibility

### Test 6.1: Legacy Format Still Works

**Objective:** Verify old pipeline format still works.

```bash
cat > legacy-pipeline.yml <<'EOF'
name: "Legacy Pipeline"
steps:
  - name: "step1"
    command: "echo Hello"
  - name: "step2"
    command: "echo World"
EOF

ferri flow show legacy-pipeline.yml
```

**Expected Result:**
- ✅ Command succeeds (falls back to legacy parser)
- ✅ No errors

---

## Test Suite 7: Error Messages

### Test 7.1: YAML Syntax Error

**Objective:** Verify clear error messages for malformed YAML.

```bash
cat > bad-yaml.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: bad-yaml
  # Missing colon below
  invalid syntax here
EOF

ferri flow show bad-yaml.yml
```

**Expected Result:**
- ❌ Command fails
- ✅ Error message mentions "YAML parse error"

---

### Test 7.2: Missing Required Field

**Objective:** Verify error for missing required fields.

```bash
cat > missing-field.yml <<'EOF'
apiVersion: ferri.flow/v1alpha1
kind: Flow
spec:
  jobs:
    test:
      runs-on: ubuntu-latest
      steps:
        - run: echo "test"
EOF

ferri flow show missing-field.yml
```

**Expected Result:**
- ❌ Command fails
- ✅ Error indicates missing `metadata` or `metadata.name`

---

## Summary Checklist

After completing all tests, verify:

- [ ] All validation tests pass/fail as expected
- [ ] Dependency resolution works correctly
- [ ] Expression syntax is recognized (no parse errors)
- [ ] Workspace definitions parse correctly
- [ ] Example flow validates successfully
- [ ] Legacy format still works (backward compatibility)
- [ ] Error messages are clear and helpful

---

## Known Limitations (Not Bugs)

The following features are intentionally not yet implemented:

- **Execution:** Flows validate but don't execute yet (TUI integration pending)
- **Reusable Actions:** `uses:` field parses but doesn't execute
- **Workspace Mounting:** Definitions parse but mounting not implemented
- **Retry Strategies:** Schema supports but runtime doesn't execute
- **ferri-runtime:** `set-output` command not yet available

These are tracked as future enhancements in #15.

---

## Reporting Issues

If any test fails unexpectedly, report with:
1. Test number and name
2. Actual vs expected result
3. Full error message
4. Flow YAML content

File issues at: https://github.com/ferri-tools/ferri/issues/15
