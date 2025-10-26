# QA Test Plan: `ferri flow`

**Objective:** Verify the functionality of the `ferri flow` command, including running pipelines, handling I/O, and visualizing workflows.

---

## Prerequisites

1.  **Initialize Ferri:**
    ```bash
    ferri init
    ```

2.  **Set Secrets and Models (for relevant tests):**
    ```bash
    # For Google text models
    ferri secrets set GOOGLE_API_KEY "your-google-api-key"
    ferri models add gemini-pro --provider google --api-key-secret GOOGLE_API_KEY --model-name gemini-2.5-pro

    # For Google image models
    ferri models add gemini-image-generator --provider google-gemini-image --api-key-secret GOOGLE_API_KEY --model-name gemini-2.5-flash-image-preview
    ```

---

## Test Cases

### Test Case 1: Simple Text Processing Flow

**Objective:** Verify that a basic, multi-step pipeline using `process` steps and standard I/O redirection works correctly.

**1. Create `simple_flow.yml`:**
```yaml
name: "Simple Text Flow"
steps:
  - name: "start"
    process:
      process: "echo 'hello world'"
      output: "step1_out.txt"
  - name: "finish"
    process:
      process: "cat step1_out.txt | tr '[:lower:]' '[:upper:]'"
      output: "final_out.txt"
```

**2. Run the flow:**
```bash
ferri flow run simple_flow.yml
```

**3. Verification:**
*   Check the contents of `final_out.txt`.
*   **Expected:** The file should contain the text `HELLO WORLD`.

---

### Test Case 2: Image Generation Flow

**Objective:** Verify that a `ModelStep` can generate an image and save it to a file.

**1. Create `image_flow.yml`:**
```yaml
name: "Cat Image Generation"
steps:
  - name: "generate-image"
    model:
      model: "gemini-image-generator"
      prompt: "a watercolor painting of a cat in a library"
      outputImage: "cat_painting.png"
```

**2. Run the flow:**
```bash
ferri flow run image_flow.yml
```

**3. Verification:**
*   Check if the file `cat_painting.png` has been created.
*   Open the file to ensure it is a valid image that matches the prompt.

---

### Test Case 3: Flow Visualization (`flow show`)

**Objective:** Verify that the `ferri flow show` command can correctly parse and display a workflow.

**1. Use `simple_flow.yml` from Test Case 1.**

**2. Run the show command:**
```bash
ferri flow show simple_flow.yml
```

**3. Verification:**
*   A Text-based User Interface (TUI) should appear in the terminal.
*   The TUI should display a graph with two nodes: "start" and "finish".
*   Pressing 'q' should exit the TUI gracefully.

---