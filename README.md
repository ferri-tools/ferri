# Ferri

Ferri is a local-first AI toolkit that acts as an intelligent director for foundation models. It evolves from a simple command runner into a proactive, agentic partner that can plan and execute complex development tasks.

Ferri creates secure, project-based environments with portable context, unifying your workflow across local (Ollama) and remote (API) models. The goal is to let you focus on your high-level goals, not on the minutiae of context management and command execution.

## The Ferri Architecture

Ferri is built in layers, allowing you to choose the right level of power for your task.

| Layer | Command(s) | Description |
|---|---|---|
| **L1: Core Execution** | `init`, `secrets`, `ctx`, `with` | The foundation. Manages your environment and executes synchronous, single-shot commands with injected context and secrets. |
| **L2: Workflow Automation** | `run`, `ps`, `flow` | The automation layer. Runs commands as long-running background jobs, monitors their status in an interactive TUI, and orchestrates multi-step, declarative workflows. |
| **L3: Agentic Engine** | `do` | The intelligent director. Takes a high-level goal, formulates a multi-step plan, and executes it. Supports interactive debugging to pause and get user feedback on errors. |

## Interactive Features

Ferri is designed for a tight feedback loop. When things go wrong, you have tools to see what's happening and intervene.

*   **Interactive Job Dashboard:** Use `ferri ps -i` to launch a terminal-based UI where you can see the real-time status of all running jobs, inspect their logs, and visualize workflow dependencies.
*   **Interactive Debugging:** When the agent encounters an error, it can pause execution and ask for your input, turning a failed run into a collaborative debugging session.

## Usage Examples

#### 1\. Basic Execution (`with`)

Run a one-shot command to get a quick answer from a local model.

```bash
# Initialize project and add context
ferri init
ferri ctx add ./src

# Query a local model via Ollama
ferri with --ctx -- ollama run llama3 "Based on the code, what is the primary goal of this project?"
```

#### 2\. Advanced Workflow (`flow`)

Define a multi-step process in a YAML file to automate repetitive tasks.

`ci-prep.yml:`

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

#### 3\. Agentic Task (`do`)

Give Ferri a high-level objective and let it figure out the steps.

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

## Commands

```
Usage: ferri [OPTIONS] COMMAND [ARGS]...

  Ferri is a local-first AI toolkit that acts as an intelligent director
  for foundation models.

Options:
  -v, --verbose    Enable verbose output for debugging.
  --version        Show the version number and exit.
  -h, --help       Show this message and exit.

Commands:
  init        Initialize a new Ferri project in the current directory.
  secrets     Manage encrypted, project-specific secrets like API keys.
  ctx         Manage the project's context (files and directories).
  with        Execute a command within a context-aware, synchronous environment.
  run         Run a command as a long-running background job.
  ps          List and manage active background jobs.
  flow        Define and run multi-step, declarative AI workflows from a file.
  do          Execute a high-level goal with an AI-powered agentic engine.
```
