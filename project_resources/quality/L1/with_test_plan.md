# Manual QA Test Plan: `with` and `run` Commands

This document outlines the manual testing steps for the `ferri with` and `ferri run` commands, focusing on their unified syntax.

**Prerequisites:**
1.  A `ferri` project must be initialized (`ferri init`).
2.  You should be in your test project directory.
3.  An Ollama model (e.g., `gemma:2b`) should be available.

---

### Test Case 1: Executing a Simple Command

**Goal:** Verify that `ferri with` can run a basic shell command.

**Steps:**

1.  **Run `echo`:**
    *   The `--` separates `ferri`'s arguments from the command to be executed.
    ```bash
    ferri with -- echo "Hello from Ferri!"
    ```
2.  **Verify the Output:**
    *   You should see the output printed directly to your terminal:
        ```
        Hello from Ferri!
        ```

---

### Test Case 2: Secret Injection

**Goal:** Verify that secrets are made available as environment variables.

**Steps:**

1.  **Set a secret:**
    ```bash
    ferri secrets set MY_SECRET_MESSAGE "it_works"
    ```
2.  **Run a command that prints the secret:**
    ```bash
    ferri with -- printenv MY_SECRET_MESSAGE
    ```
3.  **Verify the Output:**
    *   The command should print the value of the secret:
        ```
        it_works
        ```

---

### Test Case 3: Model and Context Injection (`with`)

**Goal:** Verify that `ferri with` correctly uses a specified model and injects context.

**Steps:**

1.  **Add a model alias:**
    ```bash
    ferri models add gemma --provider ollama --model-name gemma:2b
    ```
2.  **Create and add a context file:**
    ```bash
    echo "This is the context." > my_file.txt
    ferri ctx add my_file.txt
    ```
3.  **Run `with` using the model and context:**
    ```bash
    ferri with --model gemma --ctx "What is the content of the provided file?"
    ```
4.  **Verify the Output:**
    *   The model should respond with something similar to:
        > The content of the provided file is "This is the context."

---

### Test Case 4: Model and Context Injection (`run`)

**Goal:** Verify that `ferri run` works with the same syntax as `with` for background jobs.

**Steps:**

1.  **Run the same command in the background:**
    ```bash
    ferri run --model gemma --ctx "What is the content of the provided file?"
    ```
2.  **Verify the Job ID:**
    *   You should see a success message with a job ID, e.g., `Successfully submitted job 'job-xxxxxx'`.

3.  **Check the job status:**
    *   Wait a few seconds for the job to complete.
    ```bash
    ferri ps
    ```
    *   The status of your job should be `Completed`.

4.  **Yank the output:**
    *   Replace `job-xxxxxx` with your actual job ID.
    ```bash
    ferri yank job-xxxxxx
    ```
5.  **Verify the Output:**
    *   The output should be the same as the output from the `with` command in the previous test case. This confirms the unified logic works for background jobs.
