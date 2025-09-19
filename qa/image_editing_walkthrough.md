# QA Walkthrough: Advanced Multimodal Flow

**Objective:** This document provides a complete, end-to-end guide for testing an advanced multimodal workflow. We will start from a clean slate, set up two different AI models (one local, one remote), and run a flow where they collaborate to create an image.

**Goal:** Successfully execute a `ferri flow` that uses a local model (Ollama's Gemma) to generate a creative prompt, which is then used by a remote model (Google's Gemini) to generate a final image.

---

### Step 1: Start Fresh

Let's create a brand new directory for this test to ensure there are no conflicting configurations.

**Commands:**
```bash
mkdir ferri-advanced-flow-test
cd ferri-advanced-flow-test
ferri init
```

**Expected Result:**
```
Successfully initialized Ferri project in ./.ferri
```

---

### Step 2: Configure the Remote Model (Gemini)

First, we'll set up the powerful, remote image generation model.

**1. Set the Google API Key:**
This command securely stores your API key so `ferri` can use it.
```bash
# Replace "your-google-api-key" with your valid key
ferri secrets set GOOGLE_API_KEY "your-google-api-key"
```
*Expected Result:* `Secret 'GOOGLE_API_KEY' set successfully.`

**2. Register the Gemini Image Model:**
This command tells `ferri` about the specific Google model we want to use for generating images. We give it a memorable alias, `gemini-image-generator`.
```bash
ferri models add gemini-image-generator \
  --provider google-gemini-image \
  --api-key-secret GOOGLE_API_KEY \
  --model-name gemini-2.5-flash-image-preview
```
*Expected Result:* `Model 'gemini-image-generator' added successfully.`

---

### Step 3: Configure the Local Model (Gemma)

Next, we'll set up the fast, local model that will act as our "creative assistant."

**1. Pull the Gemma Model:**
First, you need to download the `gemma` model to your computer using Ollama's command line.
```bash
ollama pull gemma
```
*Expected Result:* A download will complete successfully.

**2. Register the Gemma Model with Ferri:**
Now, tell `ferri` about this local model so we can use it in our flow. We'll give it the alias `gemma`.
```bash
ferri models add gemma --provider ollama --model-name gemma
```
*Expected Result:* `Model 'gemma' added successfully.`

---

### Step 4: Create the Flow and Input Files

With our models configured, we can now create the files for our workflow.

**1. Create the Flow File:**
This YAML file defines the two-step process. Copy and paste the entire block into your terminal.
```bash
cat << 'EOF' > image_editing_flow.yml
name: "Multimodal Image Editing Flow"
steps:
  # Step 1: Use a local model to expand a simple user request into a detailed prompt.
  - name: "Generate Detailed Prompt"
    model:
      model: "gemma"
      prompt: |
        You are a creative assistant that expands simple user requests into detailed, imaginative prompts for an AI image generator.
        Take the user's request from the input and create a rich, descriptive paragraph.
        Focus on visual details, style, lighting, and composition.
        USER REQUEST: {{input}}
    input: "user_request.txt"
    output: "detailed_prompt.txt"

  # Step 2: Use the detailed prompt to generate an image with a remote model.
  - name: "Generate Edited Image"
    model:
      model: "gemini-image-generator"
      prompt: "Generate an image based on the following description: {{input}}"
      outputImage: "edited_image.png"
    input: "detailed_prompt.txt"
EOF
```
*Expected Result:* A file named `image_editing_flow.yml` is created.

**2. Create the User's Simple Request:**
This is the simple instruction that will kick off the entire flow.
```bash
echo "a knight fighting a dragon, but make it in the style of vaporwave" > user_request.txt
```
*Expected Result:* A file named `user_request.txt` is created.

---

### Step 5: Run the Flow

Now, execute the workflow. `ferri` will run the two steps in order, automatically passing the output of the first step to the second.

**Command:**
```bash
ferri flow run image_editing_flow.yml
```

**Expected Result:**
The command will run, showing the status of both steps. It should complete without any errors.

---

### Step 6: Verify the Final Output

The flow is complete. Let's check the results.

**1. Inspect the Generated Prompt:**
See what the local `gemma` model came up with.
```bash
cat detailed_prompt.txt
```
*   **Expected:** The file will contain a detailed, creative paragraph describing a vaporwave-style scene with a knight and a dragon. It will be much more descriptive than our one-line request.

**2. View the Final Image:**
Open the `edited_image.png` file in your file explorer or an image viewer.
*   **Expected:** The image should be a high-quality picture that clearly reflects the *detailed prompt* created by Gemma. You should see a knight, a dragon, and a distinct vaporwave aesthetic (e.g., neon grids, sunset colors, retro-futuristic elements).

---

This concludes the advanced walkthrough. If you have a file named `edited_image.png` that matches the creative prompt, the feature is working perfectly.