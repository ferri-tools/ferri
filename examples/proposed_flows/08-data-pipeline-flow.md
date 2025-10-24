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
