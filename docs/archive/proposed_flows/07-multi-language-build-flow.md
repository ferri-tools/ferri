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

## How to Run

This flow does not require any AI models and runs entirely with shell commands.

### Prerequisites

None.

### Execution

The following commands are fully self-contained and can be run from any directory to test the flow in an isolated environment.

```bash
# 1. Create a temporary directory and navigate into it.
mkdir -p /tmp/flow-tests/07-multi-language-build && cd /tmp/flow-tests/07-multi-language-build

# 2. Initialize a new ferri workspace.
ferri init

# 3. Create the flow YAML file in the current directory.
cat <<'EOF' > 07-multi-language-build-flow.yml
# This flow simulates a multi-language build process where frontend and backend
# assets are built in parallel and placed into a single shared workspace.
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: multi-language-build-and-package
spec:
  workspaces:
    - name: build-output

  jobs:
    build-frontend:
      name: "Build Frontend (e.g., React)"
      steps:
        - name: "Compile JS and CSS"
          workspaces:
            - name: build-output
              mountPath: dist
          run: |
            mkdir -p dist
            echo "--- Building frontend assets ---"
            mkdir -p dist/static/css
            echo "/* main.css */" > dist/static/css/main.css
            mkdir -p dist/static/js
            echo "// app.js" > dist/static/js/app.js
            echo "index.html" > dist/index.html
            echo "Frontend build complete."

    build-backend:
      name: "Build Backend (e.g., Rust)"
      steps:
        - name: "Compile Rust binary"
          workspaces:
            - name: build-output
              mountPath: dist
          run: |
            mkdir -p dist
            echo "--- Building backend binary ---"
            echo "#!/bin/sh" > dist/app.bin
            echo "echo 'Backend server running'" >> dist/app.bin
            chmod +x dist/app.bin
            echo "Backend build complete."

    package-application:
      name: "Package Final Application"
      needs:
        - build-frontend
        - build-backend
      steps:
        - name: "Verify contents and create tarball"
          workspaces:
            - name: build-output
              mountPath: app
          run: |
            echo "--- Packaging application ---"
            echo "Final directory structure:"
            ls -R app
            echo ""
            echo "Creating package.tar.gz..."
            # In a real runner, we'd use tar
            # tar -czf app/package.tar.gz -C app .
            echo "Tarball created."
            echo "Packaging complete."
EOF

# 4. Run the flow.
ferri flow run 07-multi-language-build-flow.yml
```