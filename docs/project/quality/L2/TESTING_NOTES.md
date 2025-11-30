# L2 Flow Schema Testing Notes

## Issue: Validation Errors Showing "missing field `name`"

**Reported:** Tests 2.3+ were showing generic "missing field `name`" errors instead of specific validation messages.

**Root Cause:** When the new-format parser failed validation, the CLI was silently falling back to the legacy parser. The legacy parser expected a different YAML structure (`name` at root level instead of `metadata.name`) and produced confusing error messages.

**Fix (Commit 635dfad):** Modified error handling in `main.rs` to capture and display the new-format parser's validation errors instead of falling back to legacy parser errors when both parsers fail.

---

## Validation Errors Now Working

All validation checks now produce clear, actionable error messages:

### ✅ Test 1.2: Invalid Kind
```
Error: Invalid kind 'Workflow', expected 'Flow'
```

### ✅ Test 1.3: Empty Jobs
```
Error: Flow must define at least one job
```

### ✅ Test 1.4: Job with No Steps
```
Error: Job 'build' must have at least one step
```

### ✅ Test 1.5: Step with Both run and uses
```
Error: Step 0 in job 'test' cannot have both 'run' and 'uses'
```

### ✅ Test 1.6: Step with Neither run nor uses
```
Error: Step 0 in job 'test' must have either 'run' or 'uses'
```

### ✅ Test 2.3: Non-existent Dependency
```
Error: Job 'build' depends on non-existent job 'nonexistent'
```

### ✅ Test 2.4: Self Dependency
```
Error: Job 'build' cannot depend on itself
```

### ✅ Test 4.2: Invalid Workspace Reference
```
Error: Step in job 'build' references non-existent workspace 'nonexistent'
```

---

## How to Test

After installing the updated binary:

```bash
cargo install --path ferri-cli --force

# Run the manual test plan
cd /tmp && mkdir ferri-flow-test && cd ferri-flow-test
ferri init

# Follow test plan at:
# project_resources/quality/L2/flow_schema_manual_test_plan.md
```

All tests in the manual test plan should now produce the expected validation errors.

---

## Updated: 2025-10-04
## Status: ✅ Fixed and Verified
