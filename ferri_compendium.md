# --- From: /Users/jorgeajimenez/repos/ferri/README.md ---

# Ferri

Ferri is a local-first AI toolkit that acts as an intelligent director for foundation models. It evolves from a simple command runner into a proactive, agentic partner that can plan and execute complex development tasks.

Ferri creates secure, project-based environments with portable context, unifying your workflow across local (Ollama) and remote (API) models. The goal is to let you focus on your high-level goals, not on the minutiae of context management and command execution.

## Core Concepts

*   **Project-Based:** Ferri operates within your project directory, keeping all context, secrets, and job history in a local `.ferri` directory.
*   **Context-Aware:** You tell Ferri what files are important (`ferri ctx add`), and it automatically injects them into your AI prompts.
*   **Secure:** Secrets like API keys are encrypted and automatically made available to commands, so you never have to expose them in your shell.
*   **Layered Architecture:** Start with simple commands and gradually adopt more powerful, automated, and agentic features as you need them.

## The Ferri Architecture

Ferri is built in layers, allowing you to choose the right level of power for your task.

| Layer | Command(s) | Description |
|---|---|---|
| **L1: Core Execution** | `init`, `secrets`, `models`, `ctx`, `with` | The foundation. Manages your environment, models, and executes synchronous, single-shot commands. |
| **L2: Workflow Automation** | `run`, `ps`, `yank`, `flow` | The automation layer. Runs commands as background jobs, monitors their status, and orchestrates multi-step workflows. |
| **L3: Agentic Engine** | `do` | The intelligent director. Takes a high-level goal, formulates a multi-step plan, and executes it. |

---

## Command Reference

| Command | Description |
|---|---|
| `init` | Initialize a new Ferri project in the current directory. |
| `secrets` | Manage encrypted, project-specific secrets like API keys. |
| `models` | Manage the registry of local (Ollama) and remote (API) models. |
| `ctx` | Manage the project's context from files or job outputs. |
| `with` | Execute a command within a context-aware, synchronous environment. |
| `run` | Run a command as a long-running background job. |
| `ps` | List and manage active background jobs. |
| `yank` | Fetch the output (stdout) of a completed background job. |
| `flow` | Define and run multi-step, declarative AI workflows from a file. |
| `do` | Execute a high-level goal with an AI-powered agentic engine. |

---

## Use Cases & Examples

This section provides a command-by-command breakdown of Ferri's capabilities, each with practical, real-world examples.

### `ferri init`

Initializes a new Ferri project. This creates the `.ferri` directory where all project-specific state, context, secrets, and configurations are stored.

**Use Case: Starting a New Project**
```bash
# Navigate to your project's root directory
cd my-new-app

# Initialize Ferri
ferri init
# Output: ✨ Successfully initialized Ferri project in ./.ferri
```

---


### `ferri secrets`

Manages encrypted, project-specific secrets like API keys. Secrets are stored locally and are never committed to source control.

**Use Case: Storing API Keys for Remote Models**
```bash
# Securely store your OpenAI API key
ferri secrets set OPENAI_API_KEY="sk-..."

# Securely store your Google API key
ferri secrets set GOOGLE_API_KEY "AIza..."
```

---


### `ferri models`

Manages the registry of available models. Ferri features a unified model system that automatically discovers local Ollama models and allows you to register remote, API-based models.

**Use Case: Listing All Available Models**
The `ls` command shows both auto-discovered Ollama models and any remote models you have explicitly registered.
```bash
ferri models ls
# Output:
# ALIAS             PROVIDER    ID/NAME
# llama3            ollama      llama3:latest (discovered)
# gemma:2b          ollama      gemma:2b (discovered)
# gpt-4o            openai      gpt-4o
# gemini-pro        google      gemini-1.5-pro
```

**Use Case: Registering a New Remote Model**
To use a remote model, you must first store its API key using `ferri secrets`, then register the model.
```bash
# Step 1: Store the secret
ferri secrets set ANTHROPIC_API_KEY "sk-ant..."

# Step 2: Register the model, creating an alias ('claude-opus')
# The '--api-key-secret' flag references the NAME of the secret, not the key itself.
ferri models add claude-opus \
  --provider anthropic \
  --api-key-secret ANTHROPIC_API_KEY \
  --model-name claude-3-opus-20240229
```

**Use Case: Creating a Shorter Alias for a Local Model**
```bash
# Create a simple alias 'gemma' for the longer, discovered model name
ferri models add gemma --provider ollama --model-name gemma:2b
```

---


### `ferri ctx`

Manages the project's context (the files and data provided to the AI).

**Use Case: Adding Files and Directories to the Context**
```bash
# Add the entire source directory and the main README
ferri ctx add ./src README.md
```

---


### `ferri with`

Executes a command, injecting secrets and context. This is Ferri's core execution engine.

**Use Case: Switching Seamlessly Between Local and Remote Models**
Once models are registered, you can switch between them using the `--model` flag.
```bash
# First, add some code to the context
ferri ctx add ./src

# Run a query with a fast, local model
ferri with --ctx --model llama3 "Explain the purpose of the main function."

# Run the exact same query with a powerful, remote model for a different perspective
ferri with --ctx --model gpt-4o "Explain the purpose of the main function."

# Run it again with your registered Gemini Pro model
ferri with --ctx --model gemini-pro "Explain the purpose of the main function."
```

**Use Case: Securely Running Scripts**
Any script or tool can be run with `ferri with` to gain access to stored secrets as environment variables.
```bash
# Your python script can now access the API key via os.getenv("OPENAI_API_KEY")
ferri with -- python ./scripts/deploy.py
```

---


### `ferri run`, `ps`, and `yank`

Manages long-running, asynchronous background jobs. The `run` command shares the exact same syntax as `with` for model and context flags.

**Use Case: Generating Documentation in the Background**
```bash
# Step 1: Start a long-running job with 'ferri run'
ferri run --ctx --model gpt-4o "Generate a complete project summary in Markdown"
# Output: Job submitted: job-b4c5d6

# Step 2: Check the status with 'ferri ps'
ferri ps
# Output:
# JOB ID      STATUS      COMMAND
# job-b4c5d6  COMPLETED   ollama run gpt-4o...

# Step 3: Retrieve the output with 'ferri yank'
ferri yank job-b4c5d6 > PROJECT_SUMMARY.md
```

---


### `ferri flow`

Defines and runs multi-step, declarative AI workflows from a YAML file.

**Use Case: Automating Code Generation and Testing**
```yaml
# ci-prep.yml
name: "Prepare for CI"
jobs:
  - id: generate-docs
    command: 'ferri with --ctx --model gpt-4o "Generate docs for the codebase" > DOCS.md'
  - id: write-tests
    dependencies: [generate-docs]
    command: 'ferri with --ctx DOCS.md --model gpt-4o "Write unit tests" > main.test.js'
```
```bash
# Execute the entire workflow
ferri flow run ci-prep.yml
```

---


### `ferri do`

Executes a high-level goal with an AI-powered agentic engine.

