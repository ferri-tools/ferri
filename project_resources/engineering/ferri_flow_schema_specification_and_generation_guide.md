# Ferri Flow Schema Specification and Generation Guide (Final)

**Objective:**
This document provides a complete and formal specification for the `ferri-flow.yml` file format. Use this specification as the single source of truth for generating valid, secure, and self-contained Ferri Flow workflow files.

**Core Design Philosophy:**
The Ferri Flow schema is built on three foundational principles:
1.  **Declarative and Self-Contained:** Every workflow is defined in a single YAML file. All logic, commands, and dependencies are explicitly stated within this file or within versioned, reusable actions.
2.  **Security by Default:** The primary method for passing data is a managed, in-memory context (`ctx`). Direct filesystem access is a deliberate, explicit opt-in (`workspaces`), forcing security-conscious decisions.
3.  **Industry-Standard Structure:** The schema adopts proven structural patterns from systems like Kubernetes and GitHub Actions for clarity, versioning, and long-term maintainability.

---
**I. Schema Definition: Top-Level Object**
---

Every `ferri-flow.yml` file is a single document containing a root map. This root map MUST contain the following four top-level keys:

`apiVersion`
- Type: String
- Required: Yes
- Description: Identifies the version of the `ferri flow` schema. This enables backward-compatible evolution.
- Example: `ferri.flow/v1alpha1`

`kind`
- Type: String
- Required: Yes
- Description: Specifies the type of object being defined. For a standard workflow, this value MUST be `Flow`.
- Example: `Flow`

`metadata`
- Type: Object
- Required: Yes
- Description: A map containing data that identifies and organizes the workflow.
- Contains:
    - `name` (String, Required): The unique, `kebab-case` name of the flow.
    - `labels` (Map, Optional): Key-value string pairs for organizing and selecting flows.
    - `annotations` (Map, Optional): Key-value string pairs for attaching arbitrary, non-identifying metadata.

`spec`
- Type: Object
- Required: Yes
- Description: The detailed specification of the workflow's desired state, defining its parameters, resources, and execution logic. See Section II for details.

---
**II. The `spec` Block Definition**
---

The `spec` block is the core of the workflow, describing its structure and components.

`inputs`
- Type: Map
- Required: Optional
- Description: Defines the parameters that can be passed to the flow at runtime. Each key in the map is an input name. The value is an object with the following properties:
    - `description` (String, Optional): A human-readable explanation of the input.
    - `type` (String, Required): The data type. Supported types are `string`, `number`, `boolean`.
    - `default` (Any, Optional): A default value to use if one is not provided.

`workspaces`
- Type: Array of Objects
- Required: Optional
- Description: Defines shared storage volumes for high-performance, direct filesystem I/O between jobs. This is the explicit opt-in for filesystem access. Each object in the array contains:
    - `name` (String, Required): A unique name for the workspace within the flow.

`jobs`
- Type: Map
- Required: Yes
- Description: A map where each key is a unique `job-id` and the value is an object defining a job. Jobs represent a collection of steps that run on a single runner. By default, all jobs run in parallel unless dependencies are specified. See Section III for details.

---
**III. The `jobs` and `steps` Block Definition**
---

The `jobs` map contains one or more job definitions.

`<job-id>`
- Type: Object
- Required: Yes
- Description: The job definition. The key (`<job-id>`) is the unique, `kebab-case` identifier for the job.
- Contains:
    - `name` (String, Optional): A human-readable display name for the job.
    - `runs-on` (String, Required): Specifies the runner environment for the job (e.g., `ubuntu-latest`).
    - `needs` (Array of Strings, Optional): An array of `job-id`s that must complete successfully before this job can start. This defines the execution order.
    - `steps` (Array of Objects, Required): An array of sequential steps to be executed within the job.

Each object in the `steps` array defines a single unit of work.

`steps` Array Item
- Type: Object
- Description: A single step definition. A step MUST contain either a `run` key or a `uses` key, but not both.
- Contains:
    - `id` (String, Optional): A unique, `kebab-case` identifier for the step within the job. Required if other steps need to reference its outputs.
    - `name` (String, Optional): A human-readable name for the step.
    - `run` (String, Conditional): A self-contained shell command or multi-line script to execute.
    - `uses` (String, Conditional): The identifier of a reusable action to execute (e.g., `actions/checkout@v4`).
    - `with` (Map, Optional): A map of key-value input parameters for a reusable action specified by `uses`.
    - `env` (Map, Optional): A map of environment variables to set for this specific step.
    - `workspaces` (Array of Objects, Optional): Specifies which flow-level `workspaces` to mount into this step's environment. Each object contains:
        - `name` (String, Required): The name of a workspace defined in `spec.workspaces`.
        - `mountPath` (String, Required): The absolute path inside the step's environment where the workspace will be mounted.
        - `readOnly` (Boolean, Optional, Default: `false`): If `true`, the workspace is mounted in read-only mode.
    - `retryStrategy` (Object, Optional): Defines the policy for retrying a step upon failure. See Section V for details.

---
**IV. I/O and Context Model**
---

Data is passed between steps and jobs using a managed context and an expression syntax.

**Expression Syntax:**
- Format: `${{ <expression> }}`
- Description: This syntax is used to access data from the managed context. The orchestrator evaluates these expressions before executing a step.

**Available Contexts:**
- `ctx.inputs.<input-name>`: Accesses a flow-level input parameter.
- `ctx.steps.<step-id>.outputs.<output-name>`: Accesses an output from a previous step within the same job.
- `ctx.jobs.<job-id>.outputs.<output-name>`: Accesses an output from a different job that is listed in the `needs` block.

**Setting Outputs (`ferri-runtime`):**
- Command: `ferri-runtime set-output <name>=<value>`
- Description: This is a special, internal command used within a `run` block or by a reusable action to declare an output for a step. The `ferri-runtime` utility is injected by the orchestrator into the step's environment and is **only available and functional within a running flow**. It is the sole mechanism for a step to communicate data back to the orchestrator's managed context.

---
**V. Resiliency: The `retryStrategy` Block**
---

The `retryStrategy` object can be added to any step to control its behavior on failure.

`limit`
- Type: Integer
- Required: Optional (Default: 0, meaning no retries)
- Description: The maximum number of times to retry the step after an initial failure.

`retryPolicy`
- Type: String
- Required: Optional (Default: `OnFailure`)
- Description: Specifies the condition for a retry.
    - `OnFailure`: Retries only if the step's main command exits with a non-zero code (an application-level failure).
    - `OnError`: Retries on infrastructure or system errors that prevent the step from running (e.g., container setup failure, network issues).
    - `Always`: Retries on any failure or error.

`backoff`
- Type: Object
- Required: Optional
- Description: Configures an exponential backoff delay between retries.
- Contains:
    - `duration` (String, Required): The initial duration to wait before the first retry (e.g., "10s", "1m").
    - `factor` (Integer, Optional, Default: 2): The multiplier for the duration on each subsequent retry.
    - `maxDuration` (String, Optional): The maximum possible delay between retries.
