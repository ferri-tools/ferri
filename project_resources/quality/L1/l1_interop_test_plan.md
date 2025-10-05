# Manual QA Test Plan: L1 Command Interoperability

This document outlines a manual, end-to-end test case to verify that the core L1 commands (`init`, `secrets`, `ctx`, `with`, `run`) work together seamlessly.

**Prerequisites:**
1.  Be in a clean test directory (e.g., `~/ferri-e2e-test`).

---

### The Goal

We will verify that `ferri with` and `ferri run` can correctly use secrets and context together. We will use:
- `ferri secrets` to store a value.
- `ferri ctx` to define a file that will be our context.
- `ferri with` to execute a command that prints both the secret and the context.
- `ferri run` to do the same in the background.

---

### Test Steps

**1. Initialize the Project**
```bash
ferri init
```
*Verification: Should see the success message.*

**2. Store a Secret**
```bash
ferri secrets set MY_TEST_SECRET "hello_secret"
```
*Verification: Should see the "Secret... set successfully" message.*

**3. Create and Add a Context File**
```bash
echo "hello_context" > my_context.txt
ferri ctx add my_context.txt
```
*Verification: Run `ferri ctx ls` to confirm the file was added.*

**4. Execute and Verify with `ferri with`**
*   This command will print the environment variable to show the secret was injected, and it will use `cat` to receive the context via stdin.
```bash
ferri with --ctx -- sh -c 'echo $MY_TEST_SECRET && cat'
```
*   **Important:** After running the command, you must manually type `prompt` and press Enter to provide the final argument that the context will be prepended to.

*   **Verification:** The output should be:
    ```
    hello_secret
    ---
    File: my_context.txt
    ---
    hello_context

    prompt
    ```
*   This confirms that `with` injects both secrets (as env vars) and context (into the arguments).

**5. Execute and Verify with `ferri run`**
*   Now we run the same logic as a background job.
```bash
ferri run --ctx -- sh -c 'echo $MY_TEST_SECRET && echo "background_prompt"'
```
*Verification: You should get a job ID, e.g., `job-xxxxxx`.*

*   Wait a moment for the job to complete, then check the status:
```bash
ferri ps
```
*Verification: The job should be marked as `Completed`.*

*   Finally, retrieve the output:
```bash
ferri yank <your_job_id>
```
*   **Verification:** The output should contain both the secret and the prompt:
    ```
    hello_secret
    background_prompt
    ```

This completes the end-to-end test of the unified `with` and `run` command interoperability.
