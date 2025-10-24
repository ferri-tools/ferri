# 06: Workspace-Based Pipeline Flow

This flow demonstrates the use of the `workspaces` feature, a formal mechanism for declaring and sharing data between jobs. While the current local runner executes all jobs in a shared directory anyway, this syntax is crucial for future-proofing flows for containerized or otherwise isolated execution environments.

## How It Works

This flow simulates a standard software build-and-test pipeline.

1.  **Top-Level `workspaces` Definition:** At the `spec` level, two workspaces are declared:
    -   `source-code`: Intended to hold the application's source.
    -   `build-artifacts`: Intended to hold the compiled output.

2.  **`build-application` Job:**
    -   This job simulates checking out source code and compiling it.
    -   It mounts the `source-code` workspace at `/app/src` and writes a dummy file there.
    -   It mounts the `build-artifacts` workspace at `/app/dist` and creates a simulated binary file.

3.  **`test-application` Job:**
    -   This job depends on the `build` job.
    -   It mounts **both** workspaces to simulate a real testing environment.
    -   It needs the source code (`/app/src`) to run static analysis or linting.
    -   It needs the compiled binary (`/app/dist/app.bin`) to run integration tests.
    -   Crucially, it mounts the `source-code` workspace as `readOnly: true`, demonstrating a key security feature. The test job can read the source, but it cannot modify it.

## State Management

-   **Formal Declaration:** Unlike the previous examples that relied on an implicit shared directory, this flow formally declares its data dependencies. It's immediately clear that the `build` job *produces* artifacts and the `test` job *consumes* them.
-   **Structured Access:** The `mountPath` provides a stable, predictable filesystem layout within each job, regardless of where the runner is actually storing the data on the host machine. This is essential for portability.
