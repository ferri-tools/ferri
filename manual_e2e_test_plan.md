# Ferri Comprehensive End-to-End Test Plan

This document outlines the official manual verification process for the Ferri CLI. It is designed to validate the system's integrity, spanning from initialization to advanced agentic flow orchestration.

**Goal:** Certify the release quality of Ferri by executing core workflows using both local (Ollama) and cloud (Google Gemini) models.

## Prerequisites

1.  **Ferri Binary:** Ensure the latest `ferri` binary is in your `PATH` or available in `target/debug/`.
2.  **Ollama:** Installed and running locally (`ollama serve`).
3.  **Google API Key:** A valid API key for Gemini models.
4.  **Clean Slate:** Start in a new directory or clean existing artifacts.

---

## Phase 1: Environment Initialization

**Objective:** Verify fresh project setup and directory structure.

1.  **Create Test Directory:**
    ```bash
    mkdir ferri-uat
    cd ferri-uat
    ```

2.  **Initialize Ferri:**
    ```bash
    ferri init
    ```
    *   **Verify:**
        *   ASCII art logo is displayed.
        *   `.ferri/` directory exists.
        *   `.gitignore` is created/updated.
        *   `ferri doctor` runs without errors (if implemented).

---

## Phase 2: Local Model Configuration (Ollama)

**Objective:** Validate integration with local inference via Ollama.

3.  **Prepare Ollama:**
    ```bash
    ollama pull gemma:2b
    ```
    *   *Note: Using `gemma:2b` for speed. You may substitute with `llama3` or others.*

4.  **Register Local Model:**
    ```bash
    ferri models add gemma \
      --provider ollama \
      --model-name "gemma:2b"
    ```

5.  **Verify Registry:**
    ```bash
    ferri models ls
    ```
    *   **Check:** `gemma` alias lists `ollama` as provider.

6.  **Smoke Test (Local):**
    ```bash
    ferri with --model gemma -- "Explain what Rust is in 5 words."
    ```
    *   **Expected:** A concise response generated locally.

---

## Phase 3: Cloud Model Configuration (Google Gemini)

**Objective:** Validate secure credential storage and remote API integration.

7.  **Secure API Key:**
    ```bash
    ferri secrets set GOOGLE_API_KEY
    # Enter your actual key when prompted
    ```

8.  **Register Cloud Model:**
    ```bash
    ferri models add gemini \
      --provider google \
      --model-name "gemini-2.5-pro" \
      --api-key-secret GOOGLE_API_KEY
    ```

9.  **Verify Registry:**
    ```bash
    ferri models ls
    ```
    *   **Check:** `gemini` alias lists `google` as provider.

10. **Smoke Test (Cloud):**
    ```bash
    ferri with --model gemini -- "What is the capital of France?"
    ```
    *   **Expected:** "Paris."

---

## Phase 4: Context & File Awareness

**Objective:** Ensure the CLI can ingest local files into the model's context window.

11. **Create Artifacts:**
    ```bash
    echo "The secret code is 42." > secret.txt
    ```

12. **Add to Context:**
    ```bash
    ferri ctx add secret.txt
    ```
    *   **Verify:** `ferri ctx ls` lists `secret.txt`.

13. **Context-Aware Query:**
    ```bash
    ferri with --model gemma --ctx -- "What is the secret code in the file?"
    ```
    *   **Expected:** The model answers "42".

14. **Cleanup Context:**
    ```bash
    ferri ctx rm secret.txt
    ferri ctx ls  # Should be empty
    ```

---

## Phase 5: Job Lifecycle (Background Execution)

**Objective:** Test the async job runner, process monitoring, and output retrieval.

15. **Submit Background Job:**
    ```bash
    ferri run --model gemini -- "Write a Python script to print Fibonacci numbers."
    ```
    *   **Action:** Note the returned **Job ID** (e.g., `job-abc1234`).

16. **Monitor Status:**
    ```bash
    ferri ps
    ```
    *   **Verify:** Job appears in the list. Wait for status `Succeeded`.

17. **Retrieve Output:**
    ```bash
    ferri yank <JOB_ID>
    ```
    *   **Expected:** The Python code is displayed.

---

## Phase 6: Canonical Flows Execution

**Objective:** Verify the Flow Orchestrator using standard example flows.

18. **Dependency Test (Hello-Bye):**
    *   *Create `hello-bye.yml` content:*
    ```yaml
    apiVersion: ferri.flow/v1alpha1
    kind: Flow
    metadata:
      name: simple-echo-flow
    spec:
      jobs:
        say-hello:
          runs_on: process
          steps:
            - run: echo "Hello!"
        say-goodbye:
          needs: [say-hello]
          steps:
            - run: echo "Goodbye!"
    ```
    *   **Run:**
        ```bash
        ferri flow run hello-bye.yml
        ```
    *   **Verify (TUI):** Both jobs turn Green. Dependencies are respected.

19. **Model Integration Flow (Gemma Poem):**
    *   *Create `gemma-poem.yml` content:*
    ```yaml
    apiVersion: ferri.flow/v1alpha1
    kind: Flow
    metadata:
      name: poem-generator
    spec:
      jobs:
        write-poem:
          steps:
            - name: "Generate Poem"
              # Ensure the path to 'ferri' is correct or in PATH
              run: ferri with --model gemma --output poem.txt -- "write a haiku about code"
        display:
          needs: [write-poem]
          steps:
            - run: cat poem.txt
    ```
    *   **Run:**
        ```bash
        ferri flow run gemma-poem.yml
        ```
    *   **Verify:** The flow completes, and `poem.txt` contains a haiku.

---

## Phase 7: Agentic Automation (`ferri do`)

**Objective:** Test the autonomous flow generation capabilities.

20. **Generate a Workspace Flow:**
    ```bash
    ferri do "Create a flow that creates a directory called 'test_data', writes a file 'data.txt' inside it with the current date, and then lists the directory content."
    ```
    *   **Observation:**
        *   Ferri generates a plan.
        *   Ferri executes the generated YAML.
        *   TUI shows the progress of the generated jobs.
    *   **Verification:** Check if `test_data/data.txt` exists.

---

## Phase 8: System Health

21. **Ferri Doctor:**
    ```bash
    ferri doctor
    ```
    *   **Verify:** All checks pass (Ollama connection, permissions, etc.).

---

**Success Criteria:**
If all 21 steps complete without error, the release candidate is approved.