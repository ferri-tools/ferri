# Post-Alpha Regroup: Ferri Project State

This document provides a snapshot of the Ferri project's current state, its intended architecture, and a clear-eyed assessment of the discrepancies that need to be addressed to move forward.

---

## What Ferri Is (The Vision)

**Ferri is a local-first AI toolkit and automation engine for developers.**

It acts as an intelligent director for foundation models, evolving from a simple command runner into a proactive, agentic partner. Ferri creates secure, project-based environments with portable context, unifying your workflow across local (Ollama) and remote (API) models.

The goal is to let you focus on your high-level goals, not on the minutiae of context management and command execution.

---

## Project Status: Active Development & Inconsistency

**Warning:** Ferri is currently undergoing a significant overhaul. The documentation has fallen out of sync with the implementation, leading to confusion and errors. The `feature/tui-overhaul` branch contains the latest, and sometimes experimental, changes.

---

## Core Architecture

Ferri is designed in three layers:

| Layer | Command(s) | Description | Status |
|---|---|---|---|
| **L1: Core Execution** | `init`, `secrets`, `models`, `ctx`, `with` | The foundation. Manages your environment, models, and executes synchronous, single-shot commands. | ✅ Stable |
| **L2: Workflow Automation** | `run`, `ps`, `yank`, `flow` | The automation layer. Runs commands as background jobs, monitors their status, and orchestrates multi-step workflows. | ⚠️ **Under Revision** |
| **L3: Agentic Engine** | `do` | The intelligent director. Takes a high-level goal, formulates a multi-step plan, and executes it. | 🚧 Experimental |

---

## The `ferri flow` Discrepancy: `jobs` vs. `steps`

The most significant issue identified is a mismatch between the documented design and the current implementation of the `ferri flow` engine.

*   **The Problem:** The documentation and initial design used the term `jobs` to define the stages in a flow. The current parser implementation, however, expects the keyword `steps`. This is inconsistent with `ferri run`, which manages single background `jobs`.
*   **The Path Forward:** We must standardize on a single term. The consensus is to use **`jobs`** across the entire application for consistency. A `flow` is a collection of `jobs`. This work is now tracked in ticket **[T72]**.
*   **Enhancement:** The flow engine also lacks a critical feature: dependency management. To enable complex, non-linear workflows, support for a `dependencies` key is required. This work is tracked in ticket **[T73]**.

All documentation will be updated to reflect this new, consistent standard as part of ticket **[T74]**.

---

## Actionable Roadmap

To stabilize the project and align the implementation with the vision, the following high-priority tickets have been created:

1.  **[T72] Unify Execution Terminology:** Refactor the codebase to eliminate the `steps` vs. `jobs` confusion by standardizing on `jobs`.
2.  **[T73] Implement Flow Dependencies:** Enhance the flow engine to support a `dependencies` key, enabling complex, non-linear workflows.
3.  **[T74] Comprehensive Documentation Overhaul:** Bring all documentation (`README.md`, walkthroughs, etc.) in line with the corrected implementation to provide a clear and trustworthy user experience.

Once this foundation is solidified, development can accelerate on the L3 Agentic Engine and the interactive TUI.
