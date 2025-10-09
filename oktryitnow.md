# Test Plan: E2E Integration Tests for L2 Engine (#35)

## Prerequisites

These tests run automatically as part of the test suite. No manual setup required.

## Running the Tests

From the project root (or claude-worktree):

```bash
# Run only the E2E integration tests
cargo test --test flow_e2e_tests

# Run with output
cargo test --test flow_e2e_tests -- --nocapture

# Run a specific test
cargo test --test flow_e2e_tests test_simple_single_job_execution

# Run all tests including unit tests
cargo test
```

## Test Coverage

The new integration test suite covers:

1. **Simple single-job execution** - Basic job with multiple steps
2. **Multi-job with dependencies** - Sequential jobs using `needs`
3. **Job failure propagation** - Jobs fail when steps exit with non-zero
4. **Dependency failure behavior** - Failed jobs skip downstream dependents
5. **Multiple steps in job** - Five sequential steps execute in order
6. **Step failure stops job** - Failing step prevents subsequent steps
7. **Empty command output** - Commands with no output succeed
8. **Long-running commands** - Sleep commands complete successfully
9. **Multiline shell scripts** - Multiline YAML `run:` blocks work correctly

## Expected Results

When you run the tests, you should see:

```
running 9 tests
test test_dependency_failure_skips_downstream ... ok
test test_empty_command_output ... ok
test test_job_failure_propagation ... ok
test test_long_running_command ... ok
test test_multi_job_with_dependencies ... ok
test test_multiline_shell_script ... ok
test test_multiple_steps_in_job ... ok
test test_simple_single_job_execution ... ok
test test_step_failure_stops_job ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured
```

## Adding New Tests

To add new integration tests:

1. Open `crates/ferri-automation/tests/flow_e2e_tests.rs`
2. Create a new test function with `#[test]` attribute
3. Define your flow YAML inline as a string
4. Call `execute_flow(flow_yaml)` to run it
5. Use the `ExecutionSummary` helper methods to make assertions:
   - `summary.job_succeeded("job_id")`
   - `summary.job_failed("job_id")`
   - `summary.step_completed("job_id", "step_name")`
   - `summary.step_failed("job_id", "step_name")`
   - `summary.get_step_output("job_id", "step_name")`

## Future Test Ideas

As issue #35 is ongoing, consider adding tests for:

- Context interpolation (`${{ ctx.inputs.foo }}`)
- Environment variable injection
- Workspace mounting (when implemented)
- Parallel job execution (when implemented)
- Job outputs and step outputs
- Conditional execution
- Matrix builds
- Retry logic
- Timeout handling
