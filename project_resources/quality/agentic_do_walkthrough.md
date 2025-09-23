# Walkthrough: `ferri do` Agentic Command

This document provides a step-by-step guide on how to set up and use the new L3 agentic command, `ferri do`.

## 1. Setup: Configure Your Gemini API Key

The `ferri do` command uses the Google Gemini 1.5 Pro model to generate its plans. Before you can use it, you must securely store your Google API key.

### Steps

1.  **Get your API Key:** Obtain your API key from Google AI Studio.
2.  **Initialize Ferri (if you haven't already):**
    ```bash
    ferri init
    ```
3.  **Store the Secret:** Use the `ferri secrets set` command to securely store your key. Ferri will encrypt and save it in your project's local `.ferri/secrets.json` file.
    ```bash
    ferri secrets set GEMINI_API_KEY "your-api-key-here"
    ```
    > **Note:** The secret **must** be named `GEMINI_API_KEY` for the agent to find it.

## 2. Usage: Executing a High-Level Goal

The `ferri do` command takes a natural language prompt (in quotes) that describes your high-level goal. The agent will then break this down into a multi-step `flow.yml` and execute it.

### Example

Let's ask the agent to create a new Rust project that prints "hello world".

```bash
ferri do "create a new rust project named 'helloworld' and then run it to confirm it works"
```

## 3. Expected Output

When you run the command, you will see the agent's real-time thought process, the generated plan, and the output from the execution of that plan.

The output should look similar to this:

```text
[AGENT] Generating flow for prompt: 'create a new rust project named 'helloworld' and then run it to confirm it works'
[AGENT] Sending request to Gemini API...
[AGENT] Received response from Gemini API.

--- Generated Flow ---
name: "Create and run a new Rust project"
jobs:
  - id: "create_project"
    command: "cargo new helloworld"
  - id: "run_project"
    command: "cd helloworld && cargo run"
    dependencies:
      - "create_project"
----------------------

--- Starting flow: Create and run a new Rust project ---

--- Step 'create_project': Starting ---
     Created binary (application) `helloworld` package

--- Step 'create_project': Completed ---

--- Step 'run_project': Starting ---
   Compiling helloworld v0.1.0 (/Users/jorgeajimenez/repos/ferri/helloworld)
    Finished `dev` [unoptimized + debuginfo] target(s) in 0.50s
     Running `target/debug/helloworld`
Hello, world!

--- Step 'run_project': Completed ---
```

This demonstrates that the agent successfully:
1.  Understood the goal.
2.  Generated a correct, two-step plan.
3.  Executed the plan, creating the project and running it.
4.  Streamed the output of the commands to your terminal.
