# 01: Basic Text Analysis Flow

This flow demonstrates the most fundamental concept of chaining jobs: the output of one job serves as the input for the next. It builds upon the `gemma-flow.yml` example by replacing the simple `sed` command with a standard text analysis tool, `wc` (word count).

## How It Works

1.  **`write-poem` Job:** This job uses `ferri with` to call the `gemma` model, asking it to write a short poem. Crucially, it uses the `--output` flag to save the result to a file named `poem.txt`. This file is the **intermediate artifact**.

2.  **`analyze-poem` Job:** This job has a `needs` dependency on `write-poem`, ensuring it only runs after the poem has been written. Its step then uses the standard `wc` command to count the lines, words, and characters in `poem.txt`.

## State Management

-   **Mechanism:** The state (the poem) is passed from the first job to the second by writing it to a file (`poem.txt`) in the shared workspace. The second job can then read from this file.
-   **No `yank` Needed:** It's important to note that `ferri yank` is a tool for a *user* to retrieve artifacts *after* a run is complete. It is **not** used for communication *between jobs* within a flow. Jobs in a flow share the same working directory, so they can communicate simply by reading and writing files.
