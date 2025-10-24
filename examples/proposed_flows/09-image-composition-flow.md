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
