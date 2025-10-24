# 04: Parallel Processing and Merge Flow

This flow demonstrates how to run jobs in parallel and then aggregate their results in a final "merge" job. This is a common and efficient pattern for tasks that can be broken down into independent pieces of work, such as data processing, testing, or batch analysis.

## How It Works

1.  **`generate-pros` and `generate-cons` Jobs:** These two jobs are defined with no dependencies, so the Ferri flow engine can run them **in parallel**.
    -   `generate-pros` asks an AI to list the advantages of using Rust and saves them to `pros.txt`.
    -   `generate-cons` asks an AI to list the disadvantages of using Rust and saves them to `cons.txt`.

2.  **`create-report` Job:** This is the "merge" or "fan-in" job.
    -   It has a `needs` block that lists both `generate-pros` and `generate-cons`. This ensures that it will only start after **both** parallel jobs have completed successfully.
    -   Its step uses the `cat` command to concatenate the two title files (`title.txt`, `pros.txt`, `cons.txt`) into a single, final report named `full_report.md`.

## State Management

-   **Multiple Artifacts:** The state is distributed across multiple files (`pros.txt`, `cons.txt`).
-   **Aggregation:** The `create-report` job is responsible for reading all the individual artifacts and combining them into a final, unified output. This pattern is highly scalable; you could have dozens of parallel jobs, and a single final job could aggregate all their results.