**Use Case: Agentic Code Modification**
```bash
ferri do "Add a new '/api/users' endpoint to my Express app. It needs a route, a controller with a placeholder, and must be registered in the main app file."
```

---


## Command Modifiers

For a detailed list of all command modifiers and advanced options (e.g., `--stream`, `--dry-run`, `--interactive`), see [COMMAND_MODIFIERS.md](./COMMAND_MODIFIERS.md).

---


## Demo: AI-Powered Code Review

This demo showcases a workflow where a fast, local model (Gemma) and a powerful, remote model (Gemini Pro) work together to perform a code review.

**1. Setup:**

First, add your Google API key to Ferri's secrets and register the Gemini Pro model.

```bash
# Store your API key
ferri secrets set GOOGLE_API_KEY "your-api-key-here"

# Register the Gemini Pro model
ferri models add gemini-pro \
  --provider google \
  --api-key-secret GOOGLE_API_KEY \
  --model-name gemini-pro
```

**2. Run the Flow:**

Execute the `code_review_flow.yml` pipeline. This flow uses the `pm/demo_script.py` file as its input.

```bash
ferri flow run pm/code_review_flow.yml
```

**3. What it Does:**

*   **Step 1 (Local - Gemma):** Performs a quick "triage" on `demo_script.py` and writes a summary to `triage_report.txt`.
*   **Step 2 (Remote - Gemini Pro):** Uses the triage report to perform a deep analysis and writes an enhanced, secure version of the script to `enhanced_script.py`.
*   **Step 3 (Local - Gemma):** Reads the final code and generates a git commit message in `commit_message.txt`.

---
# --- From: /Users/jorgeajimenez/repos/ferri/pm/AI_GUIDE.md ---

AI Collaboration Guide for Project Ferri

CONTEXT
You are a Rust development assistant helping me build the ferri CLI tool within a Rust workspace (ferri-core, ferri-cli). We are in a high-speed sprint to build a demo, and you must follow this guide precisely.

PRIMARY DIRECTIVE
You MUST adhere to the plans, architecture, and constraints outlined in the tickets I provide. Your primary goal is to complete one ticket at a time, following the protocol exactly.

The Workflow Protocol
When I give you a new ticket, you will perform the following steps in order:

1.  **Implement the Code:**
    *   Write the necessary code to fulfill all the sub-tasks for the given ticket.
    *   Place core logic in the `ferri-core` crate.
    *   Place CLI parsing and I/O logic in the `ferri-cli` crate.
    *   Always choose the simplest possible implementation.

2.  **Write and Run Automated Tests:**
    *   For any ticket involving new or modified logic, you **must** add corresponding unit or integration tests, even if not explicitly mentioned in a sub-task.
    *   Run `cargo test --workspace` to ensure all new and existing tests pass. Do not proceed if any test fails.

3.  **Update the Sprint Backlog:**
    *   After tests pass, you will programmatically open and modify the `sprint_backlog.csv` file.
    *   Find the rows corresponding to the sub-tasks you just completed.
    *   Change the value in the `Status` column from `To Do` to `Done`.

4.  **Log Your Work:**
    *   Append a summary of the changes you made for the completed ticket to a file named `dev.log` in the project root. Include the Ticket ID and a brief description of the implementation.

5.  **Handoff for Manual Verification:**
    *   Your final output to me will announce that the ticket is complete and all tests have passed.
    *   Provide a short, clear list of commands I can run myself to manually verify that the feature works as expected.

After this, you will stop and await my next instruction. Do not work ahead.

---
# --- From: /Users/jorgeajimenez/repos/ferri/pm/epics.md ---

# Ferri Project Epics: From Simple Tool to Intelligent Partner

Hey there! This document outlines the five core "epics" for our sprint. Think of an epic as a big chapter in our product's story. Each one delivers a major piece of value to our users (developers like us!) and gets us closer to our goal of making Ferri an indispensable AI assistant.

---

### Epic 1: The Foundation - Getting Started with Ferri

**The Big Idea:** This is all about the "out-of-the-box" experience. When a developer first tries Ferri, they should be able to get it set up in their project and configured in minutes. This epic builds that smooth on-ramp and establishes the core building blocks of the tool.

**As a developer, I want to...**
*   Easily initialize Ferri in my project.
*   Securely store my sensitive API keys without having to paste them everywhere.
*   Quickly see all the local and remote AI models I have available to use.

**Key Features (The "How"):**
*   `ferri init`: The command that sets up a new project.
*   `ferri secrets`: A secure vault for API keys.
*   `ferri models`: A registry to manage and view available AI models.

**Why it Matters:** A great first impression is everything. This epic ensures the tool feels solid, secure, and easy to grasp from the moment a developer starts using it.

---

### Epic 2: The Core Workflow - Putting AI to Work

**The Big Idea:** This is the heart of Ferri's daily utility. We'll enable developers to tell the AI what code to look at (the "context") and then run a command against it. This epic moves beyond simple prompts and makes the AI truly project-aware.

**As a developer, I want to...**
*   Tell Ferri which files and folders are relevant for my current task.
*   Run a command or ask a question and have the AI automatically use that context to give me a smart, relevant answer.

**Key Features (The "How"):**
*   `ferri ctx`: The command for managing the project's context (adding files, listing them, etc.).
*   `ferri with`: The engine that runs a command, seamlessly injecting the context and secrets.

**Why it Matters:** This delivers on the core promise of Ferri: no more manually copy-pasting code into a prompt. This makes the AI a genuine, context-aware coding partner.

---

### Epic 3: Automation & Efficiency - Running Tasks in the Background

**The Big Idea:** Some AI tasks take time. This epic is about giving that time back to the developer. We'll introduce features to run long jobs in the background and even chain simple steps together into automated workflows.

**As a developer, I want to...**
*   Kick off a long-running AI job (like generating documentation) and not have my terminal be blocked.
*   Check the status of my background jobs and grab their output when they're done.
*   Define simple, repeatable multi-step workflows in a file to automate common tasks.

**Key Features (The "How"):**
*   `ferri run`: Runs a command as a background job.
*   `ferri ps`: Lists and manages active jobs.
*   `ferri yank`: Fetches the output of a completed job.
*   `ferri flow`: Runs a multi-step workflow from a file.

**Why it Matters:** This transforms Ferri from a command-line tool into a true productivity multiplier. It respects the developer's time and handles the waiting for them.

---

### Epic 4: The Magic Wand - Intelligent, Goal-Driven Actions

**The Big Idea:** This is where Ferri starts to feel like magic. Instead of telling the AI *how* to do something step-by-step, the developer can just describe the goal. Ferri will then create and execute a plan to achieve it.

**As a developer, I want to...**
*   Describe a complex change I need in plain English (e.g., "add a new API endpoint for users").
*   Have Ferri understand my goal, figure out the necessary steps, and make the code changes for me.

**Key Features (The "How"):**
*   `ferri do`: The "agentic" engine that takes a high-level goal and turns it into an executable plan.

