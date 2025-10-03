# Demo: Ferri Run Refactor - Background Jobs on macOS

**Objective:** Validate that `ferri run` works correctly on macOS after the refactor that fixed issue #12.

**What Was Fixed:** Jobs now properly transition from "Running" → "Completed"/"Failed" status, and output can be retrieved with `ferri yank`.

**Test Environment:** macOS (also works on Linux with additional process group features)

---

## Setup

### Create a Clean Test Directory

```bash
mkdir -p /tmp/ferri-demo
cd /tmp/ferri-demo
ferri init
```

**Expected Output:**
```
Project initialized successfully! 🦀
```

---

## Test 1: Basic Shell Command

**Goal:** Verify simple shell commands run in background and complete successfully.

### Step 1: Run a simple echo command

```bash
ferri run -- echo "Hello from background job"
```

**Expected Output:**
```
Successfully submitted job 'job-xxxxxx'.
Process ID: 12345
```

### Step 2: Check job status immediately

```bash
ferri ps
```

**Expected:** You should see the job with status either "Running" or "Completed". Press `q` to exit.

### Step 3: Wait and retrieve output

```bash
sleep 1
ferri yank job-xxxxxx  # Replace with your actual job ID
```

**Expected Output:**
```
Hello from background job
```

✅ **Success Criteria:** Output is retrieved correctly, not "Job produced no output or is still running."

---

## Test 2: Multi-Line Output

**Goal:** Verify larger outputs are captured correctly.

### Run a command with multiple lines

```bash
ferri run -- bash -c "echo 'Line 1'; echo 'Line 2'; echo 'Line 3'"
```

### Retrieve output

```bash
sleep 1
ferri yank job-xxxxxx  # Replace with your job ID
```

**Expected Output:**
```
Line 1
Line 2
Line 3
```

---

## Test 3: Command with Exit Code

**Goal:** Verify failed commands are properly marked as "Failed" with error previews.

### Run a failing command

```bash
ferri run -- bash -c "echo 'Error message' >&2; exit 1"
```

### Check status

```bash
sleep 1
ferri ps
```

**Expected:** Job status should be "Failed", not stuck in "Running".

### Retrieve output

```bash
ferri yank job-xxxxxx
```

**Expected:** Should show the error message from stderr.

---

## Test 4: Long-Running Command

**Goal:** Verify status updates work for commands that take time.

### Run a sleep command

```bash
ferri run -- bash -c "echo 'Starting...'; sleep 3; echo 'Done!'"
```

### Monitor status changes

```bash
# Check immediately
ferri ps  # Should show "Running"

# Wait and check again
sleep 4
ferri ps  # Should now show "Completed"
```

### Retrieve output

```bash
ferri yank job-xxxxxx
```

**Expected Output:**
```
Starting...
Done!
```

---

## Test 5: AI Model Job (Optional - Requires Ollama)

**Goal:** Verify AI models work with background jobs.

### Prerequisites

```bash
# Install Ollama and pull a small model
# brew install ollama  # if not installed
# ollama pull gemma:2b

# Add the model to ferri
ferri models add gemma --provider ollama --model-name gemma:2b
```

### Run an AI job

```bash
ferri run --model gemma -- "Write a haiku about Rust programming"
```

### Monitor and retrieve

```bash
# Check status (might take 10-30 seconds)
ferri ps

# Once completed, get the haiku
ferri yank job-xxxxxx
```

**Expected:** A haiku about Rust appears.

---

## Test 6: Multiple Concurrent Jobs

**Goal:** Verify multiple jobs can run simultaneously.

### Launch 3 jobs at once

```bash
ferri run -- bash -c "sleep 2; echo 'Job 1 done'"
ferri run -- bash -c "sleep 2; echo 'Job 2 done'"
ferri run -- bash -c "sleep 2; echo 'Job 3 done'"
```

### Monitor all jobs

```bash
ferri ps  # Should show 3 running jobs
sleep 3
ferri ps  # Should show 3 completed jobs
```

### Retrieve each output

```bash
ferri yank job-aaa
ferri yank job-bbb
ferri yank job-ccc
```

**Expected:** Each shows its respective "Job X done" message.

---

## Test 7: Output to File

**Goal:** Verify the `--output` flag works.

### Run with output file

```bash
ferri run --output result.txt -- echo "Saved to file"
```

### Verify file was created

```bash
sleep 1
cat result.txt
```

**Expected Output:**
```
Saved to file
```

---

## Success Checklist

If all tests pass, the refactor is successful:

- ✅ Jobs spawn and execute on main thread
- ✅ Status transitions from "Running" → "Completed"/"Failed"
- ✅ `ferri yank` retrieves output correctly
- ✅ Failed jobs show error messages
- ✅ Multiple concurrent jobs work
- ✅ Output files are created properly
- ✅ No jobs stuck in "Running" state forever

---

## What Changed Under the Hood

**Old Architecture (Broken on macOS):**
- Spawned processes from background threads
- Used pipes to capture stdout/stderr
- Caused deadlocks with `pre_exec` hooks on macOS

**New Architecture (Works on macOS):**
- Spawns processes on **main thread**
- Redirects stdout/stderr **directly to files**
- Background thread only monitors completion
- Platform-specific `pre_exec` handling (`#[cfg(target_os = "linux")]`)

**Platform Differences:**
- **Linux:** Full process group management (kills all child processes)
- **macOS:** Basic process management (kills direct process only)

This is a reasonable tradeoff that makes `ferri run` reliable across platforms.
