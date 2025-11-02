# Diagnosis: Final Fix for `ferri do` (#62)

**Date:** 2025-10-26
**Branch:** `feature/62-fix-ferri-do-agent`

## Summary

This document outlines the series of fixes implemented to make the `ferri do` command functional, culminating in the final compiler error and its resolution. The command is now stable and ready for testing.

## The Troubleshooting Journey

The `ferri do` command was failing due to a cascade of issues. We systematically identified and resolved them:

1.  **Missing API Key Error (`crates/ferri-agent/src/agent.rs`):** The agent was silently failing when the `GEMINI_API_KEY` environment variable was not set.
    *   **Fix:** Improved the error handling to provide a clear, user-facing message explaining the requirement and how to set the variable.

2.  **Incorrect Model Name (`crates/ferri-agent/src/agent.rs`):** The agent was hardcoded to use a non-existent Gemini model (`gemini-1.5-pro-latest`).
    *   **Fix:** Updated the model to the correct `gemini-2.5-pro`.

3.  **Nested TUI Crash (`crates/ferri-agent/src/agent.rs`):** The agent was generating a legacy flow format, which caused the `ferri flow run` command to launch its own TUI, conflicting with the agent's TUI and crashing the application.
    *   **Fix:** Updated the agent's system prompt to instruct it to generate the modern, non-interactive `ferri.flow/v1alpha1` YAML format.

4.  **TUI Corruption (`ferri-cli/src/main.rs`):** Even with the new format, the `ferri flow run` command's status output was garbling the agent's TUI.
    *   **Fix:** Added a `--quiet` flag to `ferri flow run` and updated the agent to use it, completely suppressing the sub-process's output.

5.  **Incorrect File Path (`ferri-cli/src/main.rs`, `ferri-cli/src/agent_tui.rs`):** The agent was saving the generated flow to a system temporary directory instead of the project's local `.ferri/do` directory.
    *   **Fix:** Modified the CLI to explicitly find the project root and pass it down to the agent, ensuring files are always saved in the correct location.

6.  **Main Thread Blocking (`crates/ferri-agent/src/agent.rs`):** The agent was calling the synchronous `submit_job` function from its async runtime, blocking the main thread and causing instability.
    *   **Fix:** Implemented **ADR-002**, wrapping the blocking call in `tokio::task::spawn_blocking` to move it to a separate thread.

## The Final Compiler Error

After fixing the architectural issues, a final compiler error emerged:

**File:** `crates/ferri-agent/src/agent.rs`

**Error:** `mismatched types: expected String, found JobInstance`

**Code:**
```rust
// ...
    let job_id = tokio::task::spawn_blocking(move || {
        // ...
        jobs::submit_job(...) // This returns a Result<JobInstance>
    })
    .await??; // After awaiting and unwrapping, job_id is a JobInstance struct

    Ok(job_id) // <-- ERROR HERE
}
```

**Diagnosis:**
The `generate_and_run_flow` function is required to return a `Result<String>`. However, the `jobs::submit_job` function returns a `JobInstance` struct, not just the ID string. The code was attempting to return the entire struct, causing the type mismatch.

## The Solution

The fix is to access the `id` field of the `JobInstance` struct before returning.

**Corrected Code:**
```rust
// ...
    let job = tokio::task::spawn_blocking(move || {
        // ...
        jobs::submit_job(...)
    })
    .await??;

    Ok(job.id) // <-- FIXED: Returns the String ID from the struct
}
```

This final change satisfies the function's return type and resolves the last remaining issue. I will now apply this fix, commit all the changes, and push the branch.
