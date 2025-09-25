> ---
>
> Join our community!

Have questions, ideas, or just want to chat with other users and developers? Join our Discord server!

Join the Conversation at https://discord.gg/H8qXs9gx

> ### **Notice: Pre-Release Alpha Software**
>
> This repository contains an early, pre-release version of Ferri and should be considered **alpha-quality software**.
>
> * **Stability:** The software is not stable and is not suitable for production use.
> * **Breaking Changes:** The API, command structure, and workflow schemas are under active development and are subject to change without notice in future updates.
> * **Feedback:** This version is released for evaluation and feedback purposes. Please report any bugs or suggestions by opening an issue or contacting us at **[me@jorgeajimenez.com](mailto:me@jorgeajimenez.com)** or **[me@gretchenboria.com](mailto:me@gretchenboria.com)**.
>
> ---

# Ferri

<div align="center">
  <img src="logo.png" alt="Ferri Logo" width="200"/>
</div>

Ferri is a local-first AI toolkit that acts as an intelligent director for foundation models. It evolves from a simple command runner into a proactive, agentic partner that can plan and execute complex development tasks.

Ferri creates secure, project-based environments with portable context, unifying your workflow across local (Ollama) and remote (API) models. The goal is to let you focus on your high-level goals, not on the minutiae of context management and command execution.

## Core Concepts

*   **Project-Based:** Ferri operates within your project directory, keeping all context, secrets, and job history in a local `.ferri` directory.
*   **Context-Aware:** You tell Ferri what files are important (`ferri ctx add`), and it automatically injects them into your AI prompts.
*   **Secure:** Secrets like API keys are encrypted and automatically made available to commands, so you never have to expose them in your shell.
*   **Layered Architecture:** Start with simple commands and gradually adopt more powerful, automated, and agentic features as you need them.

## The Ferri Architecture

Ferri is built in layers, allowing you to choose the right level of power for your task.

| Layer | Command(s) | Description |
|---|---|---|
| **L1: Core Execution** | `init`, `secrets`, `models`, `ctx`, `with` | The foundation. Manages your environment, models, and executes synchronous, single-shot commands. |
| **L2: Workflow Automation** | `run`, `ps`, `yank`, `flow` | The automation layer. Runs commands as background jobs, monitors their status, and orchestrates multi-step workflows. |
| **L3: Agentic Engine** | `do` | The intelligent director. Takes a high-level goal, formulates a multi-step plan, and executes it. |

---

## Command Reference

| Command | Description |
|---|---|
| `init` | Initialize a new Ferri project in the current directory. |
| `secrets` | Manage encrypted, project-specific secrets like API keys. |
| `models` | Manage the registry of local (Ollama) and remote (API) models. |
| `ctx` | Manage the project's context from files or job outputs. |
| `with` | Execute a command within a context-aware, synchronous environment. |
| `run` | Run a command as a long-running background job. |
| `ps` | List and manage active background jobs. |
| `yank` | Fetch the output (stdout) of a completed background job. |
| `flow` | Define and run multi-step, declarative AI workflows from a file. |
| `do` | Execute a high-level goal with an AI-powered agentic engine. |

---

## Use Cases & Examples

This section provides a command-by-command breakdown of Ferri's capabilities, each with practical, real-world examples.

### `ferri init`

Initializes a new Ferri project. This creates the `.ferri` directory where all project-specific state, context, secrets, and configurations are stored.

**Use Case: Starting a New Project**
```bash
# Navigate to your project's root directory
cd my-new-app

# Initialize Ferri
ferri init
# Output: ✨ Successfully initialized Ferri project in ./.ferri
```

---

### `ferri secrets`

Manages encrypted, project-specific secrets like API keys. Secrets are stored locally and are never committed to source control.

**Use Case: Storing API Keys for Remote Models**
```bash
# Securely store your OpenAI API key
ferri secrets set OPENAI_API_KEY="sk-..."

# Securely store your Google API key
ferri secrets set GOOGLE_API_KEY "AIza..."
```

---

### `ferri models`

Manages the registry of available models. Ferri features a unified model system that automatically discovers local Ollama models and allows you to register remote, API-based models.

**Use Case: Listing All Available Models**
The `ls` command shows both auto-discovered Ollama models and any remote models you have explicitly registered.
```bash
ferri models ls
# Output:
# ALIAS             PROVIDER    ID/NAME
# llama3            ollama      llama3:latest (discovered)
# gemma:2b          ollama      gemma:2b (discovered)
# gpt-4o            openai      gpt-4o
# gemini-pro        google      gemini-1.5-pro
```

