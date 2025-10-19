# How to Debug `ferri` with RustRover

This guide provides step-by-step instructions for setting up the RustRover debugger to troubleshoot the `ferri with` command, which is the part of your flow that is currently failing.

We will configure the debugger to run the exact command from your flow's `run` step, allowing us to pause execution and inspect the data being returned from your local `gemma` model.

---

## Part 1: Set the Breakpoint

A breakpoint tells the debugger where to pause the program. We need to pause it right after we get the response from the Ollama API, so we can see the raw JSON.

1.  **Open the File:** In RustRover, open the project file:
    *   `ferri-cli/src/main.rs`

2.  **Find the Line:** Scroll down or use the search function (`Cmd+F` or `Ctrl+F`) to find the `Commands::With` block. Inside that, find the `PreparedCommand::Remote` block. Locate this specific line of code:

    ```rust
    let body = response.text().unwrap_or_default();
    ```

3.  **Set the Breakpoint:** Click in the empty space (the "gutter") to the left of the line number. A red dot will appear. This is your breakpoint.

    ![Setting a Breakpoint](https://i.imgur.com/gW7G3g0.png)

---

## Part 2: Create the Debug Configuration

Next, we need to tell RustRover exactly what command to run and how to run it.

1.  **Open Configurations:** In the top-right corner of the RustRover window, click the dropdown menu that shows your project name (it might say "ferri" or "Build") and select **"Edit Configurations..."**.

    ![Edit Configurations](https://i.imgur.com/sPGd31f.png)

2.  **Add New Cargo Command:**
    *   In the "Run/Debug Configurations" window, click the **`+`** button in the top-left.
    *   Select **"Cargo"** from the list.

3.  **Fill in the Details:** A new configuration form will appear. Fill it out with the following information:

    *   **Name:** Give it a descriptive name, like `Debug: ferri with poem`.

    *   **Command:** This field tells Cargo what to run and which arguments to pass to your program. The arguments for your program go at the end, after a `--` separator. Copy and paste the entire line into this single field:
        ```
        run --package ferri-cli --bin ferri -- with --model gemma --output poem.txt -- "write a short poem about love"
        ```

    *   **Working directory:** Click the folder icon and select the `flowy` directory where your `poem-flow.yml` is located. This is critical so that `ferri` can find its `.ferri` configuration.

    *   **Environment variables:** Click the folder icon to the right of this field. In the new window, click the `+` and add a new variable:
        *   Name: `RUST_BACKTRACE`
        *   Value: `1`
        (This gives you more detailed error messages if the program panics.)

    Your final configuration should look like this:

    ![Final Configuration](https://i.imgur.com/9aZgY3B.png)

4.  **Save:** Click **"Apply"** and then **"OK"**.

---

## Part 3: Run the Debugger

Now you are ready to start debugging.

1.  **Select the Configuration:** The configuration you just created (`Debug: ferri with poem`) should now be selected in the dropdown at the top-right.

2.  **Start Debugging:** Click the **bug icon** next to the configuration dropdown.

    ![Debug Button](https://i.imgur.com/Yf2aW3c.png)

3.  **Execution Pauses:** The debugger will start, build the code, and then run it. It will automatically stop at the red breakpoint you set in Part 1. The "Debug" tool window will appear at the bottom of RustRover, and the line in your editor will be highlighted.

---

## Part 4: Get the JSON Response

1.  **Find the Variables Pane:** In the "Debug" tool window at the bottom, you will see a "Variables" pane. This shows all the variables that are currently in scope.

2.  **Inspect `body`:** Find the variable named `body`. It will have the raw JSON response from Ollama as its value.

3.  **Copy the Value:** Right-click on the `body` variable and select **"Copy Value"**.

    ![Copy Value](https://i.imgur.com/uN3bJ2C.png)

4.  **Paste into Chat:** Paste the copied JSON content back into our chat.

Once I have this JSON, I can give you the exact code fix needed to correctly parse the poem from it.
