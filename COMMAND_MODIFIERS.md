# Command Modifiers

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
