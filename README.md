# Ferri

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
| **L1: Core Execution** | `init`, `secrets`, `ctx`, `with` | The foundation. Manages your environment and executes synchronous, single-shot commands with injected context and secrets. |
| **L2: Workflow Automation** | `run`, `ps`, `yank`, `flow` | The automation layer. Runs commands as background jobs, monitors their status, retrieves their output, and orchestrates multi-step workflows. |
| **L3: Agentic Engine** | `do` | The intelligent director. Takes a high-level goal, formulates a multi-step plan, and executes it. Supports interactive debugging to pause and get user feedback on errors. |

---

## Command Reference

| Command | Description |
|---|---|
| `init` | Initialize a new Ferri project in the current directory. |
| `secrets` | Manage encrypted, project-specific secrets like API keys. |
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

Initializes a new Ferri project in the current directory.

This command creates the `.ferri` directory where all project-specific state, context, secrets, and logs are stored. It's the first step for any new project.

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

Manages encrypted, project-specific secrets like API keys.

Secrets are encrypted and stored locally, ensuring that sensitive information never leaves your machine or gets committed to source control.

**Use Case: Storing an API Key**
```bash
# Securely store your OpenAI API key
ferri secrets set OPENAI_API_KEY="sk-..."

# If you omit the value, Ferri will prompt you to enter it securely
ferri secrets set ANTHROPIC_API_KEY
# Output: Enter value for ANTHROPIC_API_KEY: *****************
```

**Use Case: Setting a Secret from a File or Environment Variable**
```bash
# Read a secret from a file
ferri secrets set GCP_SERVICE_ACCOUNT --from-file ./service-account.json

# Read a secret from an environment variable
ferri secrets set GITHUB_TOKEN --from-env GITHUB_PAT
```

**Use Case: Listing and Removing Secrets**
```bash
# List the keys of all stored secrets
ferri secrets ls
# Output:
# OPENAI_API_KEY
# ANTHROPIC_API_KEY
# GCP_SERVICE_ACCOUNT
# GITHUB_TOKEN

# Remove a secret you no longer need
ferri secrets rm GITHUB_TOKEN
```

---

### `ferri ctx`

Manages the project's context from files, directories, or job outputs.

The context is the set of files and data that Ferri will provide to an AI model to inform its responses.

**Use Case: Adding Files and Directories to the Context**
```bash
# Add the entire source directory
ferri ctx add ./src

# Add specific files
ferri ctx add README.md CONTRIBUTING.md

# Add multiple items at once
ferri ctx add ./src ./docs package.json
```

**Use Case: Listing and Removing Context Items**
```bash
# List all items currently in the context
ferri ctx ls

# Remove an item from the context
ferri ctx rm ./docs
```

---

### `ferri with`

Executes a command within a context-aware, synchronous environment.

This is the workhorse command of Ferri. It injects secrets as environment variables and can pipe context directly into the command's standard input.

**Use Case: Code Comprehension with a Local Model**
```bash
# Ask a question about your codebase using a local Ollama model
ferri with --ctx -- ollama run llama3 "Based on the files in ./src, what is the primary purpose of this application?"
```

**Use Case: Code Generation and Saving to a File**
```bash
# Use a powerful remote model to generate a new file
# The '>' operator saves the output directly to the file
ferri with --ctx --model gpt-4o "Write a comprehensive Jest test suite for the main function" > ./tests/main.test.js

# Alternatively, use the --output flag for the same result
ferri with --ctx --model gpt-4o --output ./tests/main.test.js "Write a comprehensive Jest test suite for the main function"
```

**Use Case: Securely Running Scripts**
```bash
# Your python script can now access the API key via standard environment variables
ferri with -- python ./scripts/deploy.py

# (Inside deploy.py)
# import os
# api_key = os.getenv("OPENAI_API_KEY")
```

**Use Case: Interactive Refactoring**
Give the model supervised control to modify your files. Ferri will show you a plan and ask for confirmation before making changes.
```bash
# The '--interactive' flag enables the model to plan and execute file system changes
ferri with --ctx --interactive --model gpt-4o "Refactor the database logic from server.js into its own module at ./src/db.js and update server.js to use it."

# Ferri will first output the model's plan for your approval:
# [PLAN]
# 1. Create a new file ./src/db.js with the database connection logic.
# 2. Modify ./src/server.js to import and use the new module.
# Proceed with this plan? [y/n]
```

---

### `ferri run`, `ps`, and `yank`

Manages long-running, asynchronous background jobs.

This trio of commands allows you to offload time-consuming tasks, monitor their progress, and retrieve their results when they're done.

**Use Case: The Asynchronous Workflow**

**Step 1: Start a long-running job with `ferri run`**
This command kicks off a task in the background and immediately returns a Job ID.
```bash
# Generate documentation for the entire codebase as a background job
ferri run -- ferri with --ctx --model gemma "Generate a complete project summary in Markdown"
# Output: Job submitted: job-b4c5d6
```

**Step 2: Check the status with `ferri ps`**
Monitor the status of active and completed jobs.
```bash
ferri ps
# Output:
# JOB ID      STATUS      COMMAND
# job-b4c5d6  COMPLETED   ferri with --ctx --model gemma...
```

**Step 3: Retrieve the output with `ferri yank`**
Once the job is complete, "yank" its output to the console or a file.
```bash
# Print the output to the console
ferri yank job-b4c5d6

# Save the output directly to a new README file
ferri yank job-b4c5d6 --output README_AI.md
```

**Step 4 (Bonus): Add the job's output directly to the context**
```bash
# Add the generated summary to the context as a "virtual" file
ferri ctx add --from-job job-b4c5d6 --as project_summary.md
```

---

### `ferri flow`

Defines and runs multi-step, declarative AI workflows from a file.

Automate complex, repetitive tasks by defining them in a simple YAML file.

**Use Case: Automating CI Preparation**
Create a `ci-prep.yml` file to define a workflow that first generates documentation and then writes unit tests based on it.

**`ci-prep.yml:`**
```yaml
name: "Prepare for CI"
jobs:
  - id: generate-docs
    description: "Generate documentation for all source files."
    command: 'ferri with --ctx --model gpt-4o "Generate technical markdown docs for the codebase" > DOCS.md'

  - id: write-tests
    description: "Write unit tests based on the new documentation."
    dependencies: [generate-docs] # This job waits for 'generate-docs' to finish
    command: 'ferri with --ctx DOCS.md --model gpt-4o "Write unit tests for the main module" > main.test.js'
```

```bash
# Execute the entire workflow with a single command
ferri flow run ci-prep.yml
```

---

### `ferri do`

Executes a high-level goal with an AI-powered agentic engine.

This is the most advanced layer of Ferri. You provide a high-level objective, and Ferri's agentic engine will formulate a plan, seek your approval, and execute it.

**Use Case: Agentic Task Execution**
```bash
# Tell Ferri what you want to achieve
ferri do "Add a new '/api/users' endpoint to my Express app. It should have a route, a controller with a placeholder function, and be registered in the main app file."

# Ferri will generate and propose a plan for your approval:
# PLAN:
# 1. Create file: src/routes/users.js
# 2. Create file: src/controllers/users.js
# 3. Modify file: src/app.js to import and use the new router.
# Proceed? [y/N]
```

---

## Command Modifiers

For a detailed list of all command modifiers and advanced options that can be chained to these commands (e.g., `--stream`, `--dry-run`, `--interactive`), see [COMMAND_MODIFIERS.md](./COMMAND_MODIFIERS.md).