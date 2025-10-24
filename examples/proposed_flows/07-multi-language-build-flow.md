# 07: Multi-Language Build & Integration Flow

This flow demonstrates a common real-world scenario: a project with separate frontend and backend components that need to be built independently and then integrated into a single distributable package. It highlights how multiple parallel jobs can contribute artifacts to a single, shared workspace.

## How It Works

1.  **`build-output` Workspace:** A single workspace is defined to act as the final distribution directory (like a `dist` or `target` folder).

2.  **`build-frontend` and `build-backend` Jobs:**
    -   These two jobs run in parallel, simulating completely different build environments (e.g., one using Node.js, the other using Rust).
    -   Both jobs mount the **same `build-output` workspace** at `/dist`.
    -   The frontend job creates a `/dist/static` directory and populates it with assets.
    -   The backend job creates a `/dist/app.bin` executable.
    -   This shows how independent processes can build up a shared directory structure concurrently.

3.  **`package-application` Job:**
    -   This final job `needs` both build jobs to be complete.
    -   It mounts the now-populated `build-output` workspace.
    -   It runs `ls -R` to verify that the artifacts from **both** the frontend and backend jobs are present in the correct locations. It then creates a final `package.tar.gz`, simulating the creation of a final deployable artifact.

## State Management

-   **Workspace as a Shared Volume:** This example treats the workspace like a shared network drive or a mounted volume that multiple, independent machines (the jobs) can write to.
-   **Atomic Contributions:** Each parallel job contributes its own files to a specific subdirectory within the workspace. This is a good practice to avoid file collisions between parallel jobs.
-   **Integration Point:** The final job acts as the integration point, consuming the complete, aggregated contents of the workspace.
