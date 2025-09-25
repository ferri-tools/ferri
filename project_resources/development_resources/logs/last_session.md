### Summary of Our Troubleshooting Session

Here is a summary of the issues we've identified and the architectural changes made to address them. This will serve as our shared context when we resume.

1.  **Initial Goal & Flaw:** The initial goal was to make `ferri flow` a simple orchestrator of `ferri run`. This was architecturally flawed because it created a fragile, nested execution (`ferri flow` -> `ferri run` -> `ferri with`) that broke argument parsing and I/O redirection, leading to silent failures.

2.  **Core Architectural Shift:** We corrected this by making the flow runner a true peer to `ferri with` and `ferri run`. The `run_pipeline` function now directly uses the core `execute::prepare_command` and `jobs::submit_job` primitives, eliminating the problematic nesting.

3.  **Polymorphic Job System:** We discovered the job system could only handle local sub-processes, not remote API calls. We fixed this by refactoring `jobs.rs` to be polymorphic. It now has a `spawn_and_monitor_job` thread that can execute both `PreparedCommand::Local` and `PreparedCommand::Remote` jobs, allowing flows to correctly use models like `gemini-pro`.

4.  **Inter-Step Dependency via Context:** We identified that passing data via `stdin` was incorrect and not idiomatic for `ferri`. The correct method is the context system. We refactored `run_pipeline` to:
    *   Clear the context before each step to prevent contamination.
    *   Use `context::add_to_context` to add the files specified in a step's `input` field.
    *   Rely on the step's `command` to include the `--ctx` flag to consume that context.

5.  **Tracking Output Files:** We found that the runner was incorrectly using a job's internal `stdout.log` as the input for the next step, instead of the user-specified output file (e.g., `triage_report.txt`). We fixed this by making the flow runner track the explicit output file paths for each step and use those for subsequent inputs.

**Current Status:** The architecture is now sound, but there is likely a remaining bug in the implementation of the final `run_pipeline` logic that is preventing the correct files from being added to the context at the right time.
