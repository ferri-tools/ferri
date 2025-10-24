# 03: Simulated Conditional Execution Flow

The Ferri flow schema does not have native `if/else` constructs. However, this flow demonstrates how to **simulate conditional logic** using standard shell command features and intermediate "flag" files. This is a powerful pattern for creating workflows that can react to the output of previous steps.

## How It Works

1.  **`triage-code` Job:** This job uses `ferri with` to ask an AI to perform a quick security triage on a sample Python script. The key part of the prompt is the instruction: *"If you find any security issues, start your response with the word 'VULNERABLE:'."* The output is saved to `triage_report.txt`.

2.  **`check-for-vulnerability` Job:** This job checks the triage report.
    -   Its first step uses `grep`. If `grep` finds the word "VULNERABLE", the command succeeds, and the `&&` operator allows the second part to run: `touch is_vulnerable.flag`. This creates an empty "flag" file.
    -   If `grep` does *not* find the word, it exits with an error code, and the `touch` command is never executed. The `|| true` ensures that the step itself doesn't fail the entire job, allowing the flow to continue.

3.  **`run-security-scan` Job:** This job depends on the check. Its step uses a shell `if` statement: `[ -f is_vulnerable.flag ] && ...`. This command translates to: "If the flag file exists, then run a deep security scan." If the flag file was never created, this step does nothing.

## State Management

-   **Flag Files:** The state is passed not through the *content* of a file, but through the *existence* of one. The `is_vulnerable.flag` file acts as a boolean signal between the `check-for-vulnerability` and `run-security-scan` jobs.
-   **Shell Logic:** This example shows that you don't need complex schema features to build sophisticated logic. By leveraging the power of the shell (`&&`, `||`, `if`), you can create robust conditional workflows.
