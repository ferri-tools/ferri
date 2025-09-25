# User Acceptance Testing: Flow Engine Refactoring (T72-T77)

## 1. Introduction

This document provides a manual, **fully self-contained** testing plan to verify the architectural refactoring of the `ferri flow` and `ferri do` commands. The tests are designed to be run from any directory, simulating a real user's workflow in a clean environment.

## 2. Test Setup: Create a New Project

First, we will create a new, temporary directory for our test project and navigate into it. This ensures our tests do not interfere with any other projects.

```bash
mkdir ferri_uat_project
cd ferri_uat_project
```

## 3. Test Cases

### Test Case 1: `ferri flow run` - Core Functionality

-   **Objective:** Verify that a multi-step flow can be created and executed from scratch in a new project, correctly using the job management system.

-   **Steps:**
    1.  **Initialize the project:**
        ```bash
        ferri init
        ```

    2.  **Set the required secret:**
        ```bash
        ferri secrets set GEMINI_API_KEY
        # (Follow the prompts to enter your key)
        ```

    3.  **Register the Model:**
        ```bash
        ferri models add gemini-pro \
          --provider google \
          --api-key-secret GEMINI_API_KEY \
          --model-name gemini-1.5-pro-latest
        ```

    4.  **Create a sample script to be reviewed:**
        ```bash
        cat > demo_script.py << EOF
        # demo_script.py
        def old_function(data):
            results = []
            for i in data:
                if i % 2 == 0:
                    results.append(str(i))
            return ",".join(results)
        print(old_function(range(10)))
        EOF
        ```

    4.  **Create the workflow file using the new `command` syntax:**
        ```bash
        cat > code_review_flow.yml << EOF
        name: "Self-Contained Code Review"
        steps:
          - name: "triage-code"
            input: "demo_script.py"
            command: "with --model gemini-pro --output triage_report.txt \"Summarize this Python script and identify 3 potential improvements.\""

          - name: "enhance-code"
            input: "demo_script.py,triage_report.txt"
            command: "with --ctx --model gemini-pro --output enhanced_script.py \"Based on the triage report, rewrite the original script to implement the suggested improvements. Your response must contain only the raw, runnable code, without any markdown formatting or explanatory text.\""

          - name: "generate-commit-message"
            input: "enhanced_script.py"
            command: "with --ctx --model gemini-pro --output commit_message.txt \"Write a conventional commit message for the changes in the enhanced script.\""
        EOF
        ```

    5.  **Execute the flow:**
        ```bash
        ferri flow run code_review_flow.yml
        ```

    6.  **Monitor the jobs (in a separate terminal):** While the flow is running, open a new terminal window, navigate to the `ferri_uat_project` directory, and run the following command to poll the job status:
        ```bash
        while true; do clear; ferri ps; sleep 2; done
        # Press Ctrl+C to exit this monitoring loop when the flow is complete.
        ```

    7.  **Verify the results:** After the flow completes, check for the output files:
        ```bash
        ls triage_report.txt enhanced_script.py commit_message.txt
        ```
        Then, inspect their contents:
        ```bash
        cat triage_report.txt enhanced_script.py commit_message.txt
        ```

-   **Expected Result:**
    -   The `ferri flow run` command displays a TUI showing the steps completing successfully.
    -   `ferri ps` shows the jobs being created and transitioning to "Completed".
    -   All three output files are created and contain relevant, AI-generated content.

### Test Case 2: `ferri do` - Agentic Execution

-   **Objective:** Verify that `ferri do` can generate and execute a plan in the clean project environment.

-   **Steps:**
    1.  **Run the `ferri do` command:**
        ```bash
        ferri do "create a subdirectory named 'docs' and add a file called 'README.md' inside it with the content '# My Project'"
        ```
    2.  **Observe the TUI:** The agent should display its plan and then the live output of the execution.
    3.  **Verify the result:** After the agent finishes, check the file system:
        ```bash
        ls docs/README.md
        cat docs/README.md
        ```

-   **Expected Result:**
    -   The agent TUI shows a valid YAML plan using the `command:` syntax.
    -   The `docs` directory and the `README.md` file are created.
    -   The content of `README.md` is `# My Project`.

## 4. Cleanup

After all tests are complete, navigate out of the test directory and remove it.

```bash
cd ..
rm -rf ferri_uat_project
```