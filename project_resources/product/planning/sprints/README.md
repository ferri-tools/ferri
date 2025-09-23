# Ferri Sprint Management

This directory contains the sprint planning and tracking system for the Ferri project.

## File Structure

```
sprints/
├── README.md                 # This file
├── general_backlog.csv       # Master backlog of all tickets
├── current_sprint.csv        # Active sprint tickets only
└── archived/                 # Completed sprint records
    ├── pre_alpha_sprint.csv
    ├── alpha_s1_sprint.csv
    └── ...
```

## Sprint Organization

### General Backlog (`general_backlog.csv`)
- **Purpose**: Master repository of all project tickets (T1-T∞)
- **Scope**: Complete project history and future planning
- **Status Values**: `Backlog`, `Sprint_Ready`, `In_Progress`, `Done`, `Descoped`
- **Sprint Assignment**: `Pre_Alpha`, `Alpha_S1`, `Alpha_S2`, `Beta_S1`, etc.

### Current Sprint (`current_sprint.csv`)
- **Purpose**: Active sprint tracking with detailed progress
- **Scope**: Only tickets assigned to the current sprint
- **Additional Fields**: Story points, assignee, daily progress notes
- **Lifecycle**: Archived after sprint completion

## Sprint Naming Convention

| Phase | Sprint | Focus |
|-------|--------|--------|
| **Pre-Alpha** | `Pre_Alpha` | Foundation (T1-T71+) - Core architecture |
| **Alpha** | `Alpha_S1` | Stabilization (T72-T74) - Fix inconsistencies |
| **Alpha** | `Alpha_S2` | Feature Sprint - TUI Implementation |
| **Alpha** | `Alpha_S3` | Feature Sprint - Agentic Engine |
| **Beta** | `Beta_S1` | Polish & Performance |
| **Beta** | `Beta_S2` | Documentation & Testing |

## Workflow

### Starting a New Sprint
1. Review `general_backlog.csv` for `Sprint_Ready` tickets
2. Create new `current_sprint.csv` with selected tickets
3. Update general backlog to mark tickets as `In_Progress`

### During Sprint
- Update `current_sprint.csv` with daily progress
- Move completed tickets to `Done` in general backlog
- Add new discoveries to general backlog as `Backlog`

### Ending a Sprint
1. Archive `current_sprint.csv` → `archived/{sprint_name}_sprint.csv`
2. Update all ticket statuses in general backlog
3. Conduct sprint retrospective
4. Plan next sprint

## Current Status

**Active Sprint**: Pre-Alpha (Foundation Phase)
**Next Sprint**: Alpha_S1 (Stabilization - T72-T74 priority)

The Pre-Alpha sprint focused on building core L1 and L2 functionality. The next sprint should prioritize the inconsistencies identified in `POST_ALPHA_REGROUP.md` before advancing to advanced features.