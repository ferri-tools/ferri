# QA Walkthrough: Long-Running Jobs & `ps` Dashboard

**Objective:** This document provides a guide to test the creation of long-running background jobs with `ferri run` and to verify their status using the `ferri ps` command.

**Goal:** Successfully start multiple background jobs and view their real-time status in the `ps` dashboard.

---

### Step 1: Start a Long-Running Job

First, let's create a simple, long-running process. This command will start a process that prints the date every five seconds and runs in the background.

**Command:**
```bash
ferri run -- "while true; do date; sleep 5; done"
```

**Expected Result:**
You will see a message confirming the job was submitted, along with its unique ID.
```
Successfully submitted job 'job-xxxxxx'.
Process ID: 12345
```
*(Note: The job ID and process ID will be different on your machine.)*

---

### Step 2: Start a Job That Finishes Quickly

Next, let's create a job that completes almost instantly to ensure we can see different statuses in the dashboard.

**Command:**
```bash
ferri run -- "echo 'This job is already finished.'"
```

**Expected Result:**
You will see another confirmation message for the new job.
```
Successfully submitted job 'job-yyyyyy'.
Process ID: 12346
```

---

### Step 3: Start a Job with a Model

Now, let's start a more complex background job that uses an AI model. This will simulate a real-world use case, like generating a large piece of text.

**Prerequisite:** Ensure you have a local model like `gemma` registered.
```bash
ferri models add gemma --provider ollama --model-name gemma
```

**Command:**
```bash
ferri run --model gemma -- "write a short history of the internet"
```

**Expected Result:**
A third confirmation message will appear.
```
Successfully submitted job 'job-zzzzzz'.
```

---

### Step 4: View the `ps` Dashboard

With three jobs running (one long-running, one finished, one in progress), we can now view the dashboard.

**Command:**
```bash
ferri ps
```

**Expected Result:**
1.  The terminal will clear and a Text-based User Interface (TUI) will appear.
2.  You will see a table listing the three jobs you created (`job-xxxxxx`, `job-yyyyyy`, `job-zzzzzz`).
3.  The statuses should be varied:
    *   The first job (`while true...`) should show as **Running**.
    *   The second job (`echo...`) should show as **Completed**.
    *   The third job (`--model gemma...`) will likely show as **Running** or **Completed**, depending on how fast it finishes.
4.  You can use the **up and down arrow keys** to select different jobs in the list.
5.  When you select a job, the panel on the right should update to show the full command and the output of that job.
6.  Pressing **'q'** will exit the TUI and return you to your normal terminal prompt.

---

This concludes the long-running jobs walkthrough. If you can see all three jobs with their different statuses and view their output, the feature is working correctly.
