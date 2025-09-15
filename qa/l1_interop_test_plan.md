# Manual QA Test Plan: L1 Command Interoperability

This document outlines a manual, end-to-end test case to verify that all the core L1 commands (`init`, `secrets`, `models`, `ctx`, `with`) work together seamlessly for a realistic remote AI model use case.

**Prerequisites:**
1.  You must have a valid Google AI (Gemini) API key.
2.  You must have `curl` installed on your system.
3.  Be in a clean test directory (e.g., `~/ferri-e2e-test`).

---

### The Goal

We will use `ferri` to construct and send a request to the Gemini API. We will use:
- `ferri secrets` to store our API key.
- `ferri ctx` to define a file that will be our prompt.
- `ferri with` to execute a `curl` command that uses both the secret key and the context file.

---

### Test Steps

**1. Initialize the Project**
```bash
ferri init
```
*Verification: Should see the success message.*

**2. Store the API Key**
*   Replace `YOUR_GOOGLE_API_KEY` with your actual key.
```bash
ferri secrets set GOOGLE_API_KEY "YOUR_GOOGLE_API_KEY"
```
*Verification: Should see the "Secret... set successfully" message.*

**3. Create the Prompt File**
*   We will create a simple text file that contains our prompt.
```bash
echo "Explain the significance of the Rust programming language in 5 words." > my_prompt.txt
```
*Verification: Check the file content with `cat my_prompt.txt`.*

**4. Add the Prompt File to the Context**
```bash
ferri ctx add my_prompt.txt
```
*Verification: Should see the "Successfully added 1 path(s)" message. You can also run `ferri ctx ls` to confirm.*

**5. Execute the `curl` Command with `ferri with`**
*   This is the final step that ties everything together. We will run a `curl` command.
*   `ferri with` will automatically inject `$GOOGLE_API_KEY` into the command's environment.
*   `ferri with` will also take the content from `my_prompt.txt` and prepend it to the final argument of our command (the `{"contents":...}` JSON string).

*   **Copy and paste this entire command block into your terminal:**
```bash
ferri with -- \
  curl "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-pro-latest:generateContent?key=$GOOGLE_API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"contents":[{"parts":[{"text": " "}]}]}'
```

### Verification

If successful, you will see a JSON response directly from the Google Gemini API in your terminal. The response should contain an answer similar to this:

```json
{
  "candidates": [
    {
      "content": {
        "parts": [
          {
            "text": "Memory-safe, concurrent, fast, reliable."
          }
        ],
        "role": "model"
      },
      // ... more fields
    }
  ],
  // ... more fields
}
```

Seeing this response confirms that:
- `ferri secrets` correctly stored and decrypted the key.
- `ferri with` correctly injected the key as an environment variable (`$GOOGLE_API_KEY`).
- `ferri ctx` correctly identified the context file.
- `ferri with` correctly read the context file and injected its content into the `curl` command's data payload.

This completes the end-to-end test of the L1 toolkit.
