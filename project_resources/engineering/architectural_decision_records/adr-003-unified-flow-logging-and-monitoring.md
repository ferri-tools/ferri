# ADR-003: Unified Flow Logging and Monitoring

**Date:** 2025-10-27

**Status:** Proposed

## Context

Currently, the `ferri do` command and the `ferri flow run` command have different user experiences. `ferri do` provides a real-time TUI by passing a channel from the CLI to the orchestrator, while `ferri flow run` simply prints status updates to standard output. This creates an inconsistent and fragmented user experience. We need a unified mechanism for monitoring all flow executions, whether they are generated on-the-fly or run from a file.

## Decision

We will implement a log-based monitoring system for all flow executions.

1.  **Orchestrator Logging:** The `FlowOrchestrator` will no longer accept a channel for real-time updates. Instead, upon execution, it will create a unique, run-specific log file within the `.ferri/runs/` directory. It will write all status updates (JobPending, JobSucceeded, JobFailed, etc.) as structured log entries to this file.

2.  **CLI Monitoring:** Both `ferri do` and `ferri flow run` will use the same monitoring TUI (`flow_monitor_tui.rs`).
    *   The CLI will spawn the `FlowOrchestrator` in a background thread.
    *   The TUI will be launched on the main thread.
    *   The TUI will poll the corresponding run-specific log file for new entries, updating the display in real-time.

This decouples the orchestrator from the UI, simplifies the orchestrator's API, and provides a consistent, real-time monitoring experience for all types of flow executions. It also has the added benefit of creating a persistent, auditable log for every flow run.

## Consequences

**Positive:**
-   Unified and consistent user experience for `ferri do` and `ferri flow run`.
-   Decouples the execution logic (orchestrator) from the presentation logic (TUI).
-   Provides a persistent log for every flow run, which can be used for debugging and auditing.
-   Simplifies the `FlowOrchestrator`'s `execute` method signature.

**Negative:**
-   Introduces a small amount of I/O overhead due to writing and polling log files.
-   The TUI will need to be updated to handle file polling and parsing instead of receiving updates from a channel.
