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

## How to Run

This flow uses the `gemma` model via Ollama.

### Prerequisites

1.  **Install and run Ollama:** [https://ollama.com/](https://ollama.com/)
2.  **Pull the gemma model:** `ollama pull gemma:2b`

### Execution

The following commands are fully self-contained. They will create a temporary workspace, configure the necessary models, and then run the flow.

```bash
# 1. Create a temporary directory and navigate into it.
mkdir -p /tmp/flow-tests/03-conditional-flow && cd /tmp/flow-tests/03-conditional-flow

# 2. Initialize a new ferri workspace.
ferri init

# 3. Add the required model to the workspace's registry.
ferri models add gemma --provider ollama --model-name gemma:2b

# 4. Create the flow YAML file in the current directory.
cat <<'EOF' > 03-simulated-conditional-flow.yml
# This flow simulates conditional logic. It runs a deep security scan only if an initial AI triage finds a potential vulnerability.
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: ai-conditional-security-scan
spec:
  jobs:
    triage-code:
      name: "Perform AI Security Triage"
      steps:
        - name: "Create a sample Python script with a potential vulnerability"
          run: |
            echo 'import os
            def get_user_data(user_input):
                # This is a potential command injection vulnerability
                os.system(f"echo User data: {user_input}")
            ' > sample_script.py
        - name: "Add script to context"
          run: "ferri ctx add sample_script.py"
        - name: "Ask Gemma to check for vulnerabilities"
          run: "ferri with --ctx --model gemma --output triage_report.txt -- 'Analyze this Python script for security vulnerabilities. If you find any security issues, start your response with the word VULNERABLE:. Otherwise, start with SECURE:.'"

    check-for-vulnerability:
      name: "Check Triage Report"
      needs:
        - triage-code
      steps:
        - name: "Create a flag file if vulnerability is found"
          # This step uses grep to check the report. If 'VULNERABLE' is found, it creates a flag file.
          # The '|| true' ensures the step succeeds even if grep doesn't find the word.
          run: "(grep -q 'VULNERABLE:' triage_report.txt && touch is_vulnerable.flag) || true"

    run-security-scan:
      name: "Run Deep Security Scan (If Needed)"
      needs:
        - check-for-vulnerability
      steps:
        - name: "Run a simulated deep scan if the flag file exists"
          # This step uses a shell 'if' to check for the flag file's existence before running.
          run: |
            if [ -f is_vulnerable.flag ]; then
              echo "Vulnerability detected! Running deep security scan..."
              # In a real-world scenario, this would be a call to a tool like 'semgrep' or 'snyk'.
              sleep 5 
              echo "Deep scan complete."
            else
              echo "No vulnerabilities found in initial triage. Skipping deep scan."
            fi
EOF

# 5. Run the flow.
ferri flow run 03-simulated-conditional-flow.yml
```