# Ferri

Ferri is a local-first AI toolkit that acts as an intelligent director for foundation models. It evolves from a simple command runner into a proactive, agentic partner that can plan and execute complex development tasks.

Ferri creates secure, project-based environments with portable context, unifying your workflow across local (Ollama) and remote (API) models. The goal is to let you focus on your high-level goals, not on the minutiae of context management and command execution.

## Core Concepts

*   **Project-Based:** Ferri operates within your project directory, keeping all context, secrets, and job history in a local `.ferri` directory.
*   **Context-Aware:** You tell Ferri what files are important (`ferri ctx add`), and it automatically injects them into your AI prompts.
*   **Secure:** Secrets like API keys are encrypted and automatically made available to commands, so you never have to expose them in your shell.
*   **Layered Architecture:** Start with simple commands and gradually adopt more powerful, automated, and agentic features as you need them.

---

## Use Cases & Examples

This guide is a collection of practical, real-world use cases to get you started with Ferri.

### 1. Code & Documentation Analysis

Ask questions about your codebase using a fast, local model.

```bash
# 1. Initialize Ferri in your project
ferri init

# 2. Add your source code and documentation to the context
ferri ctx add ./src ./docs README.md

# 3. Ask a question using a local model
ferri with --ctx -- ollama run llama3 "Based on the provided context, what is the primary purpose of this application?"

# 4. Ask a more specific question
ferri with --ctx -- ollama run llama3 "What is the role of the `main` function in the `main.rs` file?"
```

### 2. Code Generation & Refactoring

Use a powerful remote model to generate new code or refactor existing code.

```bash
# 1. Store your OpenAI API key securely
ferri secrets set OPENAI_API_KEY="sk-..."

# 2. Generate a new test file
ferri with --ctx --model gpt-4o "Write a comprehensive test suite for the main function" > ./tests/main.test.js

# 3. Refactor an existing file
ferri with --ctx --model gpt-4o "Refactor the main function in my source code to be more modular. Output only the new code." > src/main.rs
```

### 3. Securely Running Scripts

Run any local script that needs access to secrets without hardcoding them.

```bash
# 1. Store your database connection string as a secret
ferri secrets set DB_URL="postgres://user:pass@host:port/db"

# 2. Run a Python script that uses this secret
ferri with -- python ./scripts/db_backup.py

# 3. Inside db_backup.py, access the secret from the environment
# import os
# db_url = os.getenv("DB_URL")
```

### 4. Automating Multi-Step Workflows

Define a series of steps in a YAML file and run them as a single command.

**`ci-prep.yml:`**
```yaml
name: "Prepare for CI"
jobs:
  - id: generate-docs
    description: "Generate documentation for all source files."
    command: 'ferri with --ctx --model gpt-4o "Generate technical markdown docs for the codebase" > DOCS.md'

  - id: write-tests
    description: "Write unit tests based on the new documentation."
    dependencies: [generate-docs]
    command: 'ferri with --ctx DOCS.md --model gpt-4o "Write unit tests for the main module" > main.test.js'
```

```bash
# Execute the entire workflow
ferri flow run ci-prep.yml
```

### 5. Agentic Task Execution

Give Ferri a high-level goal and let it formulate and execute a plan.

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

For a detailed list of all command modifiers and advanced options, see [COMMAND_MODIFIERS.md](./COMMAND_MODIFIERS.md).