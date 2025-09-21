# Ferri - The Local-First AI Toolkit for Developers

[Features](#features) | [Workflow](#workflow) | [Join](#cta)

---

# Focus on Code, Not Context.

Ferri is a local-first AI toolkit that gives your models perfect memory of your project. Stop copy-pasting and start building with a seamless, unified workflow.

[Join the Waitlist](#cta)

---

## Terminal Demo

```sh
➜ ferri init
✓ Ferri environment initialized in ./.ferri

➜ ferri secrets set OPENAI_API_KEY "sk-..."
✓ Secret 'OPENAI_API_KEY' stored securely.

# Tell Ferri what's important for your project
➜ ferri ctx add ./src README.md
✓ Context updated. Tracking 1 directory and 1 file.

# Now, run any model with full project awareness
➜ ferri with --ctx -- ollama run llama3 "What is this code's main purpose?"
[...]

➜ ferri with --ctx --model gpt-4o "Write a test suite for the main function."
[...]
```

---

## <a id="features"></a>A Toolkit for Modern AI Development

Ferri is built on three core principles to streamline your workflow.

### Unified Context

Define your project's context—files, folders, docs—once. Ferri automatically makes it available to any local or remote model, saving you endless copy-pasting.

### Secure & Local-First

Your secrets and project state live encrypted on your machine in a `.ferri` directory. It's like a secure container for your AI environment, built locally and under your control.

### Model Agnostic

Seamlessly switch between a local Ollama model for quick tasks and a powerful cloud API like GPT-4o for heavy lifting, using the exact same command structure.

---

## <a id="workflow"></a>From Local to Cloud, Instantly.

The `ferri with` command is your universal entrypoint. It wraps any command, running it "inside" the context-aware environment you've defined. This creates a consistent, powerful, and reproducible workflow.

Stop wrestling with environment variables and context stuffing. Focus on what you're building.

```sh
# Generate docs with a local model
➜ ferri with --ctx -- ollama run mistral "Generate a README for this project" > README.md
✓ README.md created.

# Refactor code with a powerful cloud model
➜ ferri with --ctx --model gpt-4o-mini "Refactor the main.py file to improve readability" --output src/main_refactored.py
✓ Refactoring complete. Saved to src/main_refactored.py.

# It works with any script or tool
➜ ferri with -- python my_script.py --api-key "$OPENAI_API_KEY"
✓ Script executed with injected secrets.
```

---

## <a id="cta"></a>Ready to Simplify Your AI Workflow?

Ferri is currently in private alpha. Join the waitlist to get early access and be notified when we launch.

**[Get Notified]**

---

© 2025 Ferri. All rights reserved.