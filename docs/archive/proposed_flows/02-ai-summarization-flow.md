# 02: AI-to-AI Chaining Flow

This flow demonstrates a more advanced and powerful concept: the output of one AI model is used as the input for another. This pattern is the foundation of many sophisticated AI workflows, such as summarization, translation, and iterative refinement.

## How It Works

1.  **`generate-long-text` Job:** This job uses a powerful remote model (`gemini-pro`) to generate a long, detailed piece of text (a technical explanation of Rust's ownership model). The result is saved to `rust_ownership.txt`. This file is the intermediate artifact that connects the two AI steps.

2.  **`summarize-text` Job:** This job depends on the first one. It uses a fast, local model (`gemma`) for a less demanding task: summarizing the generated text. It adds `rust_ownership.txt` to its context (`ferri ctx add`) and then calls the model with the `--ctx` flag. The final summary is saved to `summary.txt`.

## State Management

-   **Explicit Context:** This flow highlights the importance of Ferri's explicit context management. The second job doesn't just read the file; it formally adds it to the context using `ferri ctx add`. This ensures the AI model receives the content in a structured way.
-   **AI Collaboration:** This flow shows how you can use different models for different tasks. A powerful model can be used for the heavy lifting (generation), while a smaller, faster model can be used for subsequent refinement or analysis tasks (summarization).

## How to Run

This flow uses both a remote model (`gemini-pro`) and a local model (`gemma`).

### Prerequisites

1.  **Google API Key:** You need a Google API key with the Gemini API enabled.
2.  **Install and run Ollama:** [https://ollama.com/](https://ollama.com/)
3.  **Pull the gemma model:** `ollama pull gemma:2b`

### Execution

The following commands are fully self-contained. They will create a temporary workspace, configure the necessary secrets and models, and then run the flow.

```bash
# 1. Create a temporary directory and navigate into it.
mkdir -p /tmp/flow-tests/02-ai-summarization && cd /tmp/flow-tests/02-ai-summarization

# 2. Initialize a new ferri workspace.
ferri init

# 3. Set the required secret. Replace "YOUR_KEY" with your actual Google API key.
ferri secrets set GOOGLE_API_KEY "YOUR_KEY"

# 4. Add the required models to the workspace's registry.
ferri models add gemma --provider ollama --model-name gemma:2b
ferri models add gemini-pro \
  --provider google \
  --api-key-secret GOOGLE_API_KEY \
  --model-name gemini-2.5-pro

# 5. Create the flow YAML file in the current directory.
cat <<'EOF' > 02-ai-summarization-flow.yml
# This flow uses a powerful AI to generate a long document, then a smaller AI to summarize it.
apiVersion: ferri.flow/v1alpha1
kind: Flow
metadata:
  name: ai-text-generation-and-summarization
spec:
  jobs:
    generate-long-text:
      name: "Generate Technical Document"
      steps:
        - name: "Use Gemini to explain Rust's ownership model"
          run: "ferri with --model gemini-pro --output rust_ownership.txt -- 'Explain the concept of ownership in Rust in about 300 words, including the rules of ownership, borrowing, and slices.'"

    summarize-text:
      name: "Summarize Document with Local Model"
      needs:
        - generate-long-text
      steps:
        - name: "Add the document to the context"
          run: "ferri ctx add rust_ownership.txt"
        - name: "Use Gemma to summarize the document"
          run: "ferri with --ctx --model gemma --output summary.txt -- 'Summarize the provided text about Rust ownership in a single, concise paragraph.'"
EOF

# 6. Run the flow.
ferri flow run 02-ai-summarization-flow.yml
```

After the run, you can inspect the generated `rust_ownership.txt` and `summary.txt` in `/tmp/flow-tests/02-ai-summarization`.