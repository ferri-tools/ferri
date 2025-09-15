# Ferri

Ferri is a local-first AI toolkit that acts as an intelligent director for foundation models. It evolves from a simple command runner into a proactive, agentic partner that can plan and execute complex development tasks.

Ferri creates secure, project-based environments with portable context, unifying your workflow across local (Ollama) and remote (API) models. The goal is to let you focus on your high-level goals, not on the minutiae of context management and command execution.

## The Ferri Architecture

Ferri is built in layers, allowing you to choose the right level of power for your task.

| Layer | Command(s) | Description |
|---|---|---|
| **L1: Core Execution** | `init`, `secrets`, `ctx`, `with` | The foundation. Manages your environment and executes synchronous, single-shot commands with injected context and secrets. |
| **L2: Workflow Automation** | `run`, `ps`, `yank`, `flow` | The automation layer. Runs commands as background jobs, monitors their status, retrieves their output, and orchestrates multi-step workflows. |
| **L3: Agentic Engine** | `do` | The intelligent director. Takes a high-level goal, formulates a multi-step plan, and executes it. Supports interactive debugging to pause and get user feedback on errors. |

## A Typical Workflow

Here's how you can go from zero to a context-aware AI query in four commands:

1.  **Initialize your project:**
    (This creates a secure `.ferri` directory for all state and secrets)
    ```bash
    ferri init
    ```

2.  **Securely store an API key:**
    (The key is encrypted and never leaves your machine)
    ```bash
    ferri secrets set OPENAI_API_KEY="sk-..."
    ```

3.  **Define your context:**
    (Tell Ferri which files are relevant for your AI tasks. Do this once.)
    ```bash
    ferri ctx add ./src README.md
    ```

4.  **Run commands with context injected:**
    (Use `with` to run any command in a secure, context-aware environment.)

    *Query a local model via Ollama:*
    ```bash
    ferri with --ctx -- ollama run llama3 "Based on the code, what is the primary goal of this project?"
    ```

    *Query a remote model by just changing a flag:*
    ```bash
    ferri with --ctx --model gpt-4o "Refactor the main function in my source code to be more modular."
    ```

## Usage Examples

### `with`: The Universal Executor

The `with` command is the workhorse of Ferri. It executes any command you give it, injecting secrets and context as needed.

#### 1. Code Comprehension with a Local Model
Ask questions about your codebase using a fast, local model running via Ollama.

```bash
# First, initialize Ferri and define the context for your project
ferri init
ferri ctx add ./src ./docs README.md

# Now, ask a question. Ferri injects your context into the prompt automatically.
ferri with --ctx -- ollama run llama3 "Based on the files in ./src, what is the primary purpose of this application?"
```

#### 2. Code Generation with a Remote Model
Use a powerful remote model to generate a new file, saving the output directly.

```bash
# Store your API key securely once
ferri secrets set OPENAI_API_KEY="sk-..."

# Use the '--model' flag to switch to a remote API.
# Ferri handles auth and context injection for you.
ferri with --ctx --model gpt-4o "Write a comprehensive test suite for the main function" > ./tests/main.test.js
```

#### 3. Using Ferri as a Secure Script Runner
Run any local script or tool that needs secrets, without exposing them in your shell history or hardcoding them.

```bash
# Your python script can now access the API key via standard environment variables.
ferri with -- python ./scripts/deploy.py

# (Inside deploy.py)
# import os
# api_key = os.getenv("OPENAI_API_KEY")
```

### `secrets`: Secure and Simple Secret Management

The `secrets` command manages sensitive data like API keys. Secrets are encrypted and stored locally in the `.ferri` directory.

#### 1. Set a Secret
You can set a secret directly, from a file, or from an environment variable.

```bash
# Set a secret directly
ferri secrets set GITHUB_TOKEN="ghp_..."

# Set from a file
ferri secrets set MY_SECRET --from-file ./secret.txt

# Set from an environment variable
ferri secrets set MY_SECRET --from-env MY_ENV_VAR
```
If you omit the value, Ferri will prompt you to enter it securely.

#### 2. List Secrets
List the names of all secrets stored for the project.

```bash
ferri secrets ls
```

#### 3. Remove a Secret
Remove a secret by its key.
```bash
ferri secrets rm GITHUB_TOKEN
```

## Commands

```
Usage: ferri [OPTIONS] COMMAND [ARGS]...

  Ferri is a local-first AI toolkit that acts as an intelligent director
  for foundation models.

Options:
  -v, --verbose    Enable verbose output for debugging.
  -h, --help       Show this message and exit.

Commands:
  init        Initialize a new Ferri project in the current directory.
  secrets     Manage encrypted, project-specific secrets like API keys.
  ctx         Manage the project's context from files or job outputs.
  with        Execute a command within a context-aware, synchronous environment.
  run         Run a command as a long-running background job.
  ps          List and manage active background jobs.
  yank        Fetch the output (stdout) of a completed background job.
  flow        Define and run multi-step, declarative AI workflows from a file.
  do          Execute a high-level goal with an AI-powered agentic engine.
```

