## 1. Schema Analysis from Source Code

Based on a static analysis of the `ferri` Rust source code, the following YAML schema is guaranteed to be parsed and executed by the current implementation. The analysis focuses strictly on implemented logic, primarily the `ProcessExecutor`, and ignores fields that are defined in structs but not yet used in execution.

### Top-Level Object

| Field         | Type    | Required | Description                                                              |
|---------------|---------|----------|--------------------------------------------------------------------------|
| `apiVersion`  | String  | Yes      | The version of the Flow schema. Must be `ferri.flow/v1alpha1`.           |
| `kind`        | String  | Yes      | The type of document. Must be `Flow`.                                    |
| `metadata`    | Mapping | Yes      | Contains metadata for the flow.                                          |
| `spec`        | Mapping | Yes      | The specification of the workflow.                                       |

---

### `metadata`

| Field      | Type    | Required | Description                               |
|------------|---------|----------|-------------------------------------------|
| `name`     | String  | Yes      | A unique name for the flow (kebab-case).  |

---

### `spec`

| Field  | Type              | Required | Description                                                              |
|--------|-------------------|----------|--------------------------------------------------------------------------|
| `jobs` | Mapping           | Yes      | A map where each key is a unique job ID and the value is a Job object.   |

---

### `jobs.<job-id>`

| Field     | Type                | Required | Description                                                              |
|-----------|---------------------|----------|--------------------------------------------------------------------------|
| `name`    | String              | No       | A human-readable name for the job.                                       |
| `runs-on` | String              | No       | Specifies the executor. If omitted, defaults to `process`. Currently, this is the only implemented executor. |
| `needs`   | Sequence of Strings | No       | A list of job IDs that must complete successfully before this job will run. |
| `steps`   | Sequence of Steps   | Yes      | A list of sequential steps to execute within the job.                    |

---

### `steps` item

| Field  | Type   | Required | Description                                                              |
|--------|--------|----------|--------------------------------------------------------------------------|
| `name` | String | No       | A human-readable name for the step.                                      |
| `run`  | String | Yes      | The shell command to execute for this step. This is the only action field currently implemented by the `ProcessExecutor`. |

*Note: The fields `id`, `uses`, `with`, `env`, `workspaces`, and `retryStrategy` are defined in the schema structs but are not used by the `ProcessExecutor` and have no effect on execution at this time.*

---

## 2. Guaranteed-to-Run Sample Flow

The following YAML file uses only the fields and structures that are confirmed to be implemented and executable by the current version of the `ferri` orchestrator. This example defines a simple workflow with two jobs that run sequentially, each executing a basic `echo` command.

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

---

## 3. How to Run This Flow

Follow these steps to set up a new `ferri` project and execute the sample flow.

### Step 1: Create a Project Directory

First, create a new directory for your project and navigate into it.

```sh
mkdir my-ferri-project
cd my-ferri-project
```

### Step 2: Initialize Ferri

Initialize a new `ferri` project in the directory. This will create the hidden `.ferri` subdirectory where configuration and job data are stored.

```sh
ferri init
```

### Step 3: Create the Flow File

Save the "Guaranteed-to-Run Sample Flow" YAML content from above into a new file named `sample-flow.yml` inside your project directory.

### Step 4: Run the Flow

Execute the flow using the `ferri flow run` command, pointing to the file you just created.

```sh
ferri flow run sample-flow.yml
```

You will see output from the orchestrator as it executes each job and step in sequence.

---

## 4. Advanced Flow: Using a Local Model and `sed`

This example demonstrates a more complex workflow that uses a local AI model to generate content and then processes that content in a subsequent job.

### Step 1: Add a Local Model

First, ensure you have a local model running (e.g., via `ollama run gemma`). Then, register it with `ferri` under the alias `gemma`.

```sh
ferri models add gemma --provider ollama --model-name gemma
```

### Step 2: Create the Advanced Flow File (Corrected)

Create a new file named `poem-flow.yml` and add the following content.

**Important:** This version has been corrected. The `run` command for the first step now uses an absolute path to the `ferri` binary that was just compiled (`/Users/jorgeajimenez/repos/ferri/gemini-worktree/target/debug/ferri`). This is necessary because your system's `PATH` may contain an older version of `ferri` that doesn't include the bug fix for file output. Using the absolute path ensures you are running the corrected version.

The flow has three jobs:
1.  `write-poem`: Calls the `gemma` model to write a poem and saves it to `poem.txt`.
2.  `edit-poem`: Waits for the first job, then uses `sed` to replace "love" with "code" (case-insensitively) and saves the result to `coded-poem.txt`.
3.  `display-result`: Waits for the second job and then prints the final, edited poem to the console.

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

### Step 3: Run the Advanced Flow

Now, execute the new flow. `ferri` will orchestrate the jobs, ensuring they run in the correct order based on the `needs` dependencies.

```sh
ferri flow run poem-flow.yml
```
