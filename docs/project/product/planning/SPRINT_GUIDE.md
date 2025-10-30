# The Ferri Sprint Playbook: Your Guide to AI-Assisted Development

## Welcome\! Our Goal & Your Role

Welcome to the team\! 🚀 We're excited to have you on board for what will be a fast-paced and fascinating week.

Our single goal is to **build a functional demo of the Ferri CLI tool in one week**. What makes this project special is *how* we're building it. We're using a cutting-edge AI-assisted workflow where we, the humans, act as architects and directors, and a specialized AI assistant handles the hands-on coding.

Your role in this sprint is the most important one: you are the **Director**. You won't be writing Rust code directly. Instead, your job is to:

1.  **Analyze** high-level goals and break them down into precise, logical, and small tasks.
2.  **Instruct** the AI assistant by providing it with clear, unambiguous commands.
3.  **Verify** the AI's work to ensure it's correct, complete, and meets our quality standards.

Think of yourself as a product manager leading an incredibly fast, highly skilled, but very literal engineering team. Your ability to define requirements, communicate them clearly, and validate the results is the key to our success. This experience is a real-world masterclass in shipping a product from concept to reality, focusing on strategy and execution—skills that are at the absolute heart of the Product Manager role at any FAANG company.

-----

## The Core Workflow: How to Direct the AI

Our development process is a simple, repeatable loop. Your daily rhythm will be performing these four steps, over and over, to drive the project forward.

1.  [cite\_start]**Check the Backlog**: Your first step is always to open `sprint_backlog.csv`[cite: 1, 2, 3, 4]. This file is our **single source of truth**. Find the first `TicketID` that has sub-tasks with a `To Do` status. That's your next mission.

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

# --- Configuration ---
# Your Jira instance details are loaded from environment variables for security.
JIRA_SERVER = os.getenv("JIRA_SERVER")  # e.g., "https://your-domain.atlassian.net"
JIRA_USERNAME = os.getenv("JIRA_USERNAME") # e.g., "your-email@example.com"
JIRA_API_TOKEN = os.getenv("JIRA_API_TOKEN") # Your Jira API token
JIRA_PROJECT_KEY = "FERRI" # The project key for your Jira board

# --- Main Sync Logic ---
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
            # --- Create New Issue ---
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
            # --- Update Existing Issue ---
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
    * `export JIRA_SERVER="https://your-domain.atlassian.net"`
    * `export JIRA_USERNAME="your-email@example.com"`
    * `export JIRA_API_TOKEN="YOUR_SECRET_API_TOKEN"`
5.  **Run the script**: Simply execute `python sync_jira.py`.

You are now fully equipped to direct this project. This sprint is a unique opportunity to practice the core loops of product development—strategy, execution, and validation—at high speed. Embrace the process, be precise in your direction, and let's build something great. Let's get started! 🎉
````