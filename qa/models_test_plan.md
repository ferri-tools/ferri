# Manual QA Test Plan: `models` Command

This document outlines the manual testing steps for the `ferri models` command.

**Prerequisites:**
1.  A `ferri` project must be initialized. If you haven't done so, run `ferri init` in your test directory.
2.  (Optional but Recommended) Have [Ollama](https://ollama.com/) running on your local machine with at least one model pulled (e.g., `ollama run llama3`) to test the auto-discovery feature.
3.  You will need a Google AI API key to test the Gemini integration. You can get one from the [AI & Machine Learning page](https://console.cloud.google.com/projectselector2/apis/dashboard) in the Google Cloud Console.

---

### Test Case 1: Listing Discovered Models (Ollama)

**Goal:** Verify that `ferri` automatically discovers and lists running Ollama models.

**Steps:**

1.  **Ensure Ollama is running:** Make sure the Ollama application is active on your machine.
2.  **Run the list command:**
    ```bash
    ferri models ls
    ```
3.  **Verify the Output:**
    *   The output table should include any models you have pulled in Ollama. They will be marked with `(discovered)`. For example:
        ```
        ALIAS                PROVIDER        ID/NAME              TYPE
        llama3:latest        ollama          llama3:latest        (discovered)
        ```

---

### Test Case 2: Adding and Listing a Remote Model (Gemini)

**Goal:** Verify that a user can securely add their Google Gemini API key and register the model with `ferri`.

**Steps:**

1.  **Store your API Key:**
    *   Use the `secrets set` command to securely store your Google API key.
    ```bash
    ferri secrets set GOOGLE_API_KEY "YOUR_API_KEY_HERE"
    ```
    *   You should see a success message.

2.  **Add the Gemini Model:**
    *   Now, register the Gemini model, linking it to the secret you just stored. We'll give it the alias `gemini-pro`.
    ```bash
    ferri models add gemini-pro --provider google --model-name gemini-1.5-pro --api-key-secret GOOGLE_API_KEY
    ```
    *   You should see the message: `Model 'gemini-pro' added successfully.`

3.  **Verify the Model is Listed:**
    *   Run the list command again:
    ```bash
    ferri models ls
    ```
    *   The output should now include your newly registered Gemini model, without the `(discovered)` tag.
        ```
        ALIAS                PROVIDER        ID/NAME              TYPE
        llama3:latest        ollama          llama3:latest        (discovered)
        gemini-pro           google          gemini-1.5-pro
        ```

---

### Test Case 3: Removing a Model

**Goal:** Verify that a user can remove a model they have added.

**Steps:**

1.  **Remove the Gemini model:**
    ```bash
    ferri models rm gemini-pro
    ```
    *   You should see the message: `Model 'gemini-pro' removed successfully.`

2.  **Verify the Model is Gone:**
    *   Run the list command one last time:
    ```bash
    ferri models ls
    ```
    *   The output should no longer contain the `gemini-pro` model.

---
