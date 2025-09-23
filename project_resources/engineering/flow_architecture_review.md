# Architectural Review of `ferri flow`

## 1. Executive Summary

This document analyzes the architecture of the `ferri flow` feature based on the concern that its implementation has diverged from the project's core "philosophy of primitives." The analysis confirms this concern. The current `flow` implementation bypasses Ferri's own L2 job management system (`run`, `ps`, `yank`), creating a separate, inconsistent execution path. 

This report recommends refactoring `ferri flow` to be a true orchestrator of Ferri's primitives, which will improve consistency, unlock the power of the existing job management system for flows, and simplify long-term maintenance.

## 2. Analysis of `ferri flow` Architecture

### 2.1. The "Primitives" Philosophy

The project's `README.md` clearly defines a layered architecture where higher-level commands build upon a foundation of lower-level primitives. 

- **L1: Core Execution** (`with`, `ctx`, etc.)
- **L2: Workflow Automation** (`run`, `ps`, `yank`, `flow`)
- **L3: Agentic Engine** (`do`)

This layered approach is a core strength, as it provides consistency and predictability. The `README.md` itself shows an example of a `flow.yml` where each step is a direct call to a `ferri` primitive:

```yaml
# From README.md
jobs:
  - id: generate-docs
    command: 'ferri with --ctx --model gpt-4o "..." > DOCS.md'
  - id: write-tests
    command: 'ferri with --ctx DOCS.md --model gpt-4o "..." > main.test.js'
```

### 2.2. Divergence in the Current Implementation

The current implementation of `ferri flow`, as seen in `local_process_flow_test_plan.md`, uses a completely different, abstract syntax:

```yaml
# Current implementation
steps:
  - name: "generate-quotes"
    model:
      model: "gemini-pro"
      prompt: "..."
  - name: "filter-for-love"
    process:
      process: "grep -i 'love'"
```

This `model` and `process` syntax is a high-level abstraction. A review of `ferri-core/src/flow.rs` confirms that these steps are executed via a direct, internal implementation that bypasses the L2 job system entirely. 

### 2.3. The "Shortcut" and its Implications

By not using the `run`/`ps`/`yank` primitives, the current `flow` implementation introduces several architectural problems:

- **Inconsistency:** The way a model is called in a flow is different from how it's called on the command line. The same is true for shell processes.
- **Reduced Functionality:** Flow steps do not appear in `ferri ps`. Their output cannot be retrieved with `ferri yank`. They cannot be run as true background jobs.
- **Increased Maintenance:** There are now two separate execution paths to maintain: the `ferri with`/`run` path and the `ferri flow` path. A bug fix in one (like the environment variable or stdin issues we just fixed) may not apply to the other, leading to redundant work and potential for more bugs.

## 3. Proposed Architectural Refactoring

To address these issues, `ferri flow` should be refactored to be a thin orchestrator that generates and executes a series of `ferri` commands.

- **High-Level Goal:** Each step in a `flow.yml` should translate to a `ferri run` command. The flow runner will then use `ferri ps` to monitor the job's status and `ferri yank` to retrieve its output for the next step.

- **Benefits:**
    - **Unified Architecture:** All command execution goes through the same battle-tested L1/L2 primitives.
    - **Full Feature Set:** Flow steps would become first-class citizens in the Ferri ecosystem, appearing in `ferri ps` and manageable with `yank` and `kill`.
    - **Simplified Codebase:** The custom execution logic in `ferri-core/src/flow.rs` can be significantly simplified, as it will only need to orchestrate `ferri` commands, not replicate their functionality.

## 4. Proposed Tickets

To implement this refactoring, the following tickets should be created. They are designed to be implemented sequentially.

---

**Ticket: T72 - Refactor `flow.yml` to use `command` syntax**

- **Goal:** Align the `flow.yml` schema with the original vision in the `README.md` by replacing the `model` and `process` step kinds with a single `command` field.
- **Subtask (T72.1):** In `ferri-core/src/flow.rs`, update the `Step` and `StepKind` structs to remove `ModelStep` and `ProcessStep` and replace them with a single `command: String` field.
- **Subtask (T72.2):** Update the YAML parser to reflect this new, simpler structure.
- **Subtask (T72.3):** Update all existing `.yml` files in the project (`hybrid_process_flow.yml`, `code_review_flow.yml`, etc.) to use the new syntax. For example:

  ```yaml
  # Before
  - name: "filter-for-love"
    process:
      process: "grep -i 'love'"

  # After
  - name: "filter-for-love"
    command: "grep -i 'love'"
  ```

---

**Ticket: T73 - Integrate `ferri run` into Flow Engine**

- **Goal:** Modify the flow runner to execute each step as a background job using the `ferri run` primitive.
- **Subtask (T73.1):** In `ferri-core/src/flow.rs`, modify the `run_pipeline` function to construct a `ferri run` command string for each step.
- **Subtask (T73.2):** The constructed command should include the step's `command` field and handle I/O redirection (e.g., `ferri run -- ... > step_output.tmp`).
- **Subtask (T73.3):** Use `jobs::submit_job` to execute the constructed command string.

---

**Ticket: T74 - Implement Synchronous Execution via Polling**

- **Goal:** Since flows are currently synchronous, implement a mechanism to wait for each job to complete before starting the next one.
- **Subtask (T74.1):** After submitting a job in `run_pipeline`, enter a polling loop.
- **Subtask (T74.2):** In the loop, call `jobs::list_jobs` periodically (e.g., every 200ms) to check the status of the submitted job.
- **Subtask (T74.3):** Exit the loop when the job status is `Completed` or `Failed`.
- **Subtask (T74.4):** If the job fails, propagate the error and stop the flow.

---

**Ticket: T75 - Integrate `ferri yank` for Inter-Step I/O**

- **Goal:** Use `ferri yank` to pass output from one step as input to the next.
- **Subtask (T75.1):** Modify the `run_pipeline` input handling logic. If a step's `input` field refers to a previous step, the runner should call `jobs::get_job_output` (the function behind `ferri yank`) to retrieve the output.
- **Subtask (T75.2):** The retrieved output should then be piped as stdin to the next step's `ferri run` command.
- **Subtask (T75.3):** Update integration tests to verify that a multi-step flow correctly pipes data between steps using the new `run`/`ps`/`yank` foundation.
