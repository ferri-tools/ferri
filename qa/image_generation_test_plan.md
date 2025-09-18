# QA Test Plan: Image Generation

**Objective:** Verify that `ferri` can correctly handle image generation from remote models using the `ferri with` command.

---

## Prerequisites

1.  **Initialize Ferri:** Ensure you are in a directory with an initialized Ferri project.
    ```bash
    ferri init
    ```

2.  **Set Google API Key:** You must have a valid Google API key capable of accessing an image generation model (e.g., a version of Gemini Pro that supports it).
    ```bash
    # Replace "your-google-api-key" with your actual key
    ferri secrets set GOOGLE_API_KEY "your-google-api-key"
    ```

3.  **Add Image Generation Model:** Register a model that can generate images. This test plan assumes you are using a Gemini model.
    ```bash
    ferri models add gemini-image-generator \
      --provider google-gemini-image \
      --api-key-secret GOOGLE_API_KEY \
      --model-name gemini-2.5-flash-image-preview
    ```
    *(Note: The `gemini-2.5-flash-image-preview` model name is used as an example. Use the correct model name for image generation provided by Google.)*

---

## Test Cases

### Test Case 1: Successful Image Generation with `ferri with`

**Objective:** Verify that a remote model can generate an image and save it to a specified file.

**Command:**
```bash
ferri with --model gemini-image-generator --output "cat_photo.png" -- "a photorealistic picture of a cat"
```

**Expected Result:**
1.  The command executes without any errors printed to the console.
2.  A message is printed confirming the image was saved: `Successfully saved image to cat_photo.png`.
3.  A new file named `cat_photo.png` is created in the current directory.
4.  Opening `cat_photo.png` reveals a valid image of a cat.

---

### Test Case 2: Text Generation Still Functions Correctly

**Objective:** Verify that standard text generation is not broken by the image handling changes.

**Command:**
```bash
ferri with --model gemini-image-generator -- "what is the capital of France?"
```

**Expected Result:**
1.  The command executes without error.
2.  The console prints a text response similar to: `The capital of France is Paris.`
3.  No files are created.

---

### Test Case 3: Image Generation Without `--output` Flag

**Objective:** Verify that the tool provides a graceful error or message when an image is generated but no output path is provided.

**Command:**
```bash
ferri with --model gemini-image-generator -- "a photorealistic picture of a dog"
```

**Expected Result:**
1.  The command executes.
2.  The expected behavior is that no image file is saved. The console should either print nothing or a message indicating that image data was received but no output file was specified.
3.  The command should not crash or print a stack trace. An ideal error message would be: `Error: Could not extract text or image data from API response.` followed by the raw response body.

---

