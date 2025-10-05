# Manual QA Test Plan

This document outlines the manual testing steps to verify the core functionality of the `ferri` CLI.

**Prerequisites:**
1.  Ensure `ferri` is installed globally. If you just made changes, reinstall it by navigating to the project root and running:
    ```bash
    cargo install --path ferri-cli --force
    ```
2.  Create a new, empty directory *outside* of the `ferri` project to simulate a real user environment.
    ```bash
    mkdir ~/ferri-test-project && cd ~/ferri-test-project
    ```

---

### Test Case 1: `ferri init`

**Goal:** Verify that the `init` command correctly sets up a new project environment.

**Steps:**

1.  **Navigate to your test directory:**
    ```bash
    # Make sure you are in your new test project directory
    pwd
    ```

2.  **Run the init command:**
    ```bash
    ferri init
    ```

3.  **Verify the Output:**
    *   You should see the success message: `Successfully initialized Ferri project in ./.ferri`

4.  **Verify the Directory and Files:**
    *   Check that the `.ferri` directory was created:
        ```bash
        ls -la
        ```
        (You should see a `.ferri` directory in the list).
    *   Check that the default state files were created inside `.ferri`:
        ```bash
        ls .ferri
        ```
        (You should see `context.json`, `models.json`, and `secrets.json`).

5.  **Verify File Contents:**
    *   Check that the JSON files have the correct initial content:
        ```bash
        cat .ferri/context.json # Should print []
        cat .ferri/models.json  # Should print []
        cat .ferri/secrets.json # Should print {}
        ```

---

### Test Case 2: `ferri secrets set`

**Goal:** Verify that the `secrets set` command can correctly add and encrypt a secret.

**Steps:**

1.  **Set a new secret:**
    *   From within your initialized test project directory, run:
        ```bash
        ferri secrets set MY_API_KEY "123-abc-456-def"
        ```

2.  **Verify the Output:**
    *   You should see the success message: `Secret 'MY_API_KEY' set successfully.`

3.  **Verify the Encrypted Content:**
    *   Look at the contents of the `secrets.json` file:
        ```bash
        cat .ferri/secrets.json
        ```
    *   **IMPORTANT:** You should **NOT** see your key `"123-abc-456-def"` in plain text. The output should be a JSON object containing a long, encrypted string, for example:
        ```json
        {
          "encrypted_data": "a-long-random-looking-string"
        }
        ```
    *   This confirms the encryption is working at a basic level.

---
