# Design Evolution of the Ferri Flow Engine

## 1. Introduction

This document outlines the analysis and design evolution of the `ferri flow` engine. The initial implementation, while functional, suffered from several architectural flaws that led to user confusion, unexpected behavior, and a lack of granular control. Through a process of iterative feedback and analysis, a superior architectural model was developed that prioritizes clarity, explicitness, and alignment with the core `ferri` CLI primitives.

## 2. The Initial Problem: Implicit "Magic"

The first version of the flow engine used an `input` field to manage the context for each step.

**Example (Initial Flawed Design):**
```yaml
name: "Code Review Flow"
steps:
  - name: "triage-code"
    # "Magic" happens here: context is cleared, then demo_script.py is added.
    input: "demo_script.py"
    command: "with --ctx --model gemini-pro --output triage_report.txt \"...\""

  - name: "enhance-code"
    # "Magic" again: context is cleared, then two files are added.
    input: "demo_script.py,triage_report.txt"
    command: "with --ctx --model gemini-pro --output enhanced_script.py \"...\""
```

This design led to several critical issues:

*   **User Confusion:** The mechanism for how `input` worked was not transparent. Users were unsure if the context was being cleared, appended to, or replaced. This led to debugging challenges, such as the model hallucinating code because a `--ctx` flag was missing and the user couldn't easily see that the context was empty.
*   **Destructive Operations:** The context was completely reset at the beginning of each step. This made it **impossible to build up or compose a context** across multiple steps. A user could not, for example, add a file in step 1 and then add a second file in step 2 to create a combined context for step 3.
*   **Lack of Granularity:** There was no way to perform simple context operations (like clearing it or removing a single file) without running a command.

## 3. Design Evolution

### Iteration 1: The Abstract `type` Field

The first proposal to fix these issues was to introduce a `type` field to make the steps more explicit.

**Example (Abstract `type` Proposal):**
```yaml
name: "Code Review Flow"
steps:
  - name: "Start Fresh"
    type: context_clear

  - name: "Add Script to Context"
    type: context_add
    paths: [ "demo_script.py" ]

  - name: "Triage Code"
    type: run
    command: "with --ctx --model gemini-pro --output triage_report.txt \"...\""
```

**Critique:** While this was an improvement in explicitness, it introduced a new, abstract language (`type: context_add`) specific only to the flow engine. This violated a core design principle: a user's knowledge of the `ferri` CLI should be directly transferable to writing flows. It created a separate, internal vocabulary instead of leveraging the existing "action verbs" of the tool.

### Iteration 2: The "Primitives" Model (Final Proposal)

The key insight that resolved the design issue was that **a flow should be nothing more than a script of literal `ferri` commands.** This approach eliminates all abstraction and makes the flow engine a simple, transparent orchestrator.

Each step in the YAML is a command that a user could type directly into their terminal.

**Example (Final "Primitives" Design):**
```yaml
name: "Self-Contained Code Review"
steps:
  - name: "Start Fresh"
    # This is a literal ferri command.
    command: "ferri context clear"

  - name: "Add Script to Context"
    # This is a literal ferri command.
    command: "ferri context add demo_script.py"

  - name: "Triage Code"
    # This is a literal ferri command.
    command: "ferri with --ctx --model gemini-pro --output triage_report.txt \"...\""

  - name: "Add Triage Report to Context"
    # This is a literal ferri command.
    command: "ferri context add triage_report.txt"

  - name: "Enhance Code"
    # This is a literal ferri command.
    command: "ferri with --ctx --model gemini-pro --output enhanced_script.py \"...\""
```

## 4. Conclusion and Benefits of the "Primitives" Model

The final "Primitives" design is architecturally superior for the following reasons:

1.  **Zero Magic:** The flow runner becomes a simple, transparent script runner. It reads a command, executes it via the main CLI entry point, and moves to the next. There is no hidden logic.
2.  **Intuitive and Consistent:** If a user knows how to use `ferri` from the command line, they already know how to write a flow. This creates a seamless user experience with a minimal learning curve.
3.  **Adherence to Core Principles:** It directly aligns with the "ferri action verb" principle, where every operation is a clear, understandable command.
4.  **Future-Proof:** Any new command added to the `ferri` CLI in the future (e.g., `ferri test run`, `ferri db query`) will automatically be usable in flows without requiring any changes to the flow engine's parsing logic.
5.  **Enhanced Debuggability:** When a flow fails, the user can copy the exact `command` from the failed step and run it directly in their terminal to debug the issue in isolation.

This evolution in design moves the `ferri flow` engine from a complex, opaque system to a simple, powerful, and transparent automation tool.
