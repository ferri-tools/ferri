# Ferri Layer 1 (L1) Manual Test Plan

## 1. Introduction

This document provides a step-by-step manual testing plan for the Layer 1 (L1) functionality of the Ferri toolkit. The goal is to verify the correctness and interoperability of the core commands: `init`, `secrets`, `models`, `ctx`, and `with`.

A key focus of this plan is to demonstrate that `ferri with` serves as a **unified execution interface** for three distinct command types:
1.  Standard local shell commands.
2.  Prompts for local AI models (Ollama).
3.  API calls to remote AI models (e.g., Google Gemini).

## 2. Prerequisites

Before starting, ensure you have the following:
- `ferri` compiled and available in your system's `PATH`.
- `ollama` installed and running with at least one model pulled (e.g., `ollama pull gemma:2b`).
- A Google AI API key for testing remote model integration.
- A clean directory for a new test project.

## 3. Test Execution

### Part 1: Project Initialization

This part verifies that a new Ferri project can be created successfully.

1.  **Create a test directory and navigate into it:**
    ```
    mkdir ferri-l1-test
    cd ferri-l1-test
    ```

2.  **Initialize the project:**
    ```
    ferri init
    ```
    - **Expected Result:** The command should print `✨ Successfully initialized Ferri project in ./.ferri`. A `.ferri` directory should be created containing `secrets.json`, `models.json`, and `context.json`.

### Part 2: Configuration (`secrets` and `models`)

This part verifies the setup of secrets for remote models and the registration of both local and remote models.

1.  **Set the Google API Key:**
    *Replace `YOUR_GOOGLE_API_KEY` with your actual key.*
    ```
    ferri secrets set GOOGLE_API_KEY "YOUR_GOOGLE_API_KEY"
    ```
    - **Expected Result:** The command should succeed without error. The key will be stored in an encrypted format within `.ferri/secrets.json`.

2.  **Register a local Ollama model as an alias:**
    *This tests the manual registration of a local model. Replace `gemma:2b` with the name of an Ollama model you have pulled.*
    ```
    ferri models add gemma --provider ollama --model-name gemma:2b
    ```
    - **Expected Result:** The command should succeed without error.

3.  **Register the remote Google model:**
    ```
    ferri models add gemini-pro \
      --provider google \
      --api-key-secret GOOGLE_API_KEY \
      --model-name gemini-1.5-pro-latest
    ```
    - **Expected Result:** The command should succeed without error.

4.  **List all models to verify:**
    ```
    ferri models ls
    ```
    - **Expected Result:** The output should be a table listing the auto-discovered models, your new `gemma` alias, and the `gemini-pro` model.

### Part 3: Context Management (`ctx`)

This part verifies that both individual files and entire directories can be added to the context.

1.  **Create a sample directory and file structure:**
    ```
    mkdir src
    echo 'print("Hello from a Python script!")' > src/main.py
    echo '# My Project' > README.md
    ```

2.  **Add the directory and the file to the context:**
    ```
    ferri ctx add src README.md
    ```
    - **Expected Result:** The command should succeed without error.

3.  **List the context to verify:**
    ```
    ferri ctx ls
    ```
    - **Expected Result:** The output should list the full, canonical paths to the `src` directory and the `README.md` file.

### Part 4: The Unified Execution Interface (`ferri with`)

This is the core of the L1 test. These steps demonstrate that the exact same command structure (`ferri with`) can be used for vastly different tasks, showcasing its power as a unified interface.

**Use Case A: Standard Shell Commands**

1.  **Execute a simple `echo` command:**
    ```
    ferri with -- echo "Hello from Ferri!"
    ```
    - **Expected Result:** The output should be `Hello from Ferri!`.

2.  **Execute a command that requires a secret:**
    *This command uses `ferri with` to inject the `GOOGLE_API_KEY` secret as an environment variable, which `printenv` then displays.*
    ```
    ferri with -- printenv GOOGLE_API_KEY
    ```
    - **Expected Result:** The output should be your actual Google API key, demonstrating that `ferri with` correctly injects secrets.

**Use Case B: Local AI Models (Ollama)**

1.  **Run a prompt without context using the alias:**
    ```
    ferri with --model gemma "What is the Rust programming language?"
    ```
    - **Expected Result:** The Ollama model should stream a response explaining what Rust is.

2.  **Run a prompt WITH context using the alias:**
    *This command uses the `--ctx` flag to inject the content of `src/main.py` and `README.md` into the prompt.*
    ```
    ferri with --ctx --model gemma "Based on the context, what is this project about and what does the script do?"
    ```
    - **Expected Result:** The Ollama model should stream a response explaining that the project is "My Project" (from the README) and that the script prints a hello message (from `main.py`).

**Use Case C: Remote AI Models (API)**

1.  **Run a prompt without context:**
    ```
    ferri with --model gemini-pro "What is the Rust programming language?"
    ```
    - **Expected Result:** The Gemini API should stream a response explaining what Rust is. The behavior should be identical to the local model test, just using a different backend.

2.  **Run a prompt WITH context:**
    *This command uses the same `--ctx` flag to inject the content of `src/main.py` and `README.md`.*
    ```
    ferri with --ctx --model gemini-pro "Based on the context, what is this project about and what does the script do?"
    ```
    - **Expected Result:** The Gemini API should stream a response explaining that the project is "My Project" and that the script prints a hello message. This demonstrates that context injection works seamlessly for remote models as well.

## 4. Conclusion

If all the above steps completed with the expected results, the L1 functionality of Ferri is verified. This plan confirms that the core primitives for project management, configuration, and execution are working correctly and harmoniously. Most importantly, it proves that `ferri with` is a successful unified interface for local commands, local AI, and remote AI.
