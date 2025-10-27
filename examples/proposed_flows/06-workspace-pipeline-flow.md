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

## How to Run

This flow does not require any AI models and runs entirely with shell commands.

### Prerequisites

None.

### Execution

The following commands are fully self-contained and can be run from any directory to test the flow in an isolated environment.

```bash
# 1. Create a temporary directory and navigate into it.
mkdir -p /tmp/flow-tests/06-workspace-pipeline && cd /tmp/flow-tests/06-workspace-pipeline

# 2. Initialize a new ferri workspace.
ferri init

# 3. Create the flow YAML file in the current directory.
cat <<'EOF' > 06-workspace-pipeline-flow.yml
# This flow demonstrates the use of the 'workspaces' feature for explicitly managing
# shared data between jobs, preparing it for future containerized runners.
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: workspace-build-and-test-pipeline
spec:
  # Define the shared storage volumes for the entire flow.
  workspaces:
    - name: source-code
    - name: build-artifacts

  jobs:
    build-application:
      name: "Build Application"
      steps:
        - name: "Simulate checkout and compile"
          # This step mounts the two workspaces into its virtual filesystem.
          workspaces:
            - name: source-code
              mountPath: /app/src
            - name: build-artifacts
              mountPath: /app/dist
          run: |
            echo "--- Building Application ---"
            echo "print('hello from my app')" > /app/src/main.py
            echo "Build artifact created at $(date)" > /app/dist/app.bin
            echo "Source code:"
            cat /app/src/main.py
            echo "Build artifact:"
            cat /app/dist/app.bin

    test-application:
      name: "Test Application"
      needs:
        - build-application
      steps:
        - name: "Run tests on the build artifacts"
          # This job mounts the same workspaces, but with different permissions.
          workspaces:
            - name: source-code
              mountPath: /app/src
              readOnly: true # The test job cannot modify the source code.
            - name: build-artifacts
              mountPath: /app/dist
          run: |
            echo "--- Testing Application ---"
            echo "Verifying source code (read-only):"
            cat /app/src/main.py
            echo "Verifying build artifact:"
            cat /app/dist/app.bin
            echo "Test successful!"
EOF

# 4. Run the flow.
ferri flow run 06-workspace-pipeline-flow.yml
```