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
