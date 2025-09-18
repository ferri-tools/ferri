# Manual QA Test Plan: Flow with Local Process Step

This document outlines the manual testing steps for a `ferri flow` that integrates a local shell command (like `grep`) between two model steps.

**Prerequisites:**
1.  `ferri` is installed globally.
2.  Ollama is running with the `gemma:2b` model pulled.
3.  You are in a clean test directory (e.g., `~/ferri-local-process-test`).
4.  The `ferri init` command has been run at the root of the project.

---

### Test Case 1: AI-Generated List Filtered by `grep`

**Goal:** Verify that a flow can correctly pipe the output of a model step into a local process step (`grep`) and then use the output of that process in a subsequent model step.

**Steps:**

1.  **Initialize a project:**
    ```bash
    ferri init
    ```

2.  **Register the Model:**
    *   Create a short alias for the local Gemma model.
    ```bash
    ferri models add gemma --provider ollama --model-name gemma:2b
    ```

3.  **Create the Flow File:**
    *   Create a new file named `local_process_flow.yml` and paste the following YAML into it.

    ```yaml
    name: "AI List Generation and Local Filtering"
    steps:
      - name: "generate-fruit-list"
        model:
          model: "gemma"
          prompt: "Generate a list of 10 common fruits, one fruit per line. Do not add any other text or explanation."
        output: "all_fruits.txt"

      - name: "filter-for-a"
        process: "grep 'a'" # A simple local process step
        input: "all_fruits.txt"
        output: "a_fruits.txt"

      - name: "write-poem"
        model:
          model: "gemma"
          prompt: "You are a cheerful poet. Write a short, funny, four-line poem about the following list of fruits."
        input: "a_fruits.txt"
        output: "poem.txt"
    ```

4.  **Run the Flow:**
    ```bash
    ferri flow run local_process_flow.yml
    ```

5.  **Verify the Outputs:**
    *   Check the contents of the three generated files.
    *   **`all_fruits.txt`**: Should contain a list of about 10 fruits, one per line.
    *   **`a_fruits.txt`**: Should contain a subset of the fruits from the first file, specifically those with the letter 'a' in their name (e.g., Apple, Banana, Orange).
    *   **`poem.txt`**: Should contain a short poem about the fruits listed in `a_fruits.txt`.
    ```bash
    cat all_fruits.txt
    cat a_fruits.txt
    cat poem.txt
    ```

This confirms that the flow can successfully integrate local shell commands.
