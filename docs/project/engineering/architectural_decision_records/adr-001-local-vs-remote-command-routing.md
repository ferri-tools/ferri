# ADR-001: Clarifying Local vs. Remote Command Execution in `ferri with`

**Date:** 2025-10-18
**Status:** Accepted

## Context

We are debugging a `ferri flow` that uses the `ferri with --model gemma` command in a step. The flow is failing because the command, which is intended to call a local Ollama model via an HTTP API, is being incorrectly routed through the "Local" command execution path. This path is meant for direct subprocesses like `sed` or `echo`, not for commands that require an API request.

This led to a crucial question: Is fixing this a fundamental change to the project's architecture? This document clarifies that it is not an architectural change, but rather a bug fix that enforces the intended architecture.

## The Architectural Decision

The `ferri with` command is designed as a "smart router" with two distinct execution paths. The choice of path is determined by the presence of the `--model` flag.

1.  **`PreparedCommand::Local` Path (Direct Execution):**
    *   **Trigger:** No `--model` flag is provided.
    *   **Intended Use:** For running standard shell commands directly as a subprocess (e.g., `sed`, `ls`, `cat`).
    *   **Behavior:** The command and its arguments are executed as-is on the local machine's shell.

2.  **`PreparedCommand::Remote` Path (API-Based Execution):**
    *   **Trigger:** The `--model` flag is provided.
    *   **Intended Use:** For any command that interacts with an AI model provider's API.
    *   **Behavior:** The system looks up the specified model's configuration. Based on the model's "provider" (e.g., `ollama`, `google`), it constructs and sends an appropriate HTTP API request.

**Clarification on Terminology:** The term "Remote" in this context refers to the *method* of execution (an API call), not necessarily the *location* of the server. A `Remote` command can, and often does, target a `localhost` server, as is the case with Ollama.

## The Bug (Corrected Diagnosis)

Initial analysis incorrectly assumed the routing logic in `prepare_command` was faulty. Interactive debugging proved this wrong.

The actual bug was in how the two architectural paths handled command output in `ferri-cli/src/main.rs`:
1.  The `prepare_command` function **correctly** identifies that a model with the `ollama` provider should be executed as a direct subprocess (`ollama run ...`) and therefore returns a `PreparedCommand::Local`. This respects the architecture.
2.  However, the `Local` command execution block in `main.rs` had a critical flaw: it took the `stdout` from the completed subprocess and printed it directly to the console, **completely ignoring the `--output` flag**.
3.  The logic for writing to a file only existed in the `Remote` command execution block, which was never being called for `ollama`.

## Consequences and Resolution

*   **This is a Bug Fix, Not an Architectural Change:** The fix does not change the architecture. It corrects a feature deficiency in the `Local` execution path to make it consistent with the `Remote` path's capabilities, thereby strengthening the original design.
*   **Action:** The bug was resolved by adding the necessary `if let Some(output_path) = ...` logic to the `PreparedCommand::Local` block in `ferri-cli/src/main.rs`. This ensures that if the `--output` flag is present, the `stdout` from the subprocess is written to the specified file, as the user expects.
