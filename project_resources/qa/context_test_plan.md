# Manual QA Test Plan: `ctx` Command

This document outlines the manual testing steps for the `ferri ctx` command.

**Prerequisites:**
1.  A `ferri` project must be initialized. If you haven't done so, run `ferri init` in your test directory.

---

### Test Case 1: `ferri ctx ls` (Empty Context)

**Goal:** Verify that listing an empty context works correctly.

**Steps:**

1.  **Navigate to your initialized test project directory.**
2.  **Run the list command:**
    ```bash
    ferri ctx ls
    ```
3.  **Verify the Output:**
    *   You should see the exact message: `Context is empty.`

---

### Test Case 2: `ferri ctx add` and `ls` (Populated Context)

**Goal:** Verify that adding items to the context and listing them works correctly.

**Steps:**

1.  **Add multiple paths:**
    *   From your test directory, run the following command. Note that these files/directories do not need to actually exist for this test.
    ```bash
    ferri ctx add README.md src/components/Button.js ./styles/
    ```
2.  **Verify the Output:**
    *   You should see a success message, for example: `Successfully added 3 path(s) to context.`

3.  **List the context:**
    *   Run the list command again:
    ```bash
    ferri ctx ls
    ```
4.  **Verify the Output:**
    *   The output should now be a list of the items you added:
        ```
        Current context:
        - README.md
        - src/components/Button.js
        - ./styles/
        ```

---

### Test Case 3: `ferri ctx add` (Handling Duplicates)

**Goal:** Verify that the tool does not add duplicate entries to the context.

**Steps:**

1.  **Add a path that already exists in the context:**
    ```bash
    ferri ctx add README.md new-file.css
    ```
2.  **Verify the Output:**
    *   You should see a success message for adding the paths.

3.  **List the context:**
    *   Run the list command one more time:
    ```bash
    ferri ctx ls
    ```
4.  **Verify the Output:**
    *   The list should now contain `new-file.css`, but `README.md` should only appear **once**. The final list should have 4 items.
        ```
        Current context:
        - README.md
        - src/components/Button.js
        - ./styles/
        - new-file.css
        ```

---