**Why it Matters:** This is the "wow" feature. It represents the future of AI-assisted development, where the developer's role shifts from writing code to directing an intelligent system that writes the code for them.

---

### Epic 5: Making it Great - Polish and User Experience

**The Big Idea:** A powerful tool is only useful if people enjoy using it. This epic is focused on the details that create a polished, professional, and user-friendly experience. It's less about new features and more about making the existing features great.

**As a developer, I want to...**
*   See clear and helpful instructions when I'm not sure how to use a command.
*   Get sensible, easy-to-understand error messages when I make a mistake.
*   Have access to advanced options to customize how commands run (e.g., streaming output).

**Key Features (The "How"):**
*   Comprehensive help text for all commands.
*   Robust error handling and user feedback.
*   Command modifiers like `--stream`, `--dry-run`, etc.
*   Potentially interactive modes for a more guided experience.

**Why it Matters:** A fantastic user experience builds trust and turns a useful tool into a beloved one. It's the key to driving adoption and making Ferri a staple in a developer's toolkit.

---
# --- From: /Users/jorgeajimenez/repos/ferri/pm/SPRINT_GUIDE.md ---

# The Ferri Sprint Playbook: Your Guide to AI-Assisted Development

## Welcome! Our Goal & Your Role

Welcome to the team! 🚀 We're excited to have you on board for what will be a fast-paced and fascinating week.

Our single goal is to **build a functional demo of the Ferri CLI tool in one week**. What makes this project special is *how* we're building it. We're using a cutting-edge AI-assisted workflow where we, the humans, act as architects and directors, and a specialized AI assistant handles the hands-on coding.

Your role in this sprint is the most important one: you are the **Director**. You won't be writing Rust code directly. Instead, your job is to:

1.  **Analyze** high-level goals and break them down into precise, logical, and small tasks.
2.  **Instruct** the AI assistant by providing it with clear, unambiguous commands.
3.  **Verify** the AI's work to ensure it's correct, complete, and meets our quality standards.

Think of yourself as a product manager leading an incredibly fast, highly skilled, but very literal engineering team. Your ability to define requirements, communicate them clearly, and validate the results is the key to our success. This experience is a real-world masterclass in shipping a product from concept to reality, focusing on strategy and execution—skills that are at the absolute heart of the Product Manager role at any FAANG company.

-----

## The Core Workflow: How to Direct the AI

Our development process is a simple, repeatable loop. Your daily rhythm will be performing these four steps, over and over, to drive the project forward.

1.  [cite_start]**Check the Backlog**: Your first step is always to open `sprint_backlog.csv`[cite: 1, 2, 3, 4]. This file is our **single source of truth**. Find the first `TicketID` that has sub-tasks with a `To Do` status. That's your next mission.

2.  **Assemble the Prompt**: You'll instruct the AI by giving it a single, combined prompt. You create this by pasting two things together:

      * The **full contents** of the `AI_GUIDE.md` file. This gives the AI its core rules and constraints.
      * The specific ticket you're working on from the backlog. For example: "Your task is to complete all sub-tasks for ticket T2."

3.  **Issue the Command**: Paste the combined prompt directly into the AI (in our case, the Gemini CLI) and send it.

4.  **Verify the Handoff**: The AI is programmed to complete the ticket, run all tests, update the backlog, and log its work. Its final message to you will be a handoff for manual verification, including the exact commands to run. Your job is to execute those commands and personally confirm the feature works as expected. If it does, you're ready to loop back to step 1. If not, you'll need to debug the instructions you gave it.

-----

## How to Create New Work (The Most Important Part)

Directing the AI on existing tasks is half the job. The other, more creative half is defining what the AI should build next. This is where you translate our high-level vision into a concrete, actionable engineering plan. For a PM, this is equivalent to writing world-class user stories and acceptance criteria.

### Ticket Writing Philosophy

Follow these rules to create perfect, AI-manageable tickets.

  * **Source of Ideas**: All high-level features and goals originate from our project's `README.md`. Your job is to decompose those goals.
  * **The Golden Rule**: **Each sub-task should be small enough that it could be a single git commit.** This is the most important principle. It keeps work atomic and easy to verify.
  * **Be Specific**: Never be vague. Don't say "add a function for the context." Instead, say "In `ferri-core/src/context.rs`, define a public function `add_to_context(key: &str, value: &str) -> std::io::Result<()>`." The AI is smart but has no intuition; it needs total clarity.
  * **Always Include Tests**: A feature isn't "done" until it's tested. Every ticket that introduces new logic **must** include sub-tasks for writing both unit tests (in `ferri-core`) and integration tests (in `ferri-cli`).

### Worked Example

Let's say we need to implement the `ferri run` command, which is our next big feature after `with`. It's supposed to find a `Ferri.toml` file, read a command from it, and execute it using the context. Here’s how we’d break that down and add it to `sprint_backlog.csv`:

```csv
TicketID,Goal,SubtaskID,SubtaskDescription,Status
T5,`run` Command Implementation,T5.1,"In `ferri-core/Cargo.toml`, add the `toml` crate as a dependency.",To Do
T5,`run` Command Implementation,T5.2,"In `ferri-core/src/lib.rs`, create a public function `find_and_run_command() -> Result<(), String>`.",To Do
T5,`run` Command Implementation,T5.3,"The `find_and_run_command` logic should search for `Ferri.toml`, parse it to find a `run_command` key, and execute its value using the existing `execute_with_context` function.",To Do
T5,`run` Command Implementation,T5.4,"In `ferri-core/src/lib.rs`, add unit tests for `find_and_run_command` to handle the file existing, not existing, and being malformed.",To Do
T5,`run` Command Implementation,T5.5,"In `ferri-cli/src/main.rs`, call `ferri_core::find_and_run_command()` from the `run` command's `match` arm and print any error.",To Do
T5,`run` Command Implementation,T5.6,"In `ferri-cli/tests/`, create an integration test `run.rs` that creates a temporary `Ferri.toml` and verifies the `run` command executes it successfully.",To Do
```

-----

## Visualizing Progress with Jira

To keep stakeholders informed, we mirror our progress on a Jira board. However, there is one critical rule:

**`sprint_backlog.csv` is the source of truth. Jira is a read-only visual mirror.**

We **never** create, edit, or update tickets directly in Jira. Doing so would break our automated, data-driven workflow. The entire Jira board is populated automatically by a script that reads our CSV. This ensures that our engineering reality and our reported status are always perfectly in sync, a core principle for any data-fluent product manager.

-----

## The Jira Sync Script

This Python script is our automation engine for Jira. It reads the CSV, connects to Jira, and ensures the board perfectly reflects the state of our backlog.

````python
# sync_jira.py
import os
import pandas as pd
from jira import JIRA

