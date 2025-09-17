# Manual QA Test Plan: Hybrid Model Workflow

This document outlines the manual testing steps for a `ferri flow` that uses both a local (Ollama) and a remote (Google Gemini) model.

**Prerequisites:**
1.  `ferri` is installed globally.
2.  Ollama is running with the `gemma:2b` model pulled.
3.  You are in a clean test directory (e.g., `~/ferri-hybrid-test`).
4.  The `ferri init` command has been run at the root of the project.
5.  You have a Google AI API key.

---

### Test Case 1: Hybrid Code Review Flow

**Goal:** Verify that a flow can successfully delegate tasks between a local and a remote model, using secrets for authentication.

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
    *   Create a short alias for the local Gemma model.
    *   Register the remote Gemini Pro model, linking it to the secret.
    ```bash
    ferri models add gemma --provider ollama --model-name gemma:2b
    ferri models add gemini-pro --provider google --api-key-secret GOOGLE_API_KEY --model-name gemini-pro
    ```

4.  **Create the Demo Python Script:**
    *   This script has intentional flaws for the models to find.
    ```bash
    cat << 'EOF' > demo_script.py
"""
A simple Flask application to demonstrate Ferri's code review capabilities.
This script contains intentional flaws for the demo.
"""
from flask import Flask, request
import os

app = Flask(__name__)

@app.route('/')
def index():
    name = request.args.get('name', 'World')
    return "Hello, " + name + "!"

# Flaw: This endpoint is vulnerable to command injection
@app.route('/files')
def list_files():
    directory = request.args.get('dir', '.')
    # This is dangerous!
    file_list = os.popen("ls " + directory).read()
    return f"<pre>{file_list}</pre>"

if __name__ == '__main__':
    app.run(debug=True)
EOF
    ```

5.  **Create the Hybrid Flow File:**
    ```bash
    cat << 'EOF' > code_review_flow.yml
name: "AI-Powered Code Review & Enhancement"
steps:
  - name: "triage-code"
    model:
      model: "gemma" # Local model for speed
      prompt: "You are a code triager. Summarize the following Python script, identify any obvious flaws or style issues, and create a checklist for a senior developer to perform a deeper review. Focus on security and performance."
    input: "demo_script.py"
    output: "triage_report.txt"

  - name: "expert-review-and-enhance"
    model:
      model: "gemini-pro" # Remote model for power
      prompt: "You are an expert Python developer. Using the original code and the triage report, perform a deep code review based on the checklist. Then, generate an enhanced, production-ready version of the script that fixes all identified issues."
    input: "triage_report.txt" # Conceptually also uses the original script
    output: "enhanced_script.py"

  - name: "generate-commit-message"
    model:
      model: "gemma" # Local model for speed
      prompt: "You are a git expert. Based on the enhanced code, write a concise and conventional commit message."
    input: "enhanced_script.py"
    output: "commit_message.txt"
EOF
    ```

6.  **Run the Flow:**
    ```bash
    ferri flow run code_review_flow.yml
    ```

7.  **Verify the Outputs:**
    *   Check the contents of the three generated files.
    *   **`triage_report.txt`**: Should contain a summary and a checklist from Gemma.
    *   **`enhanced_script.py`**: Should contain a refactored, more secure version of the Python script from Gemini Pro (e.g., using `subprocess.run` instead of `os.popen`).
    *   **`commit_message.txt`**: Should contain a git commit message from Gemma.
    ```bash
    cat triage_report.txt
    cat enhanced_script.py
    cat commit_message.txt
    ```

This confirms that the hybrid workflow is executing correctly.
