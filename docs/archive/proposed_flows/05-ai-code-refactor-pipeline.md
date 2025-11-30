# 05: Complex AI Code Refactor Pipeline

This flow represents the culmination of the previous concepts, demonstrating a complex, multi-step AI pipeline that mimics a realistic, high-value developer workflow. It chains multiple AI and shell commands together to triage, refactor, test, and document a piece of code automatically.

## How It Works

1.  **`triage-code` Job:** A fast, local model (`gemma`) performs an initial, high-level review of a sample Python script. This is a cost-effective way to get a quick analysis. The output is a `triage_report.txt`.

2.  **`refactor-code` Job:** A powerful, remote model (`gemini-pro`) takes over. It uses the *original script* and the *triage report* as its context to perform a deep refactoring, saving the improved code to `enhanced_script.py`. This shows an AI building upon the work of another AI.

3.  **`generate-tests` Job:** This job runs in parallel with the refactoring. It uses the *original script* to generate a set of basic unit tests, saving them to `tests.py`. This demonstrates planning ahead, as the tests will be needed later.

4.  **`validate-and-document` Job:** This is the final merge and validation job. It depends on both the refactored code and the generated tests.
    -   It first runs the generated tests against the *new, enhanced script*. This is a critical validation step.
    -   Assuming the tests pass, it then uses the local `gemma` model to generate a conventional commit message that summarizes all the changes.

## State Management

-   **Multi-Artifact Pipeline:** This flow manages multiple intermediate artifacts (`triage_report.txt`, `enhanced_script.py`, `tests.py`) and passes them between jobs.
-   **Context Composition:** Jobs like `refactor-code` and `validate-and-document` demonstrate context composition, where multiple files are added to the context to give the AI a complete picture of the task.
-   **Implicit Validation:** The `python3 tests.py` step serves as an implicit validation gate. If the tests fail, the job will fail, and the flow will stop before generating the commit message, preventing a bad change from being documented.

## How to Run

This flow uses both a remote model (`gemini-pro`) and a local model (`gemma`).

### Prerequisites

1.  **Google API Key:** You need a Google API key with the Gemini API enabled.
2.  **Install and run Ollama:** [https://ollama.com/](https://ollama.com/)
3.  **Pull the gemma model:** `ollama pull gemma:2b`

### Execution

The following commands are fully self-contained. They will create a temporary workspace, configure the necessary secrets and models, and then run the flow.

```bash
# 1. Create a temporary directory and navigate into it.
mkdir -p /tmp/flow-tests/05-ai-refactor && cd /tmp/flow-tests/05-ai-refactor

# 2. Initialize a new ferri workspace.
ferri init

# 3. Set the required secret. Replace "YOUR_KEY" with your actual Google API key.
ferri secrets set GOOGLE_API_KEY "YOUR_KEY"

# 4. Add the required models to the workspace's registry.
ferri models add gemma --provider ollama --model-name gemma:2b
ferri models add gemini-pro \
  --provider google \
  --api-key-secret GOOGLE_API_KEY \
  --model-name gemini-2.5-pro

# 5. Create the flow YAML file in the current directory.
cat <<'EOF' > 05-ai-code-refactor-pipeline.yml
# This flow demonstrates a full AI-powered code review, refactoring, and documentation pipeline.
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: ai-automated-code-improvement-pipeline
spec:
  jobs:
    triage-code:
      name: "Triage Original Code"
      steps:
        - name: "Create a sample Python script"
          run: |
            echo '
            # Simple script to process numbers
            def process_numbers(data):
                results = []
                for i in data:
                    if i % 2 == 0:
                        results.append(str(i))
                return ",".join(results)
            ' > original_script.py
        - name: "Add script to context"
          run: "ferri ctx add original_script.py"
        - name: "Use local model for a quick triage"
          run: "ferri with --ctx --model gemma --output triage_report.txt -- 'Briefly review this Python code. Identify one area for improvement.'"

    refactor-code:
      name: "Refactor Code with Powerful AI"
      needs:
        - triage-code
      steps:
        - name: "Add original script and triage report to context"
          run: "ferri ctx add original_script.py triage_report.txt"
        - name: "Use Gemini to refactor the code based on the triage"
          run: "ferri with --ctx --model gemini-pro --output enhanced_script.py -- 'Rewrite the Python script from the context to implement the improvement suggested in the triage report. CRITICAL: Your response must be ONLY the raw, runnable code. Do not include markdown fences like ```python or any other explanatory text.'"

    generate-tests:
      name: "Generate Unit Tests"
      needs:
        - triage-code
      steps:
        - name: "Add script to context"
          run: "ferri ctx add original_script.py"
        - name: "Use Gemini to write unit tests for the original script's logic"
          run: "ferri with --ctx --model gemini-pro --output tests.py -- 'Write a Python unit test file using the unittest library to verify the logic of the script in context. The tests must be runnable with python3. CRITICAL: Your response must be ONLY the raw, runnable code. Do not include markdown fences like ```python or any other explanatory text.'"

    validate-and-document:
      name: "Validate and Generate Commit Message"
      needs:
        - refactor-code
        - generate-tests
      steps:
        - name: "Run the generated tests against the ENHANCED script"
          # This step is a crucial validation gate.
          run: "python3 tests.py"
        - name: "Add the final code to the context"
          run: "ferri ctx add enhanced_script.py"
        - name: "Generate a conventional commit message"
          run: "ferri with --ctx --model gemma --output commit_message.txt -- 'Based on the Python script in context, write a conventional commit message for the refactoring that was performed.'"
EOF

# 6. Run the flow.
ferri flow run 05-ai-code-refactor-pipeline.yml
```