# ---
# Configuration ---
# Your Jira instance details are loaded from environment variables for security.
# ---
JIRA_SERVER = os.getenv("JIRA_SERVER")  # e.g., "https://your-domain.atlassian.net"
JIRA_USERNAME = os.getenv("JIRA_USERNAME") # e.g., "your-email@example.com"
JIRA_API_TOKEN = os.getenv("JIRA_API_TOKEN") # Your Jira API token
JIRA_PROJECT_KEY = "FERRI" # The project key for your Jira board

# ---
# Main Sync Logic ---
# ---
def sync_backlog_to_jira():
    """
    Reads the sprint backlog CSV and syncs its state to Jira.
    - Creates new stories for new TicketIDs.
    - Updates existing stories based on sub-task completion.
    """
    if not all([JIRA_SERVER, JIRA_USERNAME, JIRA_API_TOKEN]):
        print("❌ Error: Missing Jira environment variables.")
        print("Please set JIRA_SERVER, JIRA_USERNAME, and JIRA_API_TOKEN.")
        return

    print("Connecting to Jira...")
    try:
        jira_options = {'server': JIRA_SERVER}
        jira = JIRA(basic_auth=(JIRA_USERNAME, JIRA_API_TOKEN), options=jira_options)
        print(f"✅ Successfully connected to Jira server at {JIRA_SERVER}")
    except Exception as e:
        print(f"❌ Error connecting to Jira: {e}")
        return

    print("Reading sprint_backlog.csv...")
    try:
        df = pd.read_csv("sprint_backlog.csv")
    except FileNotFoundError:
        print("❌ Error: sprint_backlog.csv not found in the current directory.")
        return

    # Group sub-tasks by their parent ticket
    grouped = df.groupby('TicketID')

    for ticket_id, group in grouped:
        ticket_goal = group['Goal'].iloc[0]
        issue_summary = f"[{ticket_id}] - {ticket_goal}"
        
        print(f"\nProcessing Ticket: {ticket_id}")

        # JQL query to search for an existing issue
        jql_query = f'project = {JIRA_PROJECT_KEY} AND summary ~ "{issue_summary}"'
        
        existing_issues = jira.search_issues(jql_query)

        # Build the description from sub-tasks
        description_lines = [f"* {row['SubtaskDescription']} ({row['Status']})" for index, row in group.iterrows()]
        issue_description = "\n".join(description_lines)

        if not existing_issues:
            # ---
            # Create New Issue
            # ---
            print(f"  -> Issue not found in Jira. Creating new story...")
            new_issue_fields = {
                'project': {'key': JIRA_PROJECT_KEY},
                'summary': issue_summary,
                'description': issue_description,
                'issuetype': {'name': 'Story'},
            }
            try:
                new_issue = jira.create_issue(fields=new_issue_fields)
                print(f"  ✅ Created issue: {new_issue.key}")
            except Exception as e:
                print(f"  ❌ Error creating issue for {ticket_id}: {e}")

        else:
            # ---
            # Update Existing Issue
            # ---
            issue = existing_issues[0]
            print(f"  -> Found existing issue: {issue.key}")
            
            # Check if all sub-tasks for this ticket are "Done"
            all_done = group['Status'].eq('Done').all()
            
            issue_status = issue.fields.status.name
            print(f"  -> Current Jira status: '{issue_status}' | Backlog status: {'All Done' if all_done else 'In Progress'}")

            if all_done and issue_status != "Done":
                print("  -> All sub-tasks are done. Transitioning Jira issue to 'Done'...")
                try:
                    transitions = jira.transitions(issue)
                    done_transition = next((t for t in transitions if t['name'].lower() == 'done'), None)
                    if done_transition:
                        jira.transition_issue(issue, done_transition['id'])
                        print(f"  ✅ Transitioned {issue.key} to Done.")
                    else:
                        print(f"  ⚠️ Could not find a 'Done' transition for issue {issue.key}.")
                except Exception as e:
                    print(f"  ❌ Error transitioning {issue.key}: {e}")
            elif not all_done and issue_status == "Done":
                 print("  -> Sub-tasks are not all done, but Jira issue is 'Done'. Moving back to 'In Progress'...")
                 # This logic can be expanded to move it back to 'In Progress' if needed
                 # For now, we'll just report it.
                 print(f"  ⚠️ Mismatch for {issue.key}: Backlog is not done, but Jira is.")


### How to Use the Script

1.  **Save the code:** Save the script above as `sync_jira.py` in the project root.
2.  **Create `requirements.txt`**: Create a file named `requirements.txt` with these two lines:
    ```
    jira
    pandas
    ```
3.  **Install dependencies**: Run `pip install -r requirements.txt` in your terminal.
4.  **Set Environment Variables**: You must set the following environment variables in your terminal session for the script to authenticate with your Jira instance:
    * `export JIRA_SERVER="https://your-domain.atlassian.net"
    * `export JIRA_USERNAME="your-email@example.com"
    * `export JIRA_API_TOKEN="YOUR_SECRET_API_TOKEN"
5.  **Run the script**: Simply execute `python sync_jira.py`.

You are now fully equipped to direct this project. This sprint is a unique opportunity to practice the core loops of product development—strategy, execution, and validation—at high speed. Embrace the process, be precise in your direction, and let's build something great. Let's get started! 🎉
````
---
# --- From: /Users/jorgeajimenez/repos/ferri/qa/context_test_plan.md ---

# Manual QA Test Plan: `ctx` Command

This document outlines the manual testing steps for the `ferri ctx` command.

**Prerequisites:**
1.  A `ferri` project must be initialized. If you haven't done so, run `ferri init` in your test directory.

---


### Test Case 1: `ferri ctx ls` (Empty Context)

**Goal:** Verify that listing an empty context works correctly.

**Steps:**

1.  **Navigate to your initialized test project directory.**
2.  **Run the list command:**
    ```bash
    ferri ctx ls
    ```
3.  **Verify the Output:**
    *   You should see the exact message: `Context is empty.`

---


### Test Case 2: `ferri ctx add` and `ls` (Populated Context)

**Goal:** Verify that adding items to the context and listing them works correctly.

**Steps:**

1.  **Add multiple paths:**
    *   From your test directory, run the following command. Note that these files/directories do not need to actually exist for this test.
    ```bash
    ferri ctx add README.md src/components/Button.js ./styles/
    ```
2.  **Verify the Output:**
    *   You should see a success message, for example: `Successfully added 3 path(s) to context.`

3.  **List the context:**
    *   Run the list command again:
    ```bash
    ferri ctx ls
    ```
4.  **Verify the Output:**
    *   The output should now be a list of the items you added:
        ```
        Current context:
        - README.md
        - src/components/Button.js
        - ./styles/
        ```

---


### Test Case 3: `ferri ctx add` (Handling Duplicates)

**Goal:** Verify that the tool does not add duplicate entries to the context.

**Steps:**

1.  **Add a path that already exists in the context:**
    ```bash
    ferri ctx add README.md new-file.css
    ```
2.  **Verify the Output:**
    *   You should see a success message for adding the paths.

3.  **List the context:**
    *   Run the list command one more time:
    ```bash
    ferri ctx ls
    ```
4.  **Verify the Output:**
    *   The list should now contain `new-file.css`, but `README.md` should only appear **once**. The final list should have 4 items.
        ```
        Current context:
        - README.md
        - src/components/Button.js
        - ./styles/
        - new-file.css
        ```

---
# --- From: /Users/jorgeajimenez/repos/ferri/qa/flow_pipeline_test_plan.md ---

# QA Test Plan: `ferri flow`

**Objective:** Verify the functionality of the `ferri flow` command, including running pipelines, handling I/O, and visualizing workflows.

---


## Prerequisites

1.  **Initialize Ferri:**
    ```bash
    ferri init
    ```

2.  **Set Secrets and Models (for relevant tests):**
    ```bash
    # For Google text models
    ferri secrets set GOOGLE_API_KEY "your-google-api-key"
    ferri models add gemini-pro --provider google --api-key-secret GOOGLE_API_KEY --model-name gemini-1.5-pro-latest

    # For Google image models
    ferri models add gemini-image-generator --provider google-gemini-image --api-key-secret GOOGLE_API_KEY --model-name gemini-2.5-flash-image-preview
    ```

---


## Test Cases

### Test Case 1: Simple Text Processing Flow

**Objective:** Verify that a basic, multi-step pipeline using `process` steps and standard I/O redirection works correctly.

**1. Create `simple_flow.yml`:**
```yaml
name: "Simple Text Flow"
steps:
  - name: "start"
    process:
      process: "echo 'hello world'"
      output: "step1_out.txt"
  - name: "finish"
    process:
      process: "cat step1_out.txt | tr '[:lower:]' '[:upper:]'"
      output: "final_out.txt"
