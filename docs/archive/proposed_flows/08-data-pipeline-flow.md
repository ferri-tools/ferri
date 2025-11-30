# 08: Data Processing Pipeline with Staged Workspaces

This flow demonstrates how to use multiple workspaces to represent the different stages of a data processing pipeline. Each workspace holds the data in a specific state (raw, processed, analyzed), making the flow's logic clear and modular.

## How It Works

1.  **Three Distinct Workspaces:**
    -   `raw-data`: Holds the initial, untouched dataset.
    -   `processed-data`: Holds the data after cleaning and transformation.
    -   `final-report`: Holds the final output of the analysis.

2.  **`download-dataset` Job:**
    -   This job simulates downloading a large dataset.
    -   It mounts the `raw-data` workspace and populates it with a sample `dataset.csv`.

3.  **`process-data` Job:**
    -   This job performs the core transformation logic.
    -   It mounts the `raw-data` workspace as **read-only** to ensure the original data is never modified.
    -   It mounts the `processed-data` workspace as read-write.
    -   It reads from `/data/raw`, processes the data (simulated with `grep`), and writes the result to `/data/processed`.

4.  **`generate-report` Job:**
    -   This final job performs the analysis.
    -   It mounts the `processed-data` workspace as **read-only**.
    -   It mounts the `final-report` workspace as read-write.
    -   It uses an AI model (`gemma`) to analyze the processed data and write a summary to the `final-report` workspace.

## State Management

-   **Workspaces as Data Stages:** This is the core concept. The workspaces aren't just folders; they represent the state of the data as it moves through the pipeline. This makes the flow's purpose self-documenting.
-   **Immutability and Safety:** The use of `readOnly: true` is a critical feature. It guarantees that the `process-data` job cannot accidentally corrupt the original raw data, and the `generate-report` job cannot alter the processed data. This enforces a one-way data flow, which is a best practice in data engineering.

## How to Run

This flow uses the `gemma` model via Ollama.

### Prerequisites

1.  **Install and run Ollama:** [https://ollama.com/](https://ollama.com/)
2.  **Pull the gemma model:** `ollama pull gemma:2b`

### Execution

The following commands are fully self-contained. They will create a temporary workspace, configure the necessary models, and then run the flow.

```bash
# 1. Create a temporary directory and navigate into it.
mkdir -p /tmp/flow-tests/08-data-pipeline && cd /tmp/flow-tests/08-data-pipeline

# 2. Initialize a new ferri workspace.
ferri init

# 3. Add the required model to the workspace's registry.
ferri models add gemma --provider ollama --model-name gemma:2b

# 4. Create the flow YAML file in the current directory.
cat <<'EOF' > 08-data-pipeline-flow.yml
# This flow demonstrates a multi-stage data pipeline where each stage is
# represented by a distinct workspace, ensuring data integrity.
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: staged-data-processing-pipeline
spec:
  workspaces:
    - name: raw-data
    - name: processed-data
    - name: final-report

  jobs:
    download-dataset:
      name: "Download Raw Dataset"
      steps:
        - name: "Simulate downloading a CSV"
          workspaces:
            - name: raw-data
              mountPath: data/raw
          run: |
            mkdir -p data/raw
            echo "--- Downloading data ---"
            echo -e "id,category,value\n1,A,100\n2,B,150\n3,A,200" > data/raw/dataset.csv
            echo "Download complete."

    process-data:
      name: "Process and Clean Data"
      needs:
        - download-dataset
      steps:
        - name: "Filter for category A"
          workspaces:
            - name: raw-data
              mountPath: data/raw
              readOnly: true # Enforce immutability of raw data
            - name: processed-data
              mountPath: data/processed
          run: |
            mkdir -p data/processed
            echo "--- Processing data ---"
            grep "A" data/raw/dataset.csv > data/processed/category_a.csv
            echo "Processing complete."

    generate-report:
      name: "Generate Final Report"
      needs:
        - process-data
      steps:
        - name: "Use AI to summarize the processed data"
          workspaces:
            - name: processed-data
              mountPath: data/processed
              readOnly: true
            - name: final-report
              mountPath: report
          run: |
            mkdir -p report
            echo "--- Generating report ---"
            ferri ctx add data/processed/category_a.csv
            ferri with --ctx --model gemma --output report/summary.md -- "Analyze the provided CSV data and summarize its key findings."
            echo "Report generation complete."
EOF

# 5. Run the flow.
ferri flow run 08-data-pipeline-flow.yml
```