# Multimodal Functionality Test Plan

**Objective:** To verify that `ferri` can correctly handle multimodal context (text and images) when using the `with` command with a compatible remote model.

---

### **Test Case 1: Image and Text Context Injection**

**Description:** This test ensures that both an image file and a text file from the context are correctly processed by a remote model.

**Prerequisites:**
1.  A Google API key is set as a secret (`ferri secrets set GOOGLE_API_KEY "..."`).
2.  A multimodal-capable Google model is registered (e.g., `ferri models add gemini-flash ...`).
3.  A sample image file (e.g., `test_image.jpg`) exists.
4.  A sample text file (e.g., `test_doc.txt`) with known content exists.

**Steps:**
1.  Clear the context: `ferri ctx rm --all` (or manually remove items).
2.  Add the image to the context: `ferri ctx add test_image.jpg`.
3.  Add the text file to the context: `ferri ctx add test_doc.txt`.
4.  Verify both files are in the context: `ferri ctx ls`.
5.  Execute the `with` command, asking the model to describe the image and reference the text file:
    ```bash
    ferri with --model gemini-flash --ctx -- "Describe the image. Does it relate to the content in the document?"
    ```

**Expected Result:**
- The command executes successfully without errors.
- The output from the model should be a description of `test_image.jpg` and should also reference the content of `test_doc.txt`.

---

### **Test Case 2: Text-Only Context with Multimodal Model**

**Description:** This test verifies that the command still works correctly when only text files are in the context, even when using a multimodal model.

**Prerequisites:**
1.  Same as Test Case 1, but without the image file.

**Steps:**
1.  Clear the context.
2.  Add only the text file to the context: `ferri ctx add test_doc.txt`.
3.  Execute the `with` command:
    ```bash
    ferri with --model gemini-flash --ctx -- "Summarize the document."
    ```

**Expected Result:**
- The command executes successfully.
- The output is a summary of the content in `test_doc.txt`.

---

### **Test Case 3: No Context**

**Description:** This test ensures the `--ctx` flag is handled gracefully when the context is empty.

**Prerequisites:**
1.  Same as Test Case 1.

**Steps:**
1.  Clear the context.
2.  Verify the context is empty: `ferri ctx ls`.
3.  Execute the `with` command with the `--ctx` flag:
    ```bash
    ferri with --model gemini-flash --ctx -- "What is the capital of France?"
    ```

**Expected Result:**
- The command executes successfully.
- The output should be "Paris" or a similar correct answer, demonstrating the model call worked without any context injection.