```

**2. Run the flow:**
```bash
ferri flow run simple_flow.yml
```

**3. Verification:**
*   Check the contents of `final_out.txt`.
*   **Expected:** The file should contain the text `HELLO WORLD`.

---


### Test Case 2: Image Generation Flow

**Objective:** Verify that a `ModelStep` can generate an image and save it to a file.

**1. Create `image_flow.yml`:**
```yaml
name: "Cat Image Generation"
steps:
  - name: "generate-image"
    model:
      model: "gemini-image-generator"
      prompt: "a watercolor painting of a cat in a library"
      outputImage: "cat_painting.png"
```

**2. Run the flow:**
```bash
ferri flow run image_flow.yml
```

**3. Verification:**
*   Check if the file `cat_painting.png` has been created.
*   Open the file to ensure it is a valid image that matches the prompt.

---


### Test Case 3: Flow Visualization (`flow show`)

**Objective:** Verify that the `ferri flow show` command can correctly parse and display a workflow.

**1. Use `simple_flow.yml` from Test Case 1.**

**2. Run the show command:**
```bash
ferri flow show simple_flow.yml
```

**3. Verification:**
*   A Text-based User Interface (TUI) should appear in the terminal.
*   The TUI should display a graph with two nodes: "start" and "finish".
*   Pressing 'q' should exit the TUI gracefully.

---
# --- From: /Users/jorgeajimenez/repos/ferri/qa/hybrid_model_test_plan.md ---

# Manual QA Test Plan: Hybrid Model Workflow

This document outlines the manual testing steps for a `ferri flow` that uses both a local (Ollama) and a remote (Google Gemini) model.

**Prerequisites:**
1.  `ferri` is installed globally.
2.  Ollama is running with the `gemma:2b` model pulled.
3.  You are in a clean test directory (e.g., `~/ferri-hybrid-test`).
4.  The `ferri init` command has been run at the root of the project.
5.  You have a Google AI API key.

---


### Test Case 1: Hybrid Code Review Flow

**Goal:** Verify that a flow can successfully delegate tasks between a local and a remote model, using secrets for authentication.

**Steps:**

1.  **Initialize a project:**
    ```bash
    ferri init
    ```

2.  **Set the Google API Key Secret:**
    *   Replace `"your-api-key-here"` with your actual key.
    ```bash
    ferri secrets set GOOGLE_API_KEY "your-api-key-here"
    ```

3.  **Register the Models:**
    *   Create a short alias for the local Gemma model.
    *   Register the remote Gemini Pro model, linking it to the secret.
    ```bash
    ferri models add gemma --provider ollama --model-name gemma:2b
    ferri models add gemini-pro --provider google --api-key-secret GOOGLE_API_KEY --model-name gemini-1.5-pro-latest
    ```

4.  **Create the Demo Python Script:**
    *   Create a new file named `demo_script.py` and paste the following code into it. This script has intentional flaws for the models to find.

    ```python
    """
    A simple Flask application to demonstrate Ferri's code review capabilities.
    This script contains intentional flaws for the demo.
    """
    from flask import Flask, request
    import os

    app = Flask(__name__)

    @app.route('/')
    def index():
        name = request.args.get('name', 'World')
        return "Hello, " + name + "!"

    # Flaw: This endpoint is vulnerable to command injection
    @app.route('/files')
    def list_files():
        directory = request.args.get('dir', '.')
        # This is dangerous!
        file_list = os.popen("ls " + directory).read()
        return f"<pre>{file_list}</pre>"

    if __name__ == '__main__':
        app.run(debug=True)
    ```

5.  **Create the Hybrid Flow File:**
    *   Create a new file named `code_review_flow.yml` and paste the following YAML into it.
    *   Notice how the `input:` fields tell `ferri` to pipe the content of each file into the model prompts.

    ```yaml
    name: "AI-Powered Code Review & Enhancement"
    steps:
      - name: "triage-code"
        model:
          model: "gemma" # Local model for speed
          prompt: "You are a code triager. Summarize the following Python script, identify any obvious flaws or style issues, and create a checklist for a senior developer to perform a deeper review. Focus on security and performance."
        input: "demo_script.py"
        output: "triage_report.txt"

      - name: "expert-review-and-enhance"
        model:
          model: "gemini-pro" # Remote model for power
          prompt: "You are an expert Python developer. Using the original code and the triage report, perform a deep code review based on the checklist. Then, generate an enhanced, production-ready version of the script that fixes all identified issues."
        input: "triage_report.txt" # Conceptually also uses the original script
        output: "enhanced_script.py"

      - name: "generate-commit-message"
        model:
          model: "gemma" # Local model for speed
          prompt: "You are a git expert. Based on the enhanced code, write a concise and conventional commit message."
        input: "enhanced_script.py"
        output: "commit_message.txt"
    ```

6.  **Run the Flow:**
    ```bash
    ferri flow run code_review_flow.yml
    ```

7.  **Verify the Outputs:**
    *   Check the contents of the three generated files.
    *   **`triage_report.txt`**: Should contain a summary and a checklist from Gemma.
    *   **`enhanced_script.py`**: Should contain a refactored, more secure version of the Python script from Gemini Pro (e.g., using `subprocess.run` instead of `os.popen`).
    *   **`commit_message.txt`**: Should contain a git commit message from Gemma.
    ```bash
    cat triage_report.txt
    cat enhanced_script.py
    cat commit_message.txt
    ```

This confirms that the hybrid workflow is executing correctly.

---
# --- From: /Users/jorgeajimenez/repos/ferri/qa/image_editing_walkthrough.md ---

# QA Walkthrough: Advanced Multimodal Flow

**Objective:** This document provides a complete, end-to-end guide for testing an advanced multimodal workflow. We will start from a clean slate, set up two different AI models (one local, one remote), and run a flow where they collaborate to create an image.

**Goal:** Successfully execute a `ferri flow` that uses a local model (Ollama's Gemma) to generate a creative prompt, which is then used by a remote model (Google's Gemini) to generate a final image.

---


### Step 1: Start Fresh

Let's create a brand new directory for this test to ensure there are no conflicting configurations.

**Commands:**
```bash
mkdir ferri-advanced-flow-test
cd ferri-advanced-flow-test
ferri init
```

**Expected Result:**
```
Successfully initialized Ferri project in ./.ferri
```

---


### Step 2: Configure the Remote Model (Gemini)

First, we'll set up the powerful, remote image generation model.

**1. Set the Google API Key:**
This command securely stores your API key so `ferri` can use it.
```bash
# Replace "your-google-api-key" with your valid key
ferri secrets set GOOGLE_API_KEY "your-google-api-key"
```
*Expected Result:* `Secret 'GOOGLE_API_KEY' set successfully.`

**2. Register the Gemini Image Model:**
This command tells `ferri` about the specific Google model we want to use for generating images. We give it a memorable alias, `gemini-image-generator`.
```bash
ferri models add gemini-image-generator \
  --provider google-gemini-image \
  --api-key-secret GOOGLE_API_KEY \
  --model-name gemini-2.5-flash-image-preview
