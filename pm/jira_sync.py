# pm/jira_sync.py

import csv
import os

# --- Configuration ---
# In a real-world scenario, these would be configured securely
JIRA_SERVER = "https://your-domain.atlassian.net"
JIRA_USERNAME = "your-email@example.com"
JIRA_API_TOKEN = os.environ.get("JIRA_API_TOKEN") # Best practice: load from environment

# File paths
EPICS_CSV_PATH = "pm/epics.csv"
BACKLOG_CSV_PATH = "pm/sprint_backlog.csv"

def read_csv_data(file_path):
    """Reads data from a CSV file."""
    with open(file_path, mode='r', encoding='utf-8') as infile:
        return list(csv.DictReader(infile))

def update_csv_data(file_path, data, fieldnames):
    """Writes data back to a CSV file."""
    with open(file_path, mode='w', encoding='utf-8', newline='') as outfile:
        writer = csv.DictWriter(outfile, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(data)

def sync_epics_to_jira(dry_run=True):
    """
    Reads the epics.csv file and creates corresponding epics in Jira.
    Updates the CSV with the new JiraTicketID.
    """
    print("--- Starting Epic Sync ---")
    epics_data = read_csv_data(EPICS_CSV_PATH)
    
    for epic in epics_data:
        # If it already has a Jira ID, skip it
        if epic.get("JiraTicketID"):
            print(f"Skipping Epic '{epic['Title']}' - already has Jira ID: {epic['JiraTicketID']}")
            continue

        print(f"Found new Epic to sync: '{epic['Title']}'")

        # --- Placeholder for Jira API Call ---
        # In a real script, you would make an API call here to create the epic.
        # For example:
        # new_jira_key = jira_client.create_epic(
        #     project_key="FER",
        #     summary=epic['Title'],
        #     description=epic['Description']
        # )
        new_jira_key = f"FER-{epic['EpicID'].replace('E', '10')}" # Simulated Jira Key
        # -----------------------------------------

        print(f"  - {'[Dry Run] Would create' if dry_run else 'Successfully created'} Jira Epic with key: {new_jira_key}")
        epic["JiraTicketID"] = new_jira_key

    if not dry_run:
        update_csv_data(EPICS_CSV_PATH, epics_data, epics_data[0].keys())
        print("Successfully updated epics.csv with new Jira IDs.")
    
    print("--- Epic Sync Complete ---\n")


def sync_backlog_to_jira(dry_run=True):
    """
    Reads the sprint_backlog.csv file and creates corresponding stories/tasks in Jira.
    Updates the CSV with the new JiraTicketID.
    """
    print("--- Starting Sprint Backlog Sync ---")
    backlog_data = read_csv_data(BACKLOG_CSV_PATH)

    for item in backlog_data:
        if item.get("JiraTicketID"):
            print(f"Skipping Subtask '{item['SubtaskDescription'][:40]}...' - already has Jira ID: {item['JiraTicketID']}")
            continue

        print(f"Found new backlog item to sync: '{item['SubtaskDescription'][:40]}...'")

        # --- Placeholder for Jira API Call ---
        # new_jira_key = jira_client.create_issue(
        #     project_key="FER",
        #     summary=item['SubtaskDescription'],
        #     # You would also link this to the parent epic here
        # )
        new_jira_key = f"FER-{item['SubtaskID'].replace('T', '').replace('.', '0')}" # Simulated
        # -----------------------------------------
        
        print(f"  - {'[Dry Run] Would create' if dry_run else 'Successfully created'} Jira Issue with key: {new_jira_key}")
        item["JiraTicketID"] = new_jira_key

    if not dry_run:
        update_csv_data(BACKLOG_CSV_PATH, backlog_data, backlog_data[0].keys())
        print("Successfully updated sprint_backlog.csv with new Jira IDs.")

    print("--- Sprint Backlog Sync Complete ---")


if __name__ == "__main__":
    print("Starting Jira sync process (Dry Run Mode)...")
    print("NOTE: This script is a placeholder and will not make real changes.")
    print("To run for real, you would need to implement the Jira API calls and run with a different flag.\n")
    
    # To run this for real, you'd need a library like 'jira-python'
    # and proper authentication.
    
    sync_epics_to_jira(dry_run=True)
    sync_backlog_to_jira(dry_run=True)

    print("\nSync process finished.")
