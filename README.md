# Ferri

Ferri is a local-first AI toolkit that acts as an intelligent director for foundation models. It creates secure, project-based environments with portable context, unifying your workflow across local (Ollama) and remote (API) models. The goal is to let you focus on code, not on context management.

## Core Features

### Demo v1
| Feature Group | Feature Name | Description |
|---|---|---|
| Core Environment | Project Initialization (`ferri init`) | Creates a self-contained `.ferri` directory in the user's project to store all state, logs, and configuration. |
| Core Environment | Secure Secrets Management (`ferri secrets`) | Allows developers to set, update, and remove encrypted secrets (like API keys) on a per-project basis. Secrets are encrypted with a master password. |
| Context Management | Define Context (`ferri ctx add`) | Allows a user to specify files and directories that constitute the project's "context." This list is stored within the `.ferri` environment. |
| Context Management | Remove Context (`ferri ctx rm`) | Allows a user to remove files and directories from the project's context. |
| Core Workflow | Unified Command Runner (`ferri with`) | Executes any given command in a temporary, secure environment. Injects stored secrets as environment variables. |
| Core Workflow | Context Injection (`--ctx` flag) | A flag for `ferri with` that reads the defined context, concatenates the file contents, and stuffs them into the prompt for the target model. |
| Model Integration | Local Model Support (Ollama) | The `ferri with` command should seamlessly pipe the context-aware prompt to a locally running Ollama model. |
| Model Integration | Remote Model Support (`--model` flag) | A flag for `ferri with` that sends the context-aware prompt to a specified remote API (e.g., gpt-4o), handling authentication with the stored secrets. |

### Future Vision
| Feature Group | Feature Name | Description |
|---|---|---|
| Job Management | Asynchronous Runner (`ferri run`) | Runs any command as a long-running background job, returning an immediate Job ID for tracking. |
| Job Management | Job Dashboard (`ferri ps`) | Provides an interactive dashboard to monitor the status, logs, and output of all background jobs. |
| Workflow Automation | Declarative Workflows (`ferri flow`) | Allows developers to define a multi-step AI pipeline in a YAML file, chaining jobs together with dependencies. |
| UX/DX | Improved Logging & Error Handling | Provide more helpful and verbose logging options and user-friendly error messages for easier debugging. |
| Integrations | Third-Party Tool Integrations | Explore and implement integrations with other common developer tools (e.g., Git, IDEs) to further streamline workflows. |

## Usage

A typical workflow with Ferri looks like this:

1.  **Initialize your project:**
    ```bash
    ferri init
    ```
2.  **Securely store an API key:**
    ```bash
    ferri secrets set OPENAI_API_KEY="sk-..."
    ```
3.  **Define your context:**
    ```bash
    ferri ctx add ./src README.md
    ```
4.  **Run commands with context injected:**
    ```bash
    # Query a local model via Ollama
    ferri with --ctx -- ollama run llama3 "Based on the code, what is the primary goal of this project?"

    # Query a remote model by just changing a flag
    ferri with --ctx --model gpt-4o "Refactor the main function in my source code to be more modular."
    ```

## Commands

```
Usage: ferri [OPTIONS] COMMAND [ARGS]...

  Ferri is a local-first AI toolkit that acts as an intelligent director for
  foundation models. It creates secure, project-based environments with
  portable context, unifying your workflow across local (Ollama) and remote
  (API) models.

  The goal is to let you focus on code, not on context management.

Options:
  -v, --verbose    Enable verbose output for debugging.
  --version        Show the version number and exit.
  -h, --help       Show this message and exit.

Commands:
  init        Initialize a new Ferri project in the current directory.
  secrets     Manage encrypted, project-specific secrets like API keys.
  ctx         Manage the project's context (files and directories).
  with        Execute a command within a context-aware environment.
  run         (Coming Soon) Run a command as a long-running background job.
  ps          (Coming Soon) List and manage active background jobs.
  flow        (Coming Soon) Define and run multi-step, declarative AI workflows.
```
