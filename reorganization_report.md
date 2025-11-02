# File Reorganization and Cleanup Report

This document summarizes the directory restructuring, file migrations, and cleanup operations performed on the project.

## Summary

The primary goal was to consolidate all documentation and project-related materials into a unified `docs/` directory and all image assets into an `assets/` directory. This improves organization and clarifies the project structure. Obsolete files and directories were removed after their contents were safely migrated.

## New Directory Structure

The following directories were created to house the reorganized content:

```
/
├── assets/
└── docs/
    ├── user_guide/
    │   ├── examples/
    │   └── walkthroughs/
    ├── project/
    │   ├── engineering/
    │   ├── product/
    │   ├── quality/
    │   └── development_logs/
    └── archive/
        ├── dubious_docs/
        ├── proposed_flows/
        └── sandbox/
```

A `docs/README.md` file was also created to explain the new layout.

---

## Detailed File Migrations

### 1. Assets

All image files (`.png`, `.gif`, `.jpg`, `.jpeg`) were moved from their various original locations into the central `assets/` directory.

-   **Original Locations:** `.` (root), `project_resources/`, `project_resources/quality/examples/`, `project_resources/sandbox/`, etc.
-   **New Location:** `assets/`

### 2. User Guide

Content intended for end-users was moved into `docs/user_guide/`.

-   **Canonical Flows:**
    -   **From:** `examples/canonical-flows/`
    -   **To:** `docs/user_guide/examples/`
-   **Walkthroughs:**
    -   **From:** `project_resources/quality/examples/`
    -   **To:** `docs/user_guide/walkthroughs/`

### 3. Project Documentation

Internal project documents were consolidated under `docs/project/`.

-   **Engineering:**
    -   **From:** `project_resources/engineering/`
    -   **To:** `docs/project/engineering/`
-   **Product Planning:**
    -   **From:** `project_resources/product/`
    -   **To:** `docs/project/product/`
-   **Quality Assurance:**
    -   **From:** `project_resources/quality/`
    -   **To:** `docs/project/quality/`
-   **Development Logs:**
    -   **From:** `development_resources/`
    -   **To:** `docs/project/development_logs/`

### 4. Archived Content

Outdated, experimental, or non-essential materials were archived in `docs/archive/`.

-   **Dubious Docs:**
    -   **From:** `dubious_docs/`
    -   **To:** `docs/archive/dubious_docs/`
-   **Proposed Flows:**
    -   **From:** `examples/proposed_flows/`
    -   **To:** `docs/archive/proposed_flows/`
-   **Sandbox:**
    -   **From:** `project_resources/sandbox/`
    -   **To:** `docs/archive/sandbox/`

---

## Code and File Deletions

The following files and directories were deleted *after* their contents were successfully migrated or because they were deemed obsolete.

### 1. Obsolete Directories

These parent directories were removed once they were empty:

-   `examples/`
-   `development_resources/`
-   `project_resources/`

### 2. Obsolete Files & Code

-   **Duplicate Crate:** The `ferri-core/` directory at the root level was a duplicate and was removed. The authoritative version remains in `crates/ferri-core`.
-   **Disabled Test:** `ferri-cli/tests/interop.rs.disabled` was removed.
-   **Archived Markdown:** The following root-level files were moved to `docs/archive/` for posterity:
    -   `oktryitnow.md`
    -   `POST_ALPHA_REGROUP.md`
    -   `STATE_OF_FLOW.md`

---

## File Modifications

-   **`README.md`:** The main `README.md` was updated to:
    -   Point all image links (`logo.png`, `showtime.gif`) to the new `assets/` directory.
    -   Update the description of the `ferri do` command.
    -   Add a new entry and a "Use Cases" section for the new `ferri plan` command.
