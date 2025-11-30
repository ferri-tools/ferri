# The State of `ferri flow`

This document serves as the official guide and single source of truth for the `ferri flow` feature. It outlines the core philosophy, defines the schema, provides canonical examples, and explains the underlying execution model.

## 1. Core Philosophy: A Script of Primitives

The fundamental principle of `ferri flow` is **transparency**. A flow is not a high-level abstraction; it is a simple, declarative script of shell commands.

- **Zero Magic:** The flow engine is a straightforward orchestrator. It reads a command from a step, executes it, and moves to the next. There is no hidden logic.
- **Intuitive and Consistent:** If you know how to use `ferri` and other command-line tools in your terminal, you already know how to write a flow. Knowledge is directly transferable.
- **Debuggability:** When a flow fails, you can copy the exact `run` command from the failed step and execute it directly in your terminal to debug the issue in isolation.

Every operation within a flow is a literal command, ensuring that the workflow is explicit, easy to understand, and easy to maintain.

---

## 2. Schema Definition

The following schema is the complete and guaranteed-to-run specification for a `ferri-flow.yml` file. It is based on the current, stable implementation.

### Top-Level Object

| Field        | Type    | Required | Description                                                    |
|--------------|---------|----------|----------------------------------------------------------------|
| `apiVersion` | String  | Yes      | The version of the Flow schema. Must be `ferri.flow/v1alpha1`. |
| `kind`       | String  | Yes      | The type of document. Must be `Flow`.                          |
| `metadata`   | Mapping | Yes      | Contains metadata for the flow, such as its name.              |
| `spec`       | Mapping | Yes      | The specification of the workflow's jobs and steps.            |

### `metadata`

| Field  | Type   | Required | Description                              |
|--------|--------|----------|------------------------------------------|
| `name` | String | Yes      | A unique, `kebab-case` name for the flow. |

### `spec`

| Field  | Type    | Required | Description                                                            |
|--------|---------|----------|------------------------------------------------------------------------|
| `jobs` | Mapping | Yes      | A map where each key is a unique job ID and the value is a Job object. |

### `jobs.<job-id>`

| Field     | Type                | Required | Description                                                              |
|-----------|---------------------|----------|--------------------------------------------------------------------------|
| `name`    | String              | No       | A human-readable name for the job.                                       |
| `runs-on` | String              | No       | Specifies the executor. If omitted, defaults to `process`.               |
| `needs`   | Sequence of Strings | No       | A list of job IDs that must complete successfully before this job runs.  |
| `steps`   | Sequence of Steps   | Yes      | A list of sequential steps to execute within the job.                    |

### `steps` Item

| Field  | Type   | Required | Description                                         |
|--------|--------|----------|-----------------------------------------------------|
| `name` | String | No       | A human-readable name for the step.                 |
| `run`  | String | Yes      | The exact shell command to execute for this step.   |

---

## 3. Canonical Examples

These two examples are the dogma for how to write flows. They are guaranteed to work with the current version of `ferri`.

### Example 1: Simple Sequential Jobs

This flow demonstrates the basic structure, job dependencies, and simple shell commands.

**File: `examples/canonical-flows/hello-bye-flow.yml`**
```yaml
# sample-flow.yml
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: simple-echo-flow
spec:
  jobs:
    # The first job to run
    say-hello:
      name: "Say Hello"
      runs-on: process # This is the default and only implemented executor
      steps:
        - name: "Echo a greeting"
          run: echo "Hello from the first job!"

    # This job will run after 'say-hello' completes
    say-goodbye:
      name: "Say Goodbye"
      needs:
        - say-hello
      steps:
        - name: "Echo a farewell"
          run: echo "The first job is done. Goodbye!"
```

### Example 2: Chaining `ferri` and `sed` for File I/O

This flow demonstrates a more advanced workflow that chains `ferri` commands with standard Unix tools (`sed`, `cat`) to process files.

**File: `examples/canonical-flows/gemma-flow.yml`**
```yaml
# poem-flow.yml
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: poem-generator-and-editor
spec:
  jobs:
    write-poem:
      name: "Write Poem"
      steps:
        - name: "Use Gemma to write a poem about love"
          run: /Users/jorgeajimenez/repos/ferri/gemini-worktree/target/debug/ferri with --model gemma --output poem.txt -- "write a short poem about love"

    edit-poem:
      name: "Edit Poem with sed"
      needs:
        - write-poem
      steps:
        - name: "Replace 'love' with 'code'"
          run: "sed 's/love/code/gI' poem.txt > coded-poem.txt"

    display-result:
      name: "Display Final Result"
      needs:
        - edit-poem
      steps:
        - name: "Cat the final poem"
          run: "cat coded-poem.txt"
```

---

## 4. How It Works: The Orchestrator and Executor

### The Flow Orchestrator

When you execute `ferri flow run <file.yml>`, the orchestrator performs the following actions:
1.  **Parse & Validate:** It reads and validates the YAML file against the defined schema.
2.  **Build Dependency Graph:** It analyzes the `needs` fields of all jobs to build a directed acyclic graph (DAG).
3.  **Topological Sort:** It performs a topological sort on the graph to determine the execution order. This produces "waves" of jobs that can be run in parallel. For example, all jobs with no dependencies are in the first wave. All jobs that depend only on the first wave jobs are in the second, and so on.
4.  **Execute Waves:** The orchestrator executes each wave of jobs. It spawns a separate thread for each job in the wave and waits for all of them to complete before moving to the next wave. If any job fails, the entire flow is halted.

### The `process` Executor

The executor is the component responsible for actually running the steps within a job.
- **Current Implementation:** The only executor currently implemented is named `process`.
- **Functionality:** The `process` executor is very simple. For each step in a job, it takes the `run` string and executes it as a shell command using `sh -c`. It executes the steps sequentially and waits for each one to complete before starting the next. If any step fails (returns a non-zero exit code), the job is marked as failed.

---

## 5. Architectural Note: `ferri flow` vs. `ferri run`

It is important to understand the current architectural state of `ferri`.

The `ferri flow` engine described in this document is a **foreground process orchestrator**. It is a self-contained system designed to execute a series of shell commands as defined in a YAML file.

This system is currently **separate and distinct** from Ferri's background job management system, which is controlled by the `ferri run`, `ferri ps`, and `ferri yank` commands.

**Implications:**
-   Jobs and steps executed by `ferri flow` **will not** appear in the output of `ferri ps`.
-   They are not background daemons; the `ferri flow run` command will block until the entire workflow is complete.
-   This represents an architectural divergence. The original intent was for `ferri flow` to be a high-level orchestrator *of* the `ferri run` primitives.

**Future Direction:**
A key architectural goal is to refactor the `ferri flow` engine to unify these two systems. In the future, each step in a flow will be executed as a `ferri run` job. This will bring significant benefits:
-   **Unified Architecture:** All execution will go through the same battle-tested primitives.
-   **Full Feature Set:** Flow steps will become first-class citizens, appearing in `ferri ps` and manageable with `yank` and `kill`.
-   **Simplified Codebase:** The custom execution logic in the flow engine can be removed in favor of the core job management system.

Until this refactoring is complete, please refer to this document as the guide for the current, stable implementation.