**Use Case: Registering a New Remote Model**
To use a remote model, you must first store its API key using `ferri secrets`, then register the model.
```bash
# Step 1: Store the secret
ferri secrets set ANTHROPIC_API_KEY "sk-ant..."

# Step 2: Register the model, creating an alias ('claude-opus')
# The '--api-key-secret' flag references the NAME of the secret, not the key itself.
ferri models add claude-opus \
  --provider anthropic \
  --api-key-secret ANTHROPIC_API_KEY \
  --model-name claude-3-opus-20240229
```

**Use Case: Creating a Shorter Alias for a Local Model**
```bash
# Create a simple alias 'gemma' for the longer, discovered model name
ferri models add gemma --provider ollama --model-name gemma:2b
```

---

### `ferri ctx`

Manages the project's context (the files and data provided to the AI).

**Use Case: Adding Files and Directories to the Context**
```bash
# Add the entire source directory and the main README
ferri ctx add ./src README.md
```

---

### `ferri with`

Executes a command, injecting secrets and context. This is Ferri's core execution engine.

**Use Case: Switching Seamlessly Between Local and Remote Models**
Once models are registered, you can switch between them using the `--model` flag.
```bash
# First, add some code to the context
ferri ctx add ./src

# Run a query with a fast, local model
ferri with --ctx --model llama3 "Explain the purpose of the main function."

# Run the exact same query with a powerful, remote model for a different perspective
ferri with --ctx --model gpt-4o "Explain the purpose of the main function."

# Run it again with your registered Gemini Pro model
ferri with --ctx --model gemini-pro "Explain the purpose of the main function."
```

**Use Case: Securely Running Scripts**
Any script or tool can be run with `ferri with` to gain access to stored secrets as environment variables.
```bash
# Your python script can now access the API key via os.getenv("OPENAI_API_KEY")
ferri with -- python ./scripts/deploy.py
```

---

### `ferri run`, `ps`, and `yank`

Manages long-running, asynchronous background jobs. The `run` command shares the exact same syntax as `with` for model and context flags.

**Use Case: Generating Documentation in the Background**
```bash
# Step 1: Start a long-running job with 'ferri run'
ferri run --ctx --model gpt-4o "Generate a complete project summary in Markdown"
# Output: Job submitted: job-b4c5d6

# Step 2: Check the status with 'ferri ps'
ferri ps
# Output:
# JOB ID      STATUS      COMMAND
# job-b4c5d6  COMPLETED   ollama run gpt-4o...

# Step 3: Retrieve the output with 'ferri yank'
ferri yank job-b4c5d6 > PROJECT_SUMMARY.md
```

---

### `ferri flow`

Defines and runs multi-step, declarative AI workflows from a YAML file.

**Use Case: Automating Code Generation and Testing**
```yaml
# ci-prep.yml
name: "Prepare for CI"
jobs:
  - id: generate-docs
    command: 'ferri with --ctx --model gpt-4o "Generate docs for the codebase" > DOCS.md'
  - id: write-tests
    dependencies: [generate-docs]
    command: 'ferri with --ctx DOCS.md --model gpt-4o "Write unit tests" > main.test.js'
```
```bash
# Execute the entire workflow
ferri flow run ci-prep.yml
```

---

### `ferri do`

Executes a high-level goal with an AI-powered agentic engine.

**Use Case: Agentic Code Modification**
```bash
ferri do "Add a new '/api/users' endpoint to my Express app. It needs a route, a controller with a placeholder, and must be registered in the main app file."
```

---

## Command Modifiers

For a detailed list of all command modifiers and advanced options (e.g., `--stream`, `--dry-run`, `--interactive`), see [COMMAND_MODIFIERS.md](./COMMAND_MODIFIERS.md).

---

## Demo: AI-Powered Code Review

This demo showcases a workflow where a fast, local model (Gemma) and a powerful, remote model (Gemini Pro) work together to perform a code review.

**1. Setup:**

First, add your Google API key to Ferri's secrets and register the Gemini Pro model.

```bash
# Store your API key
ferri secrets set GOOGLE_API_KEY "your-api-key-here"

# Register the Gemini Pro model
ferri models add gemini-pro \
  --provider google \
  --api-key-secret GOOGLE_API_KEY \
  --model-name gemini-pro
```

**2. Run the Flow:**

Execute the `code_review_flow.yml` pipeline. This flow uses the `engineering/demos/demo_script.py` file as its input.

```bash
ferri flow run project_resources/engineering/demos/code_review_flow.yml
```

**3. What it Does:**

*   **Step 1 (Local - Gemma):** Performs a quick "triage" on `demo_script.py` and writes a summary to `triage_report.txt`.
*   **Step 2 (Remote - Gemini Pro):** Uses the triage report to perform a deep analysis and writes an enhanced, secure version of the script to `enhanced_script.py`.
*   **Step 3 (Local - Gemma):** Reads the final code and generates a git commit message in `commit_message.txt`.

```