```
*Expected Result:* `Model 'gemini-image-generator' added successfully.`

---


### Step 3: Configure the Local Model (Gemma)

Next, we'll set up the fast, local model that will act as our "creative assistant."

**1. Pull the Gemma Model:**
First, you need to download the `gemma` model to your computer using Ollama's command line.
```bash
ollama pull gemma
```
*Expected Result:* A download will complete successfully.

**2. Register the Gemma Model with Ferri:**
Now, tell `ferri` about this local model so we can use it in our flow. We'll give it the alias `gemma`.
```bash
ferri models add gemma --provider ollama --model-name gemma
```
*Expected Result:* `Model 'gemma' added successfully.`

---


### Step 4: Create the Flow and Input Files

With our models configured, we can now create the files for our workflow.

**1. Create the Flow File:**
This YAML file defines the two-step process. Copy and paste the entire block into your terminal.
```bash
cat << 'EOF' > image_editing_flow.yml
name: "Multimodal Image Editing Flow"
steps:
  - name: "Generate Detailed Prompt"
    model:
      model: "gemma"
      prompt: |
        You are a creative assistant that expands simple user requests into detailed, imaginative prompts for an AI image generator.
        Take the user's request from the input and create a rich, descriptive paragraph.
        Focus on visual details, style, lighting, and composition.
        USER REQUEST: {{input}}
    input: "user_request.txt"
    output: "detailed_prompt.txt"

  - name: "Generate Edited Image"
    model:
      model: "gemini-image-generator"
      prompt: "Generate an image based on the following description: {{input}}"
      outputImage: "edited_image.png"
    input: "detailed_prompt.txt"
EOF
```
*Expected Result:* A file named `image_editing_flow.yml` is created.

**2. Create the User's Simple Request:**
This is the simple instruction that will kick off the entire flow.
```bash
echo "a knight fighting a dragon, but make it in the style of vaporwave" > user_request.txt
```
*Expected Result:* A file named `user_request.txt` is created.

---


### Step 5: Run the Flow

Now, execute the workflow. `ferri` will run the two steps in order, automatically passing the output of the first step to the second.

**Command:**
```bash
ferri flow run image_editing_flow.yml
```

**Expected Result:**
The command will run, showing the status of both steps. It should complete without any errors.

---


### Step 6: Verify the Final Output

The flow is complete. Let's check the results.

**1. Inspect the Generated Prompt:**
See what the local `gemma` model came up with.
```bash
cat detailed_prompt.txt
```
*   **Expected:** The file will contain a detailed, creative paragraph describing a vaporwave-style scene with a knight and a dragon. It will be much more descriptive than our one-line request.

**2. View the Final Image:**
Open the `edited_image.png` file in your file explorer or an image viewer.
*   **Expected:** The image should be a high-quality picture that clearly reflects the *detailed prompt* created by Gemma. You should see a knight, a dragon, and a distinct vaporwave aesthetic (e.g., neon grids, sunset colors, retro-futuristic elements).

---

This concludes the advanced walkthrough. If you have a file named `edited_image.png` that matches the creative prompt, the feature is working perfectly.
--- # --- From: /Users/jorgeajimenez/repos/ferri/qa/image_generation_walkthrough.md ---

# QA Walkthrough: Image Generation with `ferri with`

**Objective:** This document provides a step-by-step guide to verify the end-to-end functionality of generating an image using a remote Google model and saving it to a file.

**Goal:** Successfully run a `ferri with` command that generates an image and saves it locally, confirming the API call and file output are working correctly.

---


### Step 1: Initialize a Clean Ferri Project

First, create a new directory for our test and initialize `ferri` inside it. This ensures we are working in a clean environment.

**Commands:**
```bash
mkdir ferri-image-test
cd ferri-image-test
ferri init
```

**Expected Result:**
```
Successfully initialized Ferri project in ./.ferri
```

---


### Step 2: Set Your Google API Key

Next, securely store your Google API key. `ferri` will use this key to authenticate with the image generation API.

**Command:**
```bash
# Replace "your-google-api-key" with the key that worked in the experiment
ferri secrets set GOOGLE_API_KEY "your-google-api-key"
```

**Expected Result:**
```
Secret 'GOOGLE_API_KEY' set successfully.
```

---


### Step 3: Register the Image Generation Model

Now, we need to tell `ferri` about the specific Google model that can generate images. We will give it an alias, `gemini-image-generator`, and specify the correct provider.

**Command:**
```bash
ferri models add gemini-image-generator \
  --provider google-gemini-image \
  --api-key-secret GOOGLE_API_KEY \
  --model-name gemini-2.5-flash-image-preview
