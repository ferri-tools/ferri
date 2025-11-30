# 01: Basic Text Analysis Flow

This flow demonstrates the most fundamental concept of chaining jobs: the output of one job serves as the input for the next. It builds upon the `gemma-flow.yml` example by replacing the simple `sed` command with a standard text analysis tool, `wc` (word count).

## How It Works

1.  **`write-poem` Job:** This job uses `ferri with` to call the `gemma` model, asking it to write a short poem. Crucially, it uses the `--output` flag to save the result to a file named `poem.txt`. This file is the **intermediate artifact**.

2.  **`analyze-poem` Job:** This job has a `needs` dependency on `write-poem`, ensuring it only runs after the poem has been written. Its step then uses the standard `wc` command to count the lines, words, and characters in `poem.txt`.

## State Management

-   **Mechanism:** The state (the poem) is passed from the first job to the second by writing it to a file (`poem.txt`) in the shared workspace. The second job can then read from this file.
-   **No `yank` Needed:** It's important to note that `ferri yank` is a tool for a *user* to retrieve artifacts *after* a run is complete. It is **not** used for communication *between jobs* within a flow. Jobs in a flow share the same working directory, so they can communicate simply by reading and writing files.

## How to Run

This flow uses the `gemma` model via Ollama.

### Prerequisites

1.  **Install and run Ollama:** [https://ollama.com/](https://ollama.com/)
2.  **Pull the gemma model:** `ollama pull gemma:2b`

### Execution

The following commands are fully self-contained. They will create a temporary workspace, configure the necessary models, and then run the flow.

```bash
# 1. Create a temporary directory and navigate into it.
mkdir -p /tmp/flow-tests/01-text-analysis && cd /tmp/flow-tests/01-text-analysis

# 2. Initialize a new ferri workspace.
ferri init

# 3. Add the required model to the workspace's registry.
ferri models add gemma --provider ollama --model-name gemma:2b

# 4. Create the flow YAML file in the current directory.
cat <<'EOF' > 01-text-analysis-flow.yml
# This flow generates a piece of text and then runs a standard shell command to analyze it.
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: text-generator-and-analyzer
spec:
  jobs:
    write-poem:
      name: "Write Poem"
      steps:
        - name: "Use Gemma to write a poem"
          run: "ferri with --model gemma --output poem.txt -- 'write a short poem about the command line'"

    analyze-poem:
      name: "Analyze Poem with wc"
      needs:
        - write-poem
      steps:
        - name: "Count the lines, words, and characters in the poem"
          run: "wc poem.txt"
EOF

# 5. Run the flow.
ferri flow run 01-text-analysis-flow.yml
```

After the run, you can inspect the generated `poem.txt` in `/tmp/flow-tests/01-text-analysis`.