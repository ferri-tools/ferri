# Ferri L1 Stability Check: Manual Verification Guide

This document provides a series of manual tests to verify the stability and core functionality of Ferri's Layer 1 (L1) commands: `init`, `secrets`, `models`, `ctx`, and `with`.

## Prerequisites

1.  A working terminal with `ferri` installed and in the system's PATH.
2.  Ollama installed and running with at least one model pulled (e.g., `ollama pull llama3`).
3.  A valid API key for a remote model provider (e.g., Google, OpenAI) for testing remote model features.

## Test Environment Setup

Before starting, create a clean directory for testing.

```bash
mkdir ferri_l1_test
cd ferri_l1_test
```

---

## 1. `ferri init` Verification

### Test Steps:

1.  Run the initialization command:
    ```bash
    ferri init
    ```

### Expected Outcome:

*   The command prints a success message.
*   A new directory named `.ferri` is created in the current folder.
*   The `.ferri` directory contains the following files:
    *   `secrets.json`
    *   `models.json`
    *   `context.json`

### Verification Command:

```bash
ls -a .ferri
```
*(Expected output: `.` `..` `context.json` `models.json` `secrets.json`)*

---

## 2. `ferri secrets` Verification

### Test Steps:

1.  **Set a secret:**
    ```bash
    ferri secrets set TEST_API_KEY "sk-12345"
    ```
2.  **List secrets:**
    ```bash
    ferri secrets ls
    ```
3.  **Remove the secret:**
    ```bash
    ferri secrets rm TEST_API_KEY
    ```
4.  **List secrets again:**
    ```bash
    ferri secrets ls
    ```

### Expected Outcome:

*   The `set` command confirms the secret was set.
*   The first `ls` command lists `TEST_API_KEY`.
*   The `rm` command confirms the secret was removed.
*   The second `ls` command shows "No secrets found." or an empty list.

---

## 3. `ferri models` Verification

### Test Steps:

1.  **List auto-discovered models:**
    ```bash
    ferri models ls
    ```
2.  **Add a remote model:** (Replace with your actual API key secret name)
    ```bash
    # First, ensure the secret is set
    ferri secrets set GOOGLE_API_KEY "your-real-api-key"

    # Now, add the model
    ferri models add gemini-test --provider google --model-name gemini-pro --api-key-secret GOOGLE_API_KEY
    ```
3.  **List models again:**
    ```bash
    ferri models ls
    ```
4.  **Remove the remote model:**
    ```bash
    # Confirm with 'y' when prompted
    ferri models rm gemini-test
    ```
5.  **List models a final time:**
    ```bash
    ferri models ls
    ```

### Expected Outcome:

*   The first `ls` command shows your local Ollama models (e.g., `llama3`).
*   The `add` command confirms the model was added.
*   The second `ls` command shows both the local Ollama models and the new `gemini-test` model.
*   The `rm` command confirms the model was removed.
*   The final `ls` command shows only the local Ollama models again.

---

## 4. `ferri ctx` Verification

### Test Steps:

1.  **Create a test file:**
    ```bash
    echo "This is a test file for the Ferri context." > sample_context.txt
    ```
2.  **Add the file to the context:**
    ```bash
    ferri ctx add sample_context.txt
    ```
3.  **List the context:**
    ```bash
    ferri ctx ls
    ```
4.  **Remove the file from the context:**
    ```bash
    ferri ctx rm sample_context.txt
    ```
5.  **List the context again:**
    ```bash
    ferri ctx ls
    ```

### Expected Outcome:

*   The `add` command confirms the path was added.
*   The first `ls` command lists `sample_context.txt`.
*   The `rm` command confirms the path was removed.
*   The second `ls` command shows "Context is empty."

---

## 5. `ferri with` Verification

### Test A: Simple Command Execution

1.  **Run a standard shell command:**
    ```bash
    ferri with -- echo "Hello from Ferri"
    ```
*   **Expected Outcome:** The terminal prints `Hello from Ferri`.

### Test B: Secret Injection

1.  **Set a secret:**
    ```bash
    ferri secrets set GREETING "Hello from a secret"
    ```
2.  **Run a command that uses the secret as an environment variable:**
    ```bash
    ferri with -- sh -c 'echo $GREETING'
    ```
*   **Expected Outcome:** The terminal prints `Hello from a secret`.

### Test C: Context Injection with a Model

1.  **Add a file to the context:**
    ```bash
    echo "The primary goal of Ferri is to simplify AI development workflows." > goal.txt
    ferri ctx add goal.txt
    ```
2.  **Run a command with a local model using the context:** (Replace `llama3` with your local model if different)
    ```bash
    ferri with --ctx --model llama3 "Based on the context, what is the primary goal of Ferri?"
    ```
*   **Expected Outcome:** The model responds with something similar to "The primary goal of Ferri is to simplify AI development workflows."

---

## Cleanup

When you are finished, you can remove the test directory.

```bash
cd ..
rm -rf ferri_l1_test
```