```

**Expected Result:**
```
Model 'gemini-image-generator' added successfully.
```

---


### Step 4: Generate and Save the Image

This is the final test. We will use `ferri with` to call the model we just registered, provide a prompt, and tell it where to save the resulting image with the `--output` flag.

**Command:**
```bash
ferri with --model gemini-image-generator --output "my_cat_photo.png" -- "a photorealistic picture of a cat sleeping on a couch"
```

**Expected Result:**
1.  The command should execute without any `429` or other API errors.
2.  You should see a success message printed to your terminal:
    ```
    Successfully saved image to my_cat_photo.png
    ```
3.  A new file named `my_cat_photo.png` will be created in your `ferri-image-test` directory.
4.  When you open `my_cat_photo.png`, it should be a valid image that matches the prompt.

---

This concludes the image generation walkthrough. If all steps were successful, the feature is working correctly.
--- # --- From: /Users/jorgeajimenez/repos/ferri/qa/init_and_secrets_test_plan.md ---

# Manual QA Test Plan

This document outlines the manual testing steps to verify the core functionality of the `ferri` CLI.

**Prerequisites:**
1.  Ensure `ferri` is installed globally. If you just made changes, reinstall it by navigating to the project root and running:
    ```bash
    cargo install --path ferri-cli --force
    ```
2.  Create a new, empty directory *outside* of the `ferri` project to simulate a real user environment.
    ```bash
    mkdir ~/ferri-test-project && cd ~/ferri-test-project
    ```

---


### Test Case 1: `ferri init`

**Goal:** Verify that the `init` command correctly sets up a new project environment.

**Steps:**

1.  **Navigate to your test directory:**
    ```bash
    # Make sure you are in your new test project directory
    pwd
    ```

2.  **Run the init command:**
    ```bash
    ferri init
    ```

3.  **Verify the Output:**
    *   You should see the success message: `Successfully initialized Ferri project in ./.ferri`

4.  **Verify the Directory and Files:**
    *   Check that the `.ferri` directory was created:
        ```bash
        ls -la
        ```
        (You should see a `.ferri` directory in the list).
    *   Check that the default state files were created inside `.ferri`:
        ```bash
        ls .ferri
        ```
        (You should see `context.json`, `models.json`, and `secrets.json`).

5.  **Verify File Contents:**
    *   Check that the JSON files have the correct initial content:
        ```bash
        cat .ferri/context.json # Should print []
        cat .ferri/models.json  # Should print []
        cat .ferri/secrets.json # Should print {}
        ```

---


### Test Case 2: `ferri secrets set`

**Goal:** Verify that the `secrets set` command can correctly add and encrypt a secret.

**Steps:**

1.  **Set a new secret:**
    *   From within your initialized test project directory, run:
        ```bash
        ferri secrets set MY_API_KEY "123-abc-456-def"
        ```

2.  **Verify the Output:**
    *   You should see the success message: `Secret 'MY_API_KEY' set successfully.`

3.  **Verify the Encrypted Content:**
    *   Look at the contents of the `secrets.json` file:
        ```bash
        cat .ferri/secrets.json
        ```
    *   **IMPORTANT:** You should **NOT** see your key `"123-abc-456-def"` in plain text. The output should be a JSON object containing a long, encrypted string, for example:
        ```json
        {
          "encrypted_data": "a-long-random-looking-string"
        }
        ```
    *   This confirms the encryption is working at a basic level.

---
# --- From: /Users/jorgeajimenez/repos/ferri/qa/l1_interop_test_plan.md ---

# Manual QA Test Plan: L1 Command Interoperability

This document outlines a manual, end-to-end test case to verify that the core L1 commands (`init`, `secrets`, `ctx`, `with`, `run`) work together seamlessly.

**Prerequisites:**
1.  Be in a clean test directory (e.g., `~/ferri-e2e-test`).

---


### The Goal

We will verify that `ferri with` and `ferri run` can correctly use secrets and context together. We will use:
- `ferri secrets` to store a value.
- `ferri ctx` to define a file that will be our context.
- `ferri with` to execute a command that prints both the secret and the context.
- `ferri run` to do the same in the background.

---


### Test Steps

**1. Initialize the Project**
```bash
ferri init
```
*Verification: Should see the success message.*

**2. Store a Secret**
```bash
ferri secrets set MY_TEST_SECRET "hello_secret"
```
*Verification: Should see the "Secret... set successfully" message.*

**3. Create and Add a Context File**
```bash
echo "hello_context" > my_context.txt
ferri ctx add my_context.txt
```
*Verification: Run `ferri ctx ls` to confirm the file was added.*

**4. Execute and Verify with `ferri with`**
*   This command will print the environment variable to show the secret was injected, and it will use `cat` to receive the context via stdin.
```bash
ferri with --ctx -- sh -c 'echo $MY_TEST_SECRET && cat'
```
*   **Important:** After running the command, you must manually type `prompt` and press Enter to provide the final argument that the context will be prepended to.

*   **Verification:** The output should be:
    ```
    hello_secret
    ---
    File: my_context.txt
    ---
    hello_context

