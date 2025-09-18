# Manual QA Test Plan: Multimodal `with` Command

This document provides a step-by-step walkthrough to manually test `ferri`'s ability to handle multimodal context (images and text) with the `with` command.

**Prerequisites:**
1.  The latest version of `ferri` is installed globally (`cargo install --path ferri-cli`).
2.  You are in a clean test directory (e.g., `~/ferri-multimodal-test`).
3.  The `ferri init` command has been run in this directory.
4.  You have a Google AI API key.

---

### Test Case 1: Image and Text Context Injection

**Goal:** Verify that `ferri` can send an image and a text prompt to a remote model and receive a correct, context-aware response.

**Steps:**

1.  **Initialize a project:**
    ```bash
    mkdir ~/ferri-multimodal-test
    cd ~/ferri-multimodal-test
    ferri init
    ```

2.  **Set the Google API Key Secret:**
    *   Replace `"your-api-key-here"` with your actual key.
    ```bash
    ferri secrets set GOOGLE_API_KEY "your-api-key-here"
    ```

3.  **Register a Multimodal Model:**
    *   Register the Gemini 1.5 Flash model, which is capable of processing images.
    ```bash
    ferri models add gemini-flash --provider google --api-key-secret GOOGLE_API_KEY --model-name gemini-1.5-flash-latest
    ```
    *   Verify the model was added:
    ```bash
    ferri models ls
    ```

4.  **Create a Sample Image:**
    *   This test requires an image. You can use any `.jpg`, `.png`, or `.webp` file.
    *   For this example, save an image of a cat and name it `cat_photo.jpg` in your test directory.

5.  **Create a Sample Text File:**
    *   Create a new file named `instructions.txt` and paste the following text into it.
    ```text
    Your primary goal is to identify the main subject in the image. Your secondary goal is to guess its name. Based on the image, a good name would be "Whiskers".
    ```

6.  **Add Files to Context:**
    *   Add both the image and the text file to the `ferri` context.
    ```bash
    ferri ctx add cat_photo.jpg
    ferri ctx add instructions.txt
    ```
    *   Verify they were added:
    ```bash
    ferri ctx ls
    ```

7.  **Run the `with` Command:**
    *   Execute the `with` command, using the `--ctx` flag to send the image and text file. The prompt will ask the model to follow the instructions from the text file.
    ```bash
    ferri with --model gemini-flash --ctx -- "Follow the instructions in the provided document to analyze the image."
    ```

8.  **Verify the Output:**
    *   The command should execute without any errors.
    *   The output from the model should be a response that both identifies the cat in the image and suggests the name "Whiskers", as specified in `instructions.txt`.

This confirms that the model correctly received and processed both the image and the text context.