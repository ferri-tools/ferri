# Agent Communication Protocol (CHAT)

This protocol governs the tactical, turn-by-turn communication between AI agents working on the same issue. It is **mandatory** and complements the strategic communication that occurs on the GitHub issue itself.

## The Principle: Read Before, Write After

At the beginning and end of every user prompt (a "turn"), each agent MUST interact with a dedicated chat file for the active issue.

-   **The Chat File:** A log file specific to a single issue.
    -   **Location:** `development_resources/agent_comms/`
    -   **Filename:** `chat-<issue-number>.md` (e.g., `chat-17.md`)

## The Workflow

### 1. Start of Turn (Receiving a New Prompt)

1.  **Identify the Active Issue:** Determine the current issue number.
2.  **Read the Chat File:** Before performing any other action, **read the entire contents** of the corresponding `chat-<issue-number>.md` file.
3.  **Load as Context:** This content provides the immediate, up-to-date status from the previous agent's turn. Use it to inform your plan.
4.  **If the file doesn't exist, you are the first agent on this turn. Proceed.**

### 2. End of Turn (Before Responding to the User)

1.  **Complete All Actions:** Finish all the work requested in the user's prompt.
2.  **Append to the Chat File:** Before signaling completion to the user, **append** a structured log entry to the `chat-<issue-number>.md` file. **Do not overwrite.**

## Log Entry Format

Each entry MUST be separated by a horizontal rule (`---`) and contain the following:

```
---
**Agent:** [GEMINI|CLAUDE]
**Timestamp:** [YYYY-MM-DDTHH:MM:SSZ]
**User Prompt:** A brief, one-sentence summary of the user's request.
**Summary of Actions:**
- A bulleted list of the significant actions you took (e.g., read file X, wrote to file Y, ran command Z).
- Include the outcome (success, failure, key observation).
**Final State:** A brief description of the state of the code or environment at the end of your turn.
```

## File Lifecycle

-   **Creation:** The first agent to work on an issue in a given session creates the file.
-   **Deletion:** When the issue is **closed**, the corresponding `chat-<issue-number>.md` file MUST be deleted to ensure a clean state for future work.
