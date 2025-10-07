# Multi-Agent Collaboration Protocol

This document outlines the mandatory protocol for all AI agents collaborating on this project. Adherence to these rules is critical for preventing conflicts, ensuring code quality, and maintaining a clear development history.

## 1. The Golden Rule: Issues are the Single Source of Truth

- **No work without an issue:** No task, no matter how small, shall be started without a corresponding GitHub issue.
- **One agent per issue:** To prevent race conditions and conflicting work, only one agent may be assigned to an issue at any given time.
- **Clarity is key:** Issues must have a clear title, a detailed description of the goal, and a set of acceptance criteria.

## 2. The Workflow Protocol: A Clear Sequence of Operations

### Step 0: Workspace Isolation

- All AI agents **MUST** operate from a dedicated `git worktree` directory (e.g., `gemini-worktree/`, `claude-worktree/`).
- The root project directory is reserved exclusively for the human operator.
- Before starting any task, confirm you are in your assigned worktree. All file paths and commands must be relative to this worktree.

### Step A: Assignment and Branching

1.  **Claim the Issue:** Before starting work, an agent **must** assign themselves to the GitHub issue. This serves as a lock.
    - **Command:** `gh issue edit <number> --add-assignee "@me"`
2.  **Announce Work:** The agent **must** post a comment on the issue declaring they have started work.
    - **Comment:** `"Starting work on this issue."`
3.  **Create a Branch:** All work **must** be done on a dedicated feature branch, created from the `develop` branch.
    - **Branch Name:** `feature/<issue-number>-<short-description>` (e.g., `feature/17-define-core-schema`)
    - **Commands:**
        ```bash
        git checkout develop
        git pull origin develop
        git checkout -b feature/<issue-number>-<short-description>
        ```

### Step B: Development and Committing

1.  **Atomic Commits:** Commits must be small, logical, and represent a single unit of work.
2.  **Mandatory Issue Reference:** Every commit message **must** reference the issue number.
    - **Format:** `feat(scope): #<issue-number> - <description>` (e.g., `feat(schema): #17 - Add ApiVersion and Kind structs`)
3.  **Pre-Commit Checks:** Before every commit, the agent **must** run the project's build, lint, and test suites to ensure no regressions are introduced. A zero-warning policy is in effect.

### Step C: Handoff and Completion

1.  **Push Your Branch:** Once the work is complete or a handover is required, the agent **must** push the feature branch to the remote repository.
    - **Command:** `git push --set-upstream origin feature/<issue-number>-<short-description>`
2.  **Create Pull Request:** The agent **must** create a pull request targeting the `develop` branch.
    - **Command:** `gh pr create --base develop --title "feat(scope): #<issue-number> - <description>" --body "<summary>"`
    - The PR description should include a summary of changes, testing notes, and "Closes #<issue-number>"
3.  **Update the Issue:** The agent **must** post a final comment on the issue with a link to the PR and a summary of the work done.
4.  **Unassign Yourself:** The agent **must** unassign themselves from the issue to release the lock.
    - **Command:** `gh issue edit <number> --remove-assignee "@me"`

**Important:** PRs merge to `develop` for integration testing. Only the human maintainer merges `develop` to `main` after verifying combined changes.

## 3. Handling Blockers

If an agent is blocked, it must:
1.  Commit and push all current work to its feature branch.
2.  Post a comment on the GitHub issue detailing the blocker.
3.  Unassign itself from the issue so another agent or a human can investigate.

---
*This protocol is non-negotiable. Any deviation will result in a workflow reset.*
