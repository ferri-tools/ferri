# Manual QA Test Plan: `flow` Command AI Pipelines

This document outlines the manual testing steps for the `ferri flow` command.

**Prerequisites:**
1.  `ferri` is installed globally.
2.  Ollama is running with the `gemma:2b` model pulled.
3.  You are in a clean test directory (e.g., `~/ferri-flow-test`).
4.  The `ferri init` command has been run at the root of the project.

---

### Test Case 1: Simple Piped Workflow

**Goal:** Verify that a simple pipeline with `process` steps can pipe data from stdin to stdout.

**Steps:**

1.  **Initialize a project:**
    ```bash
    ferri init
    ```

2.  **Create a `flow.yml` file:**
    ```yaml
    # simple-pipe.yml
    name: "Simple Piped Flow"
    steps:
      - name: "add-hello"
        process:
          process: "sed 's/^/Hello, /'"
      - name: "add-excitement"
        process:
          process: "sed 's/$/!!!/'"
    ```

3.  **Run the flow with piped input:**
    ```bash
    echo "World" | ferri flow run simple-pipe.yml
    ```

4.  **Verify the Output:**
    *   You should see the final, transformed output printed to your terminal:
        ```
        Hello, World!!!
        ```

---

### Test Case 2: File and Inter-Step I/O

**Goal:** Verify that a flow can read from files, write to files, and use the output of a previous step as input.

**Steps:**

1.  **Create an input file:**
    ```bash
    echo "This is a test." > input.txt
    ```

2.  **Create a `flow.yml` file:**
    ```yaml
    # io-flow.yml
    name: "File I/O Flow"
    steps:
      - name: "read-from-file"
        process:
          process: "cat"
        input: "input.txt"
      - name: "capitalize"
        process:
          process: "tr 'a-z' 'A-Z'"
        input: "read-from-file"
        output: "output.txt"
    ```

3.  **Run the flow:**
    ```bash
    ferri flow run io-flow.yml
    ```

4.  **Verify the Output File:**
    *   Check the contents of the output file:
        ```bash
        cat output.txt
        ```
    *   The content should be the capitalized version of the input:
        ```
        THIS IS A TEST.
        ```

---

### Test Case 3: Model Step Integration

**Goal:** Verify that a `ModelStep` can be used in a pipeline and can correctly process its input.

**Steps:**

1.  **Register a model alias:**
    ```bash
    ferri models add gemma --provider ollama --model-name gemma:2b
    ```

2.  **Create a `flow.yml` file:**
    ```yaml
    # model-flow.yml
    name: "Model Integration Flow"
    steps:
      - name: "summarizer"
        model:
          model: "gemma"
          prompt: "Summarize the following text in exactly three words:"
      - name: "final-touch"
        process:
          process: "sed 's/$/... a summary./'"
    ```

3.  **Run the flow with piped input:**
    ```bash
    echo "The Rust programming language is a modern systems programming language focused on safety, speed, and concurrency. It accomplishes these goals by being memory safe without using garbage collection." | ferri flow run model-flow.yml
    ```

4.  **Verify the Output:**
    *   You should see a three-word summary from the model, followed by the text from the `sed` command. The output will be similar to this (the model's summary may vary slightly):
        ```
        Sure, here is a summary of the text in exactly three words:... a summary.
        ```

---

### Test Case 4: Flow Visualization

**Goal:** Verify that the `ferri flow show` command can visualize a workflow.

**Steps:**

1.  **Use the `io-flow.yml` file from Test Case 2.**

2.  **Run the `show` command:**
    ```bash
    ferri flow show io-flow.yml
    ```

3.  **Verify the Output:**
    *   You should see a tree-like visualization of the workflow printed to your terminal, similar to this:
        ```
        File I/O Flow
        ├─── read-from-file: Process: 'cat'
        └─── capitalize: Process: 'tr 'a-z' 'A-Z''
        ```

This confirms that the `flow` command is working correctly and is ready for your demo.