prompt
    ```
*   This confirms that `with` injects both secrets (as env vars) and context (into the arguments).

**5. Execute and Verify with `ferri run`**
*   Now we run the same logic as a background job.
```bash
ferri run --ctx -- sh -c 'echo $MY_TEST_SECRET && echo "background_prompt"'
```
*Verification: You should get a job ID, e.g., `job-xxxxxx`.*

*   Wait a moment for the job to complete, then check the status:
```bash
ferri ps
```
*Verification: The job should be marked as `Completed`.*

*   Finally, retrieve the output:
```bash
ferri yank <your_job_id>
```
*   **Verification:** The output should contain both the secret and the prompt:
    ```
    hello_secret
    background_prompt
    ```

This completes the end-to-end test of the unified `with` and `run` command interoperability.

---
# --- From: /Users/jorgeajimenez/repos/ferri/qa/local_process_flow_test_plan.md ---

# Manual QA Test Plan: Hybrid Flow with Local Processing

This document outlines the manual testing steps for a `ferri flow` that uses a remote model, a local model, and multiple local shell commands (`grep` and `sed`).

**Prerequisites:**
1.  `ferri` is installed globally.
2.  Ollama is running with the `gemma:2b` model pulled.
3.  You are in a clean test directory (e.g., `~/ferri-hybrid-process-test`).
4.  The `ferri init` command has been run at the root of the project.
5.  You have a Google AI API key.

---


### Test Case 1: AI Quote Generation, Local Filtering, and Summarization

**Goal:** Verify that a flow can chain a remote model, `grep`, `sed`, and a local model, passing the output of each step as input to the next.

**Steps:**

1.  **Initialize a project:**
    ```bash
    ferri init
    ```

2.  **Set the Google API Key Secret:**
    *   Replace `"your-api-key-here"` with your actual key.
    ```bash
    ferri secrets set GOOGLE_API_KEY "your-api-key-here"
    ```

3.  **Register the Models:**
    ```bash
    ferri models add gemma --provider ollama --model-name gemma:2b
    ferri models add gemini-pro --provider google --api-key-secret GOOGLE_API_KEY --model-name gemini-1.5-pro-latest
    ```

4.  **Create the Flow File:**
    *   Create a new file named `hybrid_process_flow.yml` and paste the following YAML into it.

    ```yaml
    name: "Hybrid AI Quote Processing"
    steps:
      - name: "generate-quotes"
        model:
          model: "gemini-pro" # Use remote model for a high-quality list
          prompt: "Generate a list of 5 famous quotes about love. Each quote should be on a new line."
        output: "all_quotes.txt"

      - name: "filter-for-love"
        process:
          process: "grep -i 'love'" # Filter for lines containing 'love'
        input: "all_quotes.txt"
        output: "love_quotes.txt"

      - name: "replace-love-with-code"
        process:
          process: "sed 's/love/code/gi'" # Replace 'love' with 'code' (case-insensitive)
        input: "love_quotes.txt"
        output: "code_quotes.txt"

      - name: "summarize-quotes"
        model:
          model: "gemma" # Use local model for a quick summary
          prompt: "You are a tech philosopher. Briefly summarize the meaning of the following quotes in a single sentence."
        input: "code_quotes.txt"
        output: "summary.txt"
    ```

5.  **Run the Flow:**
    ```bash
    ferri flow run hybrid_process_flow.yml
    ```

6.  **Verify the Outputs:**
    *   Check the contents of the four generated files.
    *   **`all_quotes.txt`**: Should contain 5 quotes about love.
    *   **`love_quotes.txt`**: Should contain the same quotes as the first file.
    *   **`code_quotes.txt`**: Should contain the same quotes, but with every instance of "love" replaced with "code".
    *   **`summary.txt`**: Should contain a philosophical summary of the modified "code" quotes.
    ```bash
    cat all_quotes.txt
    cat love_quotes.txt
    cat code_quotes.txt
    cat summary.txt
    ```

This confirms that the flow can successfully chain remote models, local processes, and local models.
--- # --- From: /Users/jorgeajimenez/repos/ferri/qa/multimodal_test_plan.md ---

# Manual QA Test Plan: Multimodal `with` Command

This document provides a step-by-step walkthrough to manually test `ferri`'s ability to handle multimodal context (images and text) with the `with` command.

**Prerequisites:**
1.  The latest version of `ferri` is installed globally (`cargo install --path ferri-cli`).
2.  You are in a clean test directory (e.g., `~/ferri-multimodal-test`).
3.  The `ferri init` command has been run in this directory.
4.  You have a Google AI API key.

---


### Test Case 1: Image and Text Context Injection

**Goal:** Verify that `ferri` can send an image and a text prompt to a remote model and receive a correct, context-aware response.

**Steps:**

1.  **Initialize a project:**
    ```bash
    mkdir ~/ferri-multimodal-test
    cd ~/ferri-multimodal-test
    ferri init
    ```

2.  **Set the Google API Key Secret:**
    *   Replace `"your-api-key-here"` with your actual key.
    ```bash
    ferri secrets set GOOGLE_API_KEY "your-api-key-here"
    ```

3.  **Register a Multimodal Model:**
    *   Register the Gemini 1.5 Flash model, which is capable of processing images.
    ```bash
    ferri models add gemini-flash --provider google --api-key-secret GOOGLE_API_KEY --model-name gemini-1.5-flash-latest
    ```
    *   Verify the model was added:
    ```bash
    ferri models ls
    ```

4.  **Create a Sample Image:**
    *   This test requires an image. You can use any `.jpg`, `.png`, or `.webp` file.
    *   For this example, save an image of a cat and name it `cat_photo.jpg` in your test directory.

5.  **Create a Sample Text File:**
    *   Create a new file named `instructions.txt` and paste the following text into it.
    ```text
    Your primary goal is to identify the main subject in the image. Your secondary goal is to guess its name. Based on the image, a good name would be "Whiskers".
    ```

6.  **Add Files to Context:**
    *   Add both the image and the text file to the `ferri` context.
    ```bash
    ferri ctx add cat_photo.jpg
    ferri ctx add instructions.txt
    ```
    *   Verify they were added:
    ```bash
    ferri ctx ls
    ```

7.  **Run the `with` Command:**
    *   Execute the `with` command, using the `--ctx` flag to send the image and text file. The prompt will ask the model to follow the instructions from the text file.
    ```bash
    ferri with --model gemini-flash --ctx -- "Follow the instructions in the provided document to analyze the image."
    ```

8.  **Verify the Output:**
    *   The command should execute without any errors.
    *   The output from the model should be a response that both identifies the cat in the image and suggests the name "Whiskers", as specified in `instructions.txt`.

This confirms that the model correctly received and processed both the image and the text context.
--- # --- From: /Users/jorgeajimenez/repos/ferri/qa/with_test_plan.md ---

# Manual QA Test Plan: `with` and `run` Commands

This document outlines the manual testing steps for the `ferri with` and `ferri run` commands, focusing on their unified syntax.

**Prerequisites:**
1.  A `ferri` project must be initialized (`ferri init`).
2.  You should be in your test project directory.
3.  An Ollama model (e.g., `gemma:2b`) should be available.

---


### Test Case 1: Executing a Simple Command

**Goal:** Verify that `ferri with` can run a basic shell command.

**Steps:**

1.  **Run `echo`:**
    *   The `--` separates `ferri`'s arguments from the command to be executed.
    ```bash
    ferri with -- echo "Hello from Ferri!"
    ```
2.  **Verify the Output:**
    *   You should see the output printed directly to your terminal:
        ```
        Hello from Ferri!
        ```

---


### Test Case 2: Secret Injection

**Goal:** Verify that secrets are made available as environment variables.

**Steps:**

1.  **Set a secret:**
    ```bash
    ferri secrets set MY_SECRET_MESSAGE "it_works"
    ```
2.  **Run a command that prints the secret:**
    ```bash
    ferri with -- printenv MY_SECRET_MESSAGE
    ```
3.  **Verify the Output:**
    *   The command should print the value of the secret:
        ```
        it_works
        ```

---


### Test Case 3: Model and Context Injection (`with`)

**Goal:** Verify that `ferri with` correctly uses a specified model and injects context.

**Steps:**

1.  **Add a model alias:**
    ```bash
    ferri models add gemma --provider ollama --model-name gemma:2b
    ```
2.  **Create and add a context file:**
    ```bash
    echo "This is the context." > my_file.txt
    ferri ctx add my_file.txt
    ```
3.  **Run `with` using the model and context:**
    ```bash
    ferri with --model gemma --ctx "What is the content of the provided file?"
    ```
4.  **Verify the Output:**
    *   The model should respond with something similar to:
        > The content of the provided file is "This is the context."

---


### Test Case 4: Model and Context Injection (`run`)

**Goal:** Verify that `ferri run` works with the same syntax as `with` for background jobs.

**Steps:**

1.  **Run the same command in the background:**
    ```bash
    ferri run --model gemma --ctx "What is the content of the provided file?"
    ```
2.  **Verify the Job ID:**
    *   You should see a success message with a job ID, e.g., `Successfully submitted job 'job-xxxxxx'`.

3.  **Check the job status:**
    *   Wait a few seconds for the job to complete.
    ```bash
    ferri ps
    ```
    *   The status of your job should be `Completed`.

4.  **Yank the output:**
    *   Replace `job-xxxxxx` with your actual job ID.
    ```bash
    ferri yank job-xxxxxx
    ```
5.  **Verify the Output:**
    *   The output should be the same as the output from the `with` command in the previous test case. This confirms the unified logic works for background jobs.

```