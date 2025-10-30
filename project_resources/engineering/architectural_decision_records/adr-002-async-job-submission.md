# ADR-002: Asynchronous Job Submission in Agentic Flows

**Date:** 2025-10-26

**Status:** Proposed

## Context

The `ferri do` command operates within a Tokio asynchronous runtime to manage its TUI and communication with the Gemini API. The final step of this process involves calling `jobs::submit_job` to execute the generated Ferri flow.

The `jobs::submit_job` function is a synchronous, blocking operation. It spawns a new OS process for the `ferri flow run` command and waits for it to start.

When this blocking function is called directly from the async context of the `ferri do` command's runtime, it blocks the main thread. This is a classic anti-pattern in async programming and is the root cause of the "spawning command on main thread" warning, leading to instability, hangs, and potential deadlocks.

## Decision

To resolve this conflict, we will decouple the synchronous job submission from the asynchronous agent runtime.

The `generate_and_run_flow` function in `ferri-agent/src/agent.rs` will be modified to wrap the call to `jobs::submit_job` within a `tokio::task::spawn_blocking` block.

This moves the blocking operation to a dedicated thread pool managed by the Tokio runtime, freeing the main async thread to continue its work (like managing the TUI) without being blocked.

## Consequences

**Positive:**
-   Resolves the "spawning command on main thread" issue, fixing the hangs and instability in the `ferri do` command.
-   Brings the agent's code into alignment with async best practices.
-   Improves the overall robustness and performance of the agentic engine.

**Negative:**
-   Slightly increases the complexity of the `generate_and_run_flow` function by introducing `spawn_blocking`. However, this is a standard and well-understood pattern for handling blocking code in async Rust.
