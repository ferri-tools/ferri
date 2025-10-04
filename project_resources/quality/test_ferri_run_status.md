# Test Instructions: Ferri Run Status Updates

## Objective

Verify that `ferri run` jobs properly transition from "Running" to "Completed" status and that output can be retrieved.

---

## Setup: Create Clean Test Environment

```bash
# Create a fresh test directory
mkdir -p /tmp/ferri-status-test-$(date +%s)
cd /tmp/ferri-status-test-$(date +%s)

# Initialize ferri
ferri init
```

---

## Test 1: Simple Command Status Transition

```bash
# Run a simple command
ferri run -- echo "test output"

# Wait 1 second for it to complete
sleep 1

# Check status in jobs.json directly
cat .ferri/jobs.json | grep -A 5 "status"

# Also check with ferri ps
ferri ps
```

### Expected Results:
- ✅ `jobs.json` shows `"status": "Completed"`
- ✅ `ferri ps` displays the job as "Completed"
- ✅ `ferri yank <job-id>` returns "test output"

---

## Test 2: Verify Job Files

```bash
# Check what files were created
ls -la .ferri/jobs/job-*/

# Expected files:
# - stdout.log (should contain "test output")
# - stderr.log (should be empty)
# - exit_code.log (should contain "0")
```

### Verify File Contents:

```bash
# Get the job ID from the previous test
JOB_ID=$(cat .ferri/jobs.json | grep '"id"' | tail -1 | cut -d'"' -f4)

# Check stdout
cat .ferri/jobs/$JOB_ID/stdout.log

# Check exit code
cat .ferri/jobs/$JOB_ID/exit_code.log

# Retrieve via yank
ferri yank $JOB_ID
```

---

## Test 3: Failed Command Handling

```bash
# Run a command that fails
ferri run -- bash -c "echo 'Error!' >&2; exit 1"

# Wait for completion
sleep 1

# Check status
cat .ferri/jobs.json | tail -20
```

### Expected Results:
- ✅ Status shows "Failed" (not "Running")
- ✅ `error_preview` field contains error message
- ✅ `ferri ps` shows job as "Failed"

---

## Test 4: Multiple Concurrent Jobs

```bash
# Launch 3 jobs
ferri run -- bash -c "sleep 1; echo 'Job 1'"
ferri run -- bash -c "sleep 1; echo 'Job 2'"
ferri run -- bash -c "sleep 1; echo 'Job 3'"

# Check initial status (should all be Running)
ferri ps

# Wait for completion
sleep 2

# Check final status (should all be Completed)
ferri ps

# Verify all outputs
ferri yank $(cat .ferri/jobs.json | grep '"id"' | tail -3 | head -1 | cut -d'"' -f4)
ferri yank $(cat .ferri/jobs.json | grep '"id"' | tail -2 | head -1 | cut -d'"' -f4)
ferri yank $(cat .ferri/jobs.json | grep '"id"' | tail -1 | cut -d'"' -f4)
```

---

## Debugging: If Status Stuck at "Running"

```bash
# Check if exit_code.log was created
ls -la .ferri/jobs/job-*/

# Check for monitor thread errors
cat .ferri/jobs/job-*/thread_error.log 2>/dev/null || echo "No error log"

# Check if process is still running
ps aux | grep [e]cho

# Manual status check
cat .ferri/jobs.json | jq '.[] | {id, status, pid}'
```

---

## Report Back

After running these tests, report:

1. **Status in `jobs.json`**: Does it say "Completed" or still "Running"?
2. **`ferri ps` output**: What statuses do you see?
3. **Files in job directory**: Which files exist? (`ls -la .ferri/jobs/job-*/`)
4. **Exit code file**: Does `exit_code.log` exist? What's in it?
5. **Thread errors**: Any `thread_error.log` files?

---

## Known Issues

### macOS Threading Issues
- Spawning from background threads can hang
- `Child.wait()` in threads can deadlock
- Current solution: Poll by PID using `sysinfo`

### If Status Never Updates
This indicates the monitor thread is not completing. Possible causes:
- PID polling loop not finding process exit
- File I/O error when writing `exit_code.log`
- Race condition between process exit and status check