## Command Modifiers

Modifiers are chainable flags that augment the behavior of Ferri's core commands. You can combine them to achieve more granular control over execution, safety, and performance.

---

### General Modifiers

*   **`--stream`**
    *   **Applies to**: `with`, `run`, `do`
    *   **Primary Use**: See the output of a command in real-time instead of waiting for it to complete. This is essential for long-running code generation or text summarization tasks.
    *   **Creative Use**: Use Ferri as a real-time conversational tool in your terminal, watching the AI "think" as it responds.

*   **`--model <model_name>`**
    *   **Applies to**: `with`, `run`, `do`
    *   **Primary Use**: Specify exactly which model to use for a command (e.g., `openai/gpt-4o`, `ollama/llama3`), overriding the project default.
    *   **Creative Use**: Run the same command multiple times with different models to compare their output for quality, speed, or style.

*   **`--no-cache`**
    *   **Applies to**: `with`, `run`, `do`
    *   **Primary Use**: Force a command to re-execute instead of returning a cached result. Use this when you've changed the underlying data and need a fresh response.
    *   **Creative Use**: When brainstorming or seeking random inspiration, use `--no-cache` to ensure the model provides a different, non-deterministic answer every time.

*   **`--silent`**
    *   **Applies to**: `run`, `flow`
    *   **Primary Use**: Suppress console output (like Job IDs). This is critical for running Ferri inside scripts (e.g., Git hooks, CI/CD jobs) where you only want to see the final output, not Ferri's own chatter.
    *   **Creative Use**: Fire off background jobs from a keybinding in your editor without ever leaving your code or seeing an intermediate message.

---

### Context & Data Handling

*   **`--ttl <duration>`**
    *   **Applies to**: `ctx`
    *   **Primary Use**: Add a file or job output to the context that automatically expires (e.g., `1h`, `30m`). This prevents your context from getting cluttered with temporary data that is only relevant for a short time.
    *   **Creative Use**: In a demo, add context that automatically cleans itself up after 5 minutes, ensuring the next demo starts with a clean slate.

*   **`--format <prompt>`**
    *   **Applies to**: `ctx`, `yank`
    *   **Primary Use**: Pre-process data before it enters the context. For example, summarizing a verbose log file or extracting key data points from a large JSON object to save tokens and improve signal-to-noise ratio.
    *   **Creative Use**: Transform data on the fly. Yank a CSV file but use `--format` to convert it to a Markdown table before printing it to the console.

---

### Workflow & Job Control

*   **`--interactive`**
    *   **Applies to**: `ps`, `do`
    *   **Primary Use**: For `ps`, this launches a terminal dashboard to monitor and manage running jobs. For `do`, it drops you into a debugging session when the agent gets stuck, allowing you to give it feedback to correct its course.
    *   **Creative Use**: Use `do --interactive` as a pair-programming partner, where the agent does the typing and you provide high-level guidance when it gets confused.

*   **`--retry <attempts>`**
    *   **Applies to**: `run`, `flow`
    *   **Primary Use**: Make your jobs more robust by automatically retrying them if they fail. This is essential for tasks that rely on network requests or other potentially flaky services.
    *   **Creative Use**: Create a job that waits for a service to come online by retrying a connection command every 30 seconds.

*   **`--parallel`**
    *   **Applies to**: `flow`
    *   **Primary Use**: Significantly speed up workflows by running independent jobs at the same time instead of one after another.
    *   **Creative Use**: Run a "bake-off" where multiple models process the same data in parallel, allowing you to quickly compare the results of each.

---

### Safety & Planning

*   **`--dry-run`**
    *   **Applies to**: `do`, `flow`
    *   **Primary Use**: See the plan of action before any files are changed or commands are executed. This is the most important safety feature for agentic tasks, letting you verify the agent's intentions.
    *   **Creative Use**: Use the agent's generated plan from a dry run as a personal to-do list, copying it into your notes to execute manually.

*   **`--gate <prompt>`**
    *   **Applies to**: `flow` (defined in the YAML file)
    *   **Primary Use**: Add a manual confirmation step to a workflow. This is critical for dangerous steps, like deploying to production or deleting resources, ensuring a human is there to give the final "go-ahead."
    *   **Creative Use**: In a tutorial workflow, use gates to pause execution and explain what's about to happen before proceeding to the next step.

*   **`--budget <amount>`**
    *   **Applies to**: `do`, `flow`
    *   **Primary Use**: Set a hard spending limit (e.g., `$1.00`) on any task that uses a paid API. This is a crucial safeguard to prevent accidental or runaway costs.
    *   **Creative Use**: Challenge the agent to complete a task as cheaply as possible by giving it a very small budget, forcing it to use tokens efficiently.
