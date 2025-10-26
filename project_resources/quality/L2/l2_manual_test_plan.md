# Ferri Layer 2 (L2) Manual Test Plan: The Asynchronous AI Engine

## 1. Introduction

This document provides a manual testing plan for the Layer 2 (L2) functionality of the Ferri toolkit, framed through its primary purpose: **to serve as a robust, asynchronous AI engine.**

The goal is to verify that `ferri run`, `ps`, and `yank` work together to reliably execute, monitor, and retrieve the results of long-running, complex AI tasks. This plan moves beyond simple shell commands to focus on the core AI-centric workflows that are central to Ferri's design.

We will test the full lifecycle for:
1.  **Long-Form Creative AI Generation:** Running a creative writing task that takes significant time.
2.  **Context-Aware Code Refactoring:** Simulating a background code analysis and modification job.
3.  **Multi-Step AI Pipelines (via Flow Files):** Orchestrating a sequence of dependent AI tasks.

## 2. Prerequisites

Before starting, ensure you have the following:
- `ferri` compiled and available in your system's `PATH`.
- `ollama` installed and running with a creative model pulled (e.g., `ollama pull gemma:2b`).
- A Google AI API key for testing powerful remote models.
- A clean directory for a new test project.

## 3. Test Execution

### Part 1: Project and Model Setup

This part ensures the environment is correctly configured for both local and remote AI execution.

1.  **Create and enter a test directory:**
    ```
    mkdir ferri-ai-engine-test
    cd ferri-ai-engine-test
    ```

2.  **Initialize the project:**
    ```
    ferri init
    ```
    - **Expected Result:** `✨ Successfully initialized Ferri project in ./.ferri`.

3.  **Set the Google API Key:**
    *Replace `YOUR_GOOGLE_API_KEY` with your actual key.*
    ```
    ferri secrets set GOOGLE_API_KEY "YOUR_GOOGLE_API_KEY"
    ```

4.  **Register a local Ollama model for creative tasks:**
    ```
    ferri models add gemma --provider ollama --model-name gemma:2b
    ```

5.  **Register a remote Google model for complex reasoning:**
    ```
    ferri models add gemini-pro \
      --provider google \
      --api-key-secret GOOGLE_API_KEY \
      --model-name gemini-2.5-pro
    ```

### Part 2: Test Case - Long-Running Creative AI Job

This test verifies the most fundamental L2 use case: offloading a time-consuming generative AI task to the background.

1.  **Run the AI job:**
    *This command asks the local model to write a lengthy story, simulating a real creative task.*
    ```
    ferri run --model gemma -- "Write a 500-word sci-fi story about a cat who discovers a hidden portal in its litter box."
    ```
    - **Expected Result:** The command immediately returns a `run_id`. Note this ID.

2.  **Monitor the job's progress:**
    ```
    ferri ps
    ```
    - **Expected Result:** The job is listed with a `Running` status. After some time (depending on your hardware), it should transition to `Succeeded`.

3.  **Yank the completed story:**
    *Once the job is `Succeeded`, retrieve the output.*
    ```
    ferri yank <your_run_id>
    ```
    - **Expected Result:** The terminal prints the full, multi-paragraph sci-fi story. This confirms that long-form generation, execution, and retrieval are working correctly.

### Part 3: Test Case - Asynchronous, Context-Aware Code Refactoring

This test simulates a developer workflow: asking an AI to refactor code in the background while continuing to work.

1.  **Create a sample Python script:**
    ```python
    cat > old_script.py << EOF
    # old_script.py
    def process_data(data):
        results = []
        for i in data:
            if i % 2 == 0:
                results.append(str(i))
        return ",".join(results)
    EOF
    ```

2.  **Add the script to the context:**
    ```
    ferri ctx add old_script.py
    ```

3.  **Run the background refactoring job:**
    *This asks the powerful remote model to analyze the context and rewrite the script.*
    ```
    ferri run --ctx --model gemini-pro -- "Rewrite the Python script in the context to be more idiomatic and efficient. Respond only with the raw code."
    ```
    - **Expected Result:** The command returns a new `run_id`.

4.  **Monitor and yank the result:**
    *Use `ferri ps` to wait for the job to complete.*
    ```
    ferri yank <your_refactor_run_id> > new_script.py
    ```
    - **Expected Result:** The `new_script.py` file is created and contains a refactored version of the original script (e.g., using a list comprehension). This verifies that context is correctly passed to background AI jobs.

### Part 4: Test Case - Multi-Step AI Pipeline (Flow File)

This test verifies the orchestration of a sequence of dependent AI jobs, the foundation for complex automation.

1.  **Create a flow file (`review_flow.yml`):**
    *This flow first generates a code review, then writes a commit message based on that review.*
    ```yaml
    name: AI Code Review Pipeline
    jobs:
      - name: generate-review
        steps:
          - uses: local
            with:
              command: "ferri with --ctx --model gemini-pro 'Review the code in old_script.py and save the output.' > review.txt"

      - name: write-commit-message
        steps:
          - uses: local
            with:
              # This job uses the output of the first job as its context
              command: "ferri with --ctx review.txt --model gemma 'Write a conventional commit message based on this code review.' > commit_message.txt"
    ```

2.  **Run the entire flow:**
    ```
    ferri run review_flow.yml
    ```
    - **Expected Result:** The command returns a `run_id` for the flow.

3.  **Monitor the flow's jobs:**
    ```
    ferri ps <your_flow_run_id>
    ```
    - **Expected Result:** A detailed view shows both `generate-review` and `write-commit-message` jobs transitioning to `Succeeded`.

4.  **Yank the final artifact:**
    *We only need to verify the final output of the pipeline.*
    ```
    ferri yank <your_flow_run_id> write-commit-message
    ```
    - **Expected Result:** A `write-commit-message` directory is created containing `commit_message.txt`. The file should contain an AI-generated commit message (e.g., `feat: Refactor data processing logic`).

## 4. Conclusion

If all the above test cases are successful, the L2 functionality of Ferri is verified as a capable asynchronous AI engine. This plan confirms that Ferri can reliably manage and orchestrate long-running, context-aware, and multi-step AI jobs, which is the core of its Layer 2 architecture.
