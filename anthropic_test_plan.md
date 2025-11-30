# Ferri Anthropic (Claude) Provider Test Plan

This document outlines the steps to test the integration of the Anthropic (Claude) provider with the Ferri CLI.

## Prerequisites

1.  You have an Anthropic API key. You can get one from the [Anthropic Console](https://console.anthropic.com/).
2.  The `ferri` CLI is installed and initialized in your project (`ferri init`).

## Testing Steps

### 1. Set the Anthropic API Key

First, you need to securely store your Anthropic API key using the `ferri secrets` command. This prevents your key from being exposed in your command history or scripts.

```bash
ferri secrets set ANTHROPIC_API_KEY
```

You will be prompted to enter your API key.

### 2. Add a Claude Model

Next, add a Claude model to Ferri using the `ferri models add` command. You need to provide:
- An **alias** for the model (e.g., `claude-opus`).
- The **provider**, which is `anthropic`.
- The **model name**, which is the official model ID from Anthropic (e.g., `claude-3-opus-20240229`).
- The **API key secret**, which is the name of the secret you just set (`ANTHROPIC_API_KEY`).

```bash
ferri models add claude-opus \
  --provider anthropic \
  --model-name claude-3-opus-20240229 \
  --api-key-secret ANTHROPIC_API_KEY
```

### 3. Verify the Model was Added

You can verify that the model was added successfully by listing the available models:

```bash
ferri models ls
```

You should see `claude-opus` in the list of models.

### 4. Test the Model with `ferri with`

Now you can use the model to execute a command. The `ferri with` command will execute the command and print the output to the console.

**Example:** Ask the model to write a haiku about Rust.

```bash
ferri with --model claude-opus -- "write a haiku about rust"
```

**Expected Output:**
You should see a haiku about Rust printed to the console.

### 5. Test the Model with `ferri run`

The `ferri run` command submits the job to run in the background. This is useful for long-running tasks.

**Example:** Ask the model to explain the borrow checker.

```bash
ferri run --model claude-opus -- "explain the rust borrow checker in simple terms"
```

This will return a job ID. You can check the status of the job with `ferri ps` and retrieve the output with `ferri yank <job-id>`.

**Expected Output:**
After the job completes, `ferri yank <job-id>` should print a clear explanation of the Rust borrow checker.

## Summary

If you can successfully complete all of the steps above, the Anthropic provider integration is working correctly.
