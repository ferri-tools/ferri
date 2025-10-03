# Test: Background Jobs with Status Updates (Ollama Local Models)

**Objective:** Verify that `ferri run` jobs properly transition from "Running" to "Completed" status for local Ollama models.

**Note:** Remote model support (`--model gemini`) is not yet implemented for background jobs.

---

## Prerequisites

1. Rebuild and install `ferri`:
   ```bash
   cd /Users/jorgeajimenez/repos/ferri
   cargo build --release
   cargo install --path .
   ```

2. Ensure Ollama is running with a model pulled:
   ```bash
   ollama pull gemma:2b
   ```

---

## Setup: Clean Test Environment

```bash
# Create fresh test directory
mkdir -p /tmp/ferri-jobs-test-$(date +%s)
cd /tmp/ferri-jobs-test-$(date +%s)

# Initialize ferri
ferri init
```

---

## Configure Ollama Model

```bash
# Add local Ollama model
ferri models add gemma --provider ollama --model-name gemma:2b

# Verify model is registered
ferri models ls
```

---

## Test 1: Simple Local Job

```bash
# Run a quick local job
ferri run --model gemma -- "Write a haiku about Rust programming"

# Immediately check status (should show "Running")
ferri ps

# Wait 5 seconds for completion
sleep 5

# Check status again (should show "Completed")
ferri ps

# Get the job ID from the last run
JOB_ID=$(cat .ferri/jobs.json | grep '"id"' | tail -1 | cut -d'"' -f4)

# Retrieve the output
ferri yank $JOB_ID
```

**Expected Results:**
- ✅ Job initially shows "Running" in `ferri ps`
- ✅ After completion, job shows "Completed" in `ferri ps`
- ✅ `ferri yank` returns the haiku output

---

## Test 2: Multiple Concurrent Jobs

```bash
# Launch 3 jobs at once
ferri run --model gemma -- "Name 3 Rust crates for web development"
ferri run --model gemma -- "What is cargo?"
ferri run --model gemma -- "Explain borrowing in one sentence"

# Check all are running
ferri ps

# Wait for completion
sleep 8

# Verify all completed
ferri ps

# View all outputs
cat .ferri/jobs.json | grep '"id"' | tail -3 | cut -d'"' -f4 | while read job; do
  echo "=== $job ==="
  ferri yank $job
  echo ""
done
```

**Expected Results:**
- ✅ All 3 jobs show "Running" initially
- ✅ All 3 transition to "Completed"
- ✅ All outputs are retrievable

---

## Test 3: Long-Running Job with Real-Time Status Checks

```bash
# Start a longer generation
ferri run --model gemma -- "Write a 300-word story about a developer debugging code at 3am"

# Check status multiple times while it runs
ferri ps
sleep 3
ferri ps
sleep 3
ferri ps
sleep 5

# Final check (should be completed)
ferri ps

# Get output
JOB_ID=$(cat .ferri/jobs.json | grep '"id"' | tail -1 | cut -d'"' -f4)
ferri yank $JOB_ID
```

**Expected Results:**
- ✅ Status updates correctly on each `ferri ps` call
- ✅ Final status is "Completed"
- ✅ Full story is retrieved

---

## Test 4: Verify Job Files Created Correctly

```bash
# Run a simple job
ferri run --model gemma -- "Count to 5"

# Get the job ID
JOB_ID=$(cat .ferri/jobs.json | grep '"id"' | tail -1 | cut -d'"' -f4)

# Wait for completion
sleep 5

# Check files were created
ls -la .ferri/jobs/$JOB_ID/

# Expected files:
# - stdout.log (contains the output)
# - stderr.log (should be empty or minimal)

# Check stdout content
cat .ferri/jobs/$JOB_ID/stdout.log

# Verify ferri ps shows completed
ferri ps
```

**Expected Results:**
- ✅ Job directory exists with stdout.log and stderr.log
- ✅ stdout.log contains the model output
- ✅ Status shows "Completed" in `ferri ps`

---

## Debugging

If jobs stay "Running":

```bash
# Check if process still exists
ps aux | grep -i ollama
ps aux | grep -i ferri

# Check job files
ls -la .ferri/jobs/job-*/

# Check jobs.json directly
cat .ferri/jobs.json | tail -30

# Check stdout/stderr logs
cat .ferri/jobs/job-*/stdout.log
cat .ferri/jobs/job-*/stderr.log

# Check if PID is still valid
cat .ferri/jobs.json | grep '"pid"' | tail -1
```

---

## Success Criteria

All of the following must work:

1. ✅ Local (Ollama) jobs complete and update status
2. ✅ Multiple concurrent jobs all transition correctly
3. ✅ `ferri ps` shows accurate real-time status
4. ✅ `ferri yank` retrieves all outputs successfully
5. ✅ Job files (stdout.log, stderr.log) are created properly

---

## Known Limitations

- ❌ Remote models (Gemini, etc.) are not yet supported in `ferri run`
- ✅ Only local Ollama models work with background jobs currently
