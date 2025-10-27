# 09: AI Image Generation & Composition Flow

This flow demonstrates a more advanced use of workspaces where a single job needs to access multiple, distinct data sources to perform a "composition" task. It simulates a multi-modal AI workflow that generates an image and then combines it with a separate asset (a watermark).

## How It Works

1.  **`generated-image` and `assets` Workspaces:** Two workspaces with different purposes are defined. One will hold the dynamically generated content, and the other will hold static assets.

2.  **`generate-image` and `download-watermark` Jobs:**
    -   These jobs run in parallel.
    -   The `generate-image` job uses an AI model to create a `cat.png` file and saves it to the `generated-image` workspace.
    -   The `download-watermark` job simulates downloading a `watermark.png` file and saves it to the `assets` workspace.

3.  **`apply-watermark` Job:**
    -   This is the core "composition" job. It `needs` both of the previous jobs to be complete.
    -   Crucially, it mounts **both** workspaces into its filesystem at different paths:
        -   `generated-image` is mounted at `/images`.
        -   `assets` is mounted at `/assets` as **read-only**.
    -   The `run` command then simulates using a tool like `imagemagick` to combine the image from the first workspace (`/images/cat.png`) with the asset from the second (`/assets/watermark.png`). The final result is written back to the `/images` workspace.

## State Management

-   **Multi-Workspace Mounting:** This is the key concept. A single job can declare its need for multiple, independent data sources, and the flow runner makes them available at specified locations. This is impossible with a single shared directory, as you couldn't guarantee the separation of concerns.
-   **Composition Pattern:** This flow is a template for any task that involves combining a dynamic element with a static one. Examples include:
    -   Applying a standard header/footer to an AI-generated report.
    -   Injecting an AI-generated code snippet into a boilerplate file.
    -   Placing a generated 3D model into a pre-defined scene.

## How to Run

This flow does not require any AI models and runs entirely with shell commands, simulating the AI steps.

### Prerequisites

None.

### Execution

The following commands are fully self-contained and can be run from any directory to test the flow in an isolated environment.

```bash
# 1. Create a temporary directory and navigate into it.
mkdir -p /tmp/flow-tests/09-image-composition && cd /tmp/flow-tests/09-image-composition

# 2. Initialize a new ferri workspace.
ferri init

# 3. Create the flow YAML file in the current directory.
cat <<'EOF' > 09-image-composition-flow.yml
# This flow demonstrates a "composition" job that mounts multiple workspaces
# to combine an AI-generated image with a static asset.
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: ai-image-composition-pipeline
spec:
  workspaces:
    - name: generated-image
    - name: assets

  jobs:
    generate-image:
      name: "Generate Image with AI"
      steps:
        - name: "Use Gemini to generate a picture of a cat"
          workspaces:
            - name: generated-image
              mountPath: output
          run: |
            mkdir -p output
            echo "--- Generating AI image ---"
            # Simulate creating an image file. A real model would do this.
            echo "IMAGE_DATA_OF_A_CAT" > output/cat.png
            echo "Image generation complete."

    download-watermark:
      name: "Download Watermark Asset"
      steps:
        - name: "Simulate downloading a watermark.png"
          workspaces:
            - name: assets
              mountPath: assets
          run: |
            mkdir -p assets
            echo "--- Downloading assets ---"
            echo "WATERMARK_PNG_DATA" > assets/watermark.png
            echo "Asset download complete."

    apply-watermark:
      name: "Apply Watermark to Image"
      needs:
        - generate-image
        - download-watermark
      steps:
        - name: "Combine image and watermark"
          # This job mounts two different workspaces to combine their contents.
          workspaces:
            - name: generated-image
              mountPath: images # Contains the dynamic content
            - name: assets
              mountPath: assets   # Contains the static assets
              readOnly: true
          run: |
            mkdir -p images
            echo "--- Applying watermark ---"
            # Simulate a tool like ImageMagick: composite watermark.png cat.png final.png
            IMAGE=$(cat images/cat.png)
            WATERMARK=$(cat assets/watermark.png)
            echo "${IMAGE}_WITH_${WATERMARK}" > images/final_watermarked_cat.png
            echo "Composition complete. Final image:"
            cat images/final_watermarked_cat.png
EOF

# 4. Run the flow.
ferri flow run 09-image-composition-flow.yml
```