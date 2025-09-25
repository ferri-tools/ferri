# User Acceptance Testing: Flow Engine Refactoring (T72-T77)

## 1. Introduction

This document provides a manual testing plan to verify the architectural refactoring of the `ferri flow` and `ferri do` commands. The goal is to ensure that the new implementation, which relies on Ferri's L2 primitives (`run`, `ps`, `yank`), is fully functional, robust, and consistent with the project's design philosophy.

## 2. Prerequisites

1.  **Install the latest binary:** Ensure the most recent version of `ferri` has been compiled and installed.
    ```bash
    cargo install --path ferri-cli --force
    ```

2.  **Set up environment:** Use the `ferri secrets` command to securely store your Gemini API key. The flow engine will automatically inject it into the environment.
    ```bash
    ferri secrets set GEMINI_API_KEY
    # (Follow the prompts to enter your key)
    ```

## 3. Test Cases

### Test Case 1: `ferri flow run` - Core Functionality

-   **Objective:** Verify that a multi-step flow using the new `command:` syntax executes correctly from start to finish.
-   **Setup:**
    1.  Create the necessary directory and a sample Python script for the test:
        ```bash
        mkdir -p pm
        cat > pm/demo_script.py << EOF
        # pm/demo_script.py
        import os

        def old_and_inefficient_function(data):
            # This is a sample script with obvious room for improvement.
            results = []
            for i in data:
                if i % 2 == 0: # Inefficient check
                    results.append(str(i))
            return ",".join(results)

        def another_function():
            # This function is unused.
            pass

        if __name__ == "__main__":
            my_data = range(20)
            print(old_and_inefficient_function(my_data))
        EOF
        ```
-   **Steps:**
    1.  Navigate to the root of the `ferri` project directory.
    2.  Execute the corrected code review flow:
        ```bash
        ferri flow run project_resources/engineering/demos/code_review_flow.corrected.yml
        ```
    3.  While the flow is running, open a **separate terminal window** and monitor the job system:
        ```bash
        watch ferri ps
        ```
        *(Press `Ctrl+C` to exit `watch` when done)*.
    4.  Wait for the flow to complete in the first terminal.
    5.  Verify that the output files have been created:
        ```bash
        ls triage_report.txt enhanced_script.py commit_message.txt
        ```
    6.  Inspect the contents of the generated files to ensure they are not empty and contain reasonable, AI-generated content.
        ```bash
        cat triage_report.txt
        cat enhanced_script.py
        cat commit_message.txt
        ```
-   **Expected Result:**
    -   The `ferri flow run` command displays a TUI showing the steps executing sequentially and completing successfully.
    -   The `ferri ps` command shows the individual jobs being created, running, and transitioning to "Completed".
    -   All three output files (`triage_report.txt`, `enhanced_script.py`, `commit_message.txt`) are created in the root directory.
    -   The content of each file is plausible and relevant to its purpose.

### Test Case 2: `ferri do` - Agentic Flow Generation and Execution

-   **Objective:** Verify that the `ferri do` agent correctly generates a YAML file with the new `command:` syntax and executes it successfully.
-   **Steps:**
    1.  Ensure the `test_app` directory does not already exist.
        ```bash
        rm -rf test_app
        ```
    2.  Run the `ferri do` command with a file system manipulation prompt:
        ```bash
        ferri do "create a new directory called 'test_app' and then create an empty file inside it named 'app.js'"
        ```
    3.  Observe the agent TUI. It should first display the generated YAML flow and then show the live output of the flow's execution.
    4.  Wait for the TUI to show that the flow has completed successfully.
    5.  Verify that the directory and file were created as requested:
        ```bash
        ls -R test_app
        ```
-   **Expected Result:**
    -   The agent TUI appears and shows a valid YAML plan using the `command:` syntax.
    -   The TUI then streams the output of the `ferri flow run` command.
    -   The command completes successfully.
    -   The `test_app` directory exists, and it contains an empty file named `app.js`.

### Test Case 3: Error Handling in `ferri flow run`

-   **Objective:** Verify that the flow runner correctly identifies a failing step, halts execution, and reports the failure.
-   **Setup:**
    1.  Create a new flow file designed to fail:
        ```bash
        cat > failing_flow.yml << EOF
        name: "Test Failure Handling"
        steps:
          - name: "this-step-will-succeed"
            command: "echo 'This is the first step.'"
          - name: "this-step-will-fail"
            command: "ls /non_existent_directory_for_sure"
          - name: "this-step-should-not-run"
            command: "echo 'This should never appear.'"
        EOF
        ```
-   **Steps:**
    1.  Execute the failing flow:
        ```bash
        ferri flow run failing_flow.yml
        ```
    2.  Observe the TUI output.
    3.  After the flow terminates, check the job list:
        ```bash
        ferri ps
        ```
-   **Expected Result:**
    -   The TUI shows "this-step-will-succeed" as "Completed".
    -   The TUI shows "this-step-will-fail" as "Failed" and displays an error message (e.g., "No such file or directory").
    -   The TUI shows "this-step-should-not-run" as "Pending" or does not show it as having started.
    -   The flow terminates without attempting to run the final step.
    -   `ferri ps` shows a job corresponding to the `ls` command with a "Failed" status.

## 4. Cleanup

After all tests are complete, you can remove the generated files:
```bash
rm triage_report.txt enhanced_script.py commit_message.txt failing_flow.yml
rm -rf pm
rm -rf test_app
```
