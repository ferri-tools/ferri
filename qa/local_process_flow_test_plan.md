# Manual QA Test Plan: Hybrid Flow with Local Processing

This document outlines the manual testing steps for a `ferri flow` that uses a remote model, a local model, and multiple local shell commands (`grep` and `sed`).

**Prerequisites:**
1.  `ferri` is installed globally.
2.  Ollama is running with the `gemma:2b` model pulled.
3.  You are in a clean test directory (e.g., `~/ferri-hybrid-process-test`).
4.  The `ferri init` command has been run at the root of the project.
5.  You have a Google AI API key.

---

### Test Case 1: AI Quote Generation, Local Filtering, and Summarization

**Goal:** Verify that a flow can chain a remote model, `grep`, `sed`, and a local model, passing the output of each step as input to the next.

**Steps:**

1.  **Initialize a project:**
    ```bash
    ferri init
    ```

2.  **Set the Google API Key Secret:**
    *   Replace `"your-api-key-here"` with your actual key.
    ```bash
    ferri secrets set GOOGLE_API_KEY "your-api-key-here"
    ```

3.  **Register the Models:**
    ```bash
    ferri models add gemma --provider ollama --model-name gemma:2b
    ferri models add gemini-pro --provider google --api-key-secret GOOGLE_API_KEY --model-name gemini-1.5-pro-latest
    ```

4.  **Create the Flow File:**
    *   Create a new file named `hybrid_process_flow.yml` and paste the following YAML into it.

    ```yaml
    name: "Hybrid AI Quote Processing"
    steps:
      - name: "generate-quotes"
        model:
          model: "gemini-pro" # Use remote model for a high-quality list
          prompt: "Generate a list of 5 famous quotes about love. Each quote should be on a new line."
        output: "all_quotes.txt"

      - name: "filter-for-love"
        process: "grep -i 'love'" # Filter for lines containing 'love'
        input: "all_quotes.txt"
        output: "love_quotes.txt"

      - name: "replace-love-with-code"
        process: "sed 's/love/code/gi'" # Replace 'love' with 'code' (case-insensitive)
        input: "love_quotes.txt"
        output: "code_quotes.txt"

      - name: "summarize-quotes"
        model:
          model: "gemma" # Use local model for a quick summary
          prompt: "You are a tech philosopher. Briefly summarize the meaning of the following quotes in a single sentence."
        input: "code_quotes.txt"
        output: "summary.txt"
    ```

5.  **Run the Flow:**
    ```bash
    ferri flow run hybrid_process_flow.yml
    ```

6.  **Verify the Outputs:**
    *   Check the contents of the four generated files.
    *   **`all_quotes.txt`**: Should contain 5 quotes about love.
    *   **`love_quotes.txt`**: Should contain the same quotes as the first file.
    *   **`code_quotes.txt`**: Should contain the same quotes, but with every instance of "love" replaced with "code".
    *   **`summary.txt`**: Should contain a philosophical summary of the modified "code" quotes.
    ```bash
    cat all_quotes.txt
    cat love_quotes.txt
    cat code_quotes.txt
    cat summary.txt
    ```

This confirms that the flow can successfully chain remote models, local processes, and local models.