# 02: AI-to-AI Chaining Flow

This flow demonstrates a more advanced and powerful concept: the output of one AI model is used as the input for another. This pattern is the foundation of many sophisticated AI workflows, such as summarization, translation, and iterative refinement.

## How It Works

1.  **`generate-long-text` Job:** This job uses a powerful remote model (`gemini-pro`) to generate a long, detailed piece of text (a technical explanation of Rust's ownership model). The result is saved to `rust_ownership.txt`. This file is the intermediate artifact that connects the two AI steps.

2.  **`summarize-text` Job:** This job depends on the first one. It uses a fast, local model (`gemma`) for a less demanding task: summarizing the generated text. It adds `rust_ownership.txt` to its context (`ferri ctx add`) and then calls the model with the `--ctx` flag. The final summary is saved to `summary.txt`.

## State Management

-   **Explicit Context:** This flow highlights the importance of Ferri's explicit context management. The second job doesn't just read the file; it formally adds it to the context using `ferri ctx add`. This ensures the AI model receives the content in a structured way.
-   **AI Collaboration:** This flow shows how you can use different models for different tasks. A powerful model can be used for the heavy lifting (generation), while a smaller, faster model can be used for subsequent refinement or analysis tasks (summarization).
