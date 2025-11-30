# Ferri L2 Debugging Test Plan

This document provides a set of commands to debug the L2 functionality for both local (Ollama) and remote (OpenRouter)
models. The `--verbose` flag is used to provide detailed logs.

## Prerequisites

1. You have an OpenRouter API key and an Ollama model pulled locally.
2. The `ferri` CLI is installed and initialized in your project (`ferri init`).
3. You have set your OpenRouter API key as a secret:
   ```bash
   ferri secrets set OPENROUTER_API_KEY
   ```
4. You have added an Ollama model:
   ```bash
   ferri models add gemma --provider ollama --model-name gemma
   ```
5. You have added an OpenRouter model:
   ```bash
   ferri models add openrouter-gemma \
     --provider openrouter \
     --model-name "google/gemma-2-27b-it" \
     --api-key-secret OPENROUTER_API_KEY
   ```

## Testing Commands

Run the following commands and paste the output into the chat.

### Local Model (Ollama)

#### `ferri with`

```bash
ferri with --verbose --model gemma -- "write a haiku about rust"
```

#### `ferri run`

```bash
ferri run --verbose --model gemma -- "write a haiku about rust"
```

After running the `ferri run` command, get the job ID and then run:

```bash
ferri ps
ferri yank <job-id>
```

### Remote Model (OpenRouter)

#### `ferri with`

```bash
ferri with --verbose --model openrouter-gemma -- "write a haiku about rust"
```

#### `ferri run`

```bash
ferri run --verbose --model openrouter-gemma -- "write a haiku about rust"
```

After running the `ferri run` command, get the job ID and then run:

```bash
ferri ps
ferri yank <job-id>
```
