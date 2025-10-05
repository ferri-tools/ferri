# Gemini Workflow

This document outlines the primary workflow for interacting with the Ferri project. I will use this as my guide for our sessions.

## Session Start

At the start of each session, I will:

1. Greet you with a fact about rust or python to help me learn
2.  **Ask you what you'd like to do.**

---

## What would you like to do?

**1. Check project status:** I will use `gh issue list` to analyze GitHub issues and cross-reference them with the actual implementation in the codebase to give you a summary of completed, in-progress, and pending tasks.

**2. Work on a ticket:** Tell me which ticket you want to work on (e.g., "Let's work on issue #8"). I will then focus on implementing the issue.

**3. Create a new ticket:** I will help you create a new GitHub issue with appropriate labels, milestones, and a clear description of the work required.

**4. Mark a ticket as done:** Tell me the issue number you've completed (e.g., "#5 is done"), and I will close the GitHub issue.

**5. Something else:** If you have a different task in mind, just let me know.

---

## Session Logging

At the end of a session, or when a major pivot in strategy occurs, I will write a summary of our progress to a log file.

*   **Location:** `development_resources/logs/`
*   **Filename:** `log-YYYY-MM-DD.txt`
*   **Content:** The log will include a summary of key events, strategic decisions, and especially any failures or loops I got stuck in, to ensure we learn from them.

This will create a persistent record of our work that we can refer back to.

---

## Core Reference Files

To inform our work, I will always refer to:
*   **`README.md`**: For understanding the project's architecture and goals.
*   **GitHub Issues** (via `gh issue list` and `gh issue view`): As the single source of truth for all development tasks.
*   **The source code**: To verify implementation status.

This should go without saying but always confirm if changing the .gitignore. It already happened before and it made things go crazy for a bit.

---
## Development Best Practices

To ensure a high-quality, maintainable codebase, we will adhere to the following practices:

*   **Atomic Commits:** Each commit should represent a single, logical change. Commits should reference their corresponding issue number (e.g., `fix(auth): #21 - Fix login redirect bug`).
*   **Test-Driven Development (TDD):** For new functionality, we will write a failing test *before* writing the implementation. This ensures the code is correct, testable, and meets requirements from the start.
*   **Continuous Integration:** After every commit, I will run the project's build, lint, and test suites to catch regressions early.
*   **Zero-Warning Policy:** Compiler warnings must be treated as errors and fixed immediately. A clean, warning-free build is required at all times.
*   **User-Centric Testing:** When a feature is complete, I will install the binary and provide clear instructions for you to perform the final user acceptance testing (UAT).
*   **Push Protocol:** Code will only be pushed to the remote repository (`git push`) after you have personally verified and approved the functionality. Frequent local commits will continue.

---
## Multi-Agent Collaboration

When working in a multi-agent environment, all development practices are superseded by the formal protocol defined in `AGENTS.md`. All agents, including this one, **must** adhere to that protocol to ensure synchronized, conflict-free work.

---

### **CRITICAL: Issue and Branching Protocol**

To ensure all work is tracked, isolated, and aligned with project management, the following protocol is **mandatory and non-negotiable**.

1.  **Issue First:** No work of any kind shall begin without a corresponding GitHub issue.
    *   Before any action, I will generate the appropriate `gh issue create` command.
    *   I will present this command to you for execution.
    *   I will not proceed until the issue is created.

2.  **Branch from Issue:** All work must be done on a dedicated feature branch.
    *   After an issue is created, I will immediately create a new feature branch.
    *   The branch name must follow the format: `feature/<short-description>` (e.g., `feature/refactor-auth-module`).
    *   I will use `git checkout -b <branch-name>` to create and switch to this branch.

3.  **Work in Isolation:** All subsequent file modifications, commits, and tests will occur exclusively on this feature branch. The `main` branch will not be touched directly.

This protocol is the first step in any development task. Failure to adhere to it will result in an immediate halt and reset of the workflow.

---

### **CRITICAL: The Focused Context Mandate**

To combat context bloat, ensure efficiency, and maintain focus, the following protocol is a **non-negotiable directive**. Deviating from this mandate is a primary cause of errors and inefficient performance. Adherence is mandatory.

I will approach context gathering in the following tiered manner:

1.  **Tier 1 (Core Context):** Always begin with the absolute minimum:
    *   `README.md`: For high-level architecture and project goals.
    *   GitHub Issues (via `gh issue view <number>`): To understand the specific ticket, goal, and tasks.

2.  **Tier 2 (Targeted Discovery):** With the core context established, I will use `glob` and `search_file_content` with keywords from the ticket to *identify* the most relevant files. I will not read them yet. This prevents premature context loading.

3.  **Tier 3 (Surgical Read):** I will use `read_file` to load *only* the specific, high-confidence files identified in Tier 2.

4.  **Tier 4 (Expansion by Necessity):** If and only if the surgically-read files contain direct references to other modules, functions, or files necessary to complete the task, I will deliberately read those specific files.

**Accountability Protocol:** In my planning phase for any task, I will explicitly state which context tier I am operating in and justify the files I choose to read. This makes my process transparent and auditable against this mandate.

---

### **CRITICAL: Resource Consumption Protocol**

To prevent unexpected costs, the following protocol is a **mandatory safeguard**.

1.  **Identify High-Cost Operations:** I will actively monitor for operations that are likely to consume a large number of tokens. This includes, but is not limited to:
    *   Reading multiple large files.
    *   Reading an entire directory of files.
    *   Performing overly broad code searches.
    *   Generating large volumes of code or text.

2.  **Warn and Confirm:** Before proceeding with any high-cost operation, I will:
    *   **Pause:** Stop execution before the expensive step.
    *   **Warn:** Explicitly state that the next action may have a high token cost.
    *   **Justify:** Briefly explain why the operation is necessary to achieve your goal.
    *   **Request Confirmation:** Ask for your explicit "yes" or "proceed" to continue.

This protocol places you in direct control of token-heavy operations, ensuring there are no surprises.

---

### **CRITICAL: Context Offloading Protocol**

If at any point my context becomes fragmented, lost, or if I am repeatedly failing to solve a problem, the following recovery protocol must be initiated to ensure a clean restart.

1.  **Acknowledge Context Loss:** I will explicitly state that I am low on context and need to perform a reset.

2.  **Generate a Summary:** I will produce a concise, bulleted summary of our entire session. This summary will include:
    *   The initial high-level goal.
    *   Each distinct problem or bug that was identified.
    *   The architectural changes and fixes that were implemented to address each problem.
    *   The current status and my hypothesis for the remaining issue.

3.  **Save Session to File:** I will save this summary to a dedicated file: `development_resources/logs/last_session.md`.

4.  **Await Resume Command:** I will inform you that the session has been saved and await your instruction to "resume from last session." When you give the command, I will read the file to restore my context and continue our work.