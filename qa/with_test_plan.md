# Manual QA Test Plan: `with` Command

This document outlines the manual testing steps for the `ferri with` command.

**Prerequisites:**
1.  A `ferri` project must be initialized (`ferri init`).
2.  You should be in your test project directory.

---

### Test Case 1: Executing a Simple Command

**Goal:** Verify that `ferri with` can run a basic shell command.

**Steps:**

1.  **Run `echo`:**
    *   The `--` is important. It tells `ferri` that everything after it is the command to be executed.
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

**Goal:** Verify that secrets stored in `ferri` are made available as environment variables to the command being run.

**Steps:**

1.  **Set a secret:**
    *   First, store a secret that we can test with.
    ```bash
    ferri secrets set MY_SECRET_MESSAGE "it_works"
    ```
    *   You should see a success message.

2.  **Run a command that prints the secret:**
    *   We will use the `printenv` command (available on macOS and Linux) to print the value of the environment variable.
    ```bash
    ferri with -- printenv MY_SECRET_MESSAGE
    ```
3.  **Verify the Output:**
    *   The command should print the value of the secret you set:
        ```
        it_works
        ```
    *   This confirms that `ferri with` successfully decrypted your secret and injected it into the environment for the command it ran.

---
