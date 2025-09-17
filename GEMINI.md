# Gemini Workflow

This document outlines the primary workflow for interacting with the Ferri project. I will use this as my guide for our sessions.

## Session Start

At the start of each session, I will:

1. Greet you with a fact about rust or python to help me learn
2.  **Ask you what you'd like to do.**

---

## What would you like to do?

**1. Check project status:** I will analyze `pm/sprint_backlog.csv` and cross-reference it with the actual implementation in the codebase to give you a summary of completed, in-progress, and pending tasks.

**2. Work on a ticket:** Tell me which ticket you want to work on (e.g., "Let's work on T8"). I will then focus on implementing the subtasks for that ticket.

**3. Create a new ticket:** I will help you add a new row to `pm/sprint_backlog.csv`, ensuring the goal and subtasks match the existing style and granularity.

    *   **Ticket Formatting Rules:**
    *   Each ticket must be broken down into granular, single-action subtasks.
    *   Each subtask must have its own row in the CSV.
    *   The `TicketID` and `Goal` fields must be repeated for every subtask row to ensure traceability.


**4. Mark a ticket as done:** Tell me the subtask ID you've completed (e.g., "T5.8 is done"), and I will update its status in the CSV.

**5. Something else:** If you have a different task in mind, just let me know.

---

## Core Reference Files

To inform our work, I will always refer to:
*   **`README.md`**: For understanding the project's architecture and goals.
*   **`pm/sprint_backlog.csv`**: As the single source of truth for all development tasks.
*   **The source code**: To verify implementation status.

This should go without saying but always confirm if changing the .gitignore. It already happened before and it made things go crazy for a bit.

---
## Development Best Practices

To ensure a high-quality, maintainable codebase, we will adhere to the following practices:

*   **Ticket-Driven Development:** All work, from new features to bug fixes, must begin with a ticket in `pm/sprint_backlog.csv`. This ensures every change is tracked and justified.
*   **Atomic Commits:** Each commit should represent a single, logical change. Commits must reference their corresponding ticket ID (e.g., `fix(auth): T21 - Fix login redirect bug`).
*   **Test-Driven Development (TDD):** For new functionality, we will write a failing test *before* writing the implementation. This ensures the code is correct, testable, and meets requirements from the start.
*   **Continuous Integration:** After every commit, I will run the project's build, lint, and test suites to catch regressions early.
*   **User-Centric Testing:** When a feature is complete, I will install the binary and provide clear instructions for you to perform the final user acceptance testing (UAT).

