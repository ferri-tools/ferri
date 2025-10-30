# Ferri Quality Assurance

This directory contains manual test plans, walkthroughs, and quality assurance documentation organized by Ferri's architectural layers.

---

## Directory Structure

```
quality/
├── L1/           # Layer 1: Core Execution (init, secrets, models, ctx, with)
├── L2/           # Layer 2: Workflow Automation (run, ps, yank, flow)
├── L3/           # Layer 3: Agentic Engine (do)
├── examples/     # Feature demonstrations and walkthroughs
└── README.md     # This file
```

---

## Layer 1: Core Execution

**Commands:** `init`, `secrets`, `models`, `ctx`, `with`
**Status:** ✅ Stable

Test plans for the foundation layer that manages your environment, models, and executes synchronous, single-shot commands.

### Files:
- `init_and_secrets_test_plan.md` - Project initialization and secrets management
- `models_test_plan.md` - Model registration and configuration
- `context_test_plan.md` - Context management (ctx add/rm/ls/clear)
- `with_test_plan.md` - Synchronous command execution
- `l1_manual_test_plan.md` - Comprehensive L1 testing suite
- `l1_interop_test_plan.md` - Cross-feature integration tests
- `hybrid_model_test_plan.md` - Mixed local/remote model workflows
- `multimodal_test_plan.md` - Vision and multimodal capabilities

---

## Layer 2: Workflow Automation

**Commands:** `run`, `ps`, `yank`, `flow`
**Status:** ⚠️ Under active development

Test plans for the automation layer that runs commands as background jobs, monitors their status, and orchestrates multi-step workflows.

### Files:
- `flow_schema_manual_test_plan.md` - **NEW!** Test plan for ferri-flow.yml schema (Issue #15)
- `flow_pipeline_test_plan.md` - Legacy flow pipeline tests
- `local_process_flow_test_plan.md` - Local process-based flows
- `ferri_run_refactor_demo.md` - Background job execution demo
- `test_ferri_run_status.md` - Job status transition validation
- `uat_flow_engine_refactor.md` - Flow engine UAT
- `long_running_jobs_walkthrough.md` - Long-running job handling

---

## Layer 3: Agentic Engine

**Commands:** `do`
**Status:** 🚧 Experimental

Test plans for the intelligent director that takes high-level goals, formulates multi-step plans, and executes them.

### Files:
- `agentic_do_walkthrough.md` - Agentic command execution walkthrough

---

## Examples & Walkthroughs

Feature-specific demonstrations and use-case walkthroughs.

### Files:
- `contour_detection_walkthrough.md` - Computer vision example
- `image_editing_walkthrough.md` - Image manipulation demo
- `image_generation_walkthrough.md` - AI image generation
- `gemini_code_review_walkthrough.md` - Code review with Gemini
- `long_running_gemini_walkthrough.md` - Extended Gemini sessions

---

## Running Tests

### Prerequisites
```bash
# Build and install ferri
cargo install --path ferri-cli --force

# Verify installation
ferri --version
```

### Execute a Test Plan
1. Navigate to the appropriate layer directory
2. Open the test plan markdown file
3. Follow the step-by-step instructions
4. Check off completed tests in the summary checklist

### Reporting Issues
When you find a bug:
1. Note the test number and description
2. Record actual vs expected behavior
3. Capture error messages and logs
4. Open an issue at https://github.com/ferri-tools/ferri/issues

---

## Test Plan Template

When creating new test plans, use this structure:

```markdown
# [Layer] [Feature] Manual Test Plan

**Feature:** Feature description
**Layer:** L1/L2/L3
**Status:** Ready for testing / In progress / Blocked
**Prerequisites:** Any required setup

## Test Suite X: Category Name

### Test X.Y: Test Name

**Objective:** What this test validates

(Setup commands)

**Expected Result:**
- ✅ Success criteria
- ❌ Failure criteria

## Summary Checklist
- [ ] Test results...
```

---

## Contributing

When adding test plans:
1. Place in the appropriate layer directory
2. Follow the naming convention: `{feature}_{type}_plan.md`
3. Update this README with a description
4. Link to related GitHub issues

---

## Status Legend

- ✅ Stable - Feature is production-ready
- ⚠️ Under Development - Active work in progress
- 🚧 Experimental - Early stage, expect changes
- ❌ Deprecated - No longer supported
