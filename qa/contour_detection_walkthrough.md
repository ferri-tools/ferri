# QA Walkthrough: Multimodal Image Editing (Contour Detection)

**Objective:** This document provides a complete, end-to-end guide for testing a multimodal image editing workflow. We will start from a clean slate, configure a multimodal AI model, and run a flow that takes a local image, analyzes it, and outputs an edited version with contours drawn on it.

**Goal:** Successfully execute a `ferri flow` that uses Google's Gemini model to analyze a local image of a surgical tool and return a new image with the tool's contours highlighted.

---

### Step 1: Start Fresh

Let's create a brand new directory for this test to ensure there are no conflicting configurations.

**Commands:**
```bash
mkdir ferri-contour-test
cd ferri-contour-test
ferri init
```

**Expected Result:**
```
Successfully initialized Ferri project in ./.ferri
```

---

### Step 2: Configure the Remote Model (Gemini)

We need to configure a powerful, remote multimodal model that can both understand the content of an image and follow instructions to edit it.

**1. Set the Google API Key:**
This command securely stores your API key so `ferri` can use it.
```bash
# Replace "your-google-api-key" with your valid key
ferri secrets set GOOGLE_API_KEY "your-google-api-key"
```
*Expected Result:* `Secret 'GOOGLE_API_KEY' set successfully.`

**2. Register the Gemini Multimodal Model:**
This command tells `ferri` about the specific Google model we want to use for analyzing and editing images. We give it a memorable alias, `gemini-image-editor`.
```bash
ferri models add gemini-image-editor \
  --provider google-gemini-image \
  --api-key-secret GOOGLE_API_KEY \
  --model-name gemini-1.5-flash-8b-preview
```
*Expected Result:* `Model 'gemini-image-editor' added successfully.`

---

### Step 3: Create the Flow and Input Files

With our model configured, we can now create the files for our workflow.

**1. Prepare the Input Image:**
For this test, we need an image to analyze. Copy the `surgical_image.png` from the project's `testbox/videoparse` directory into your current `ferri-contour-test` directory.

**Command:**
```bash
# Make sure you are in the 'ferri-contour-test' directory
cp ../testbox/videoparse/surgical_image.png ./source_image.png
```
*Expected Result:* A file named `source_image.png` is now in your directory.

**2. Create the Flow File:**
This YAML file defines the single-step process: send an image and a prompt to the model and get an edited image back.
```bash
cat << 'EOF' > contour_flow.yml
name: "Contour Detection Flow"
steps:
  - name: "Analyze and Draw Contours"
    model:
      # Use the multimodal model we registered
      model: "gemini-image-editor"
      # The prompt tells the model what to do with the image
      prompt: "Analyze the input image and then draw bright green, 4-pixel-wide contours around the main surgical instrument you identify."
      # Specify the input image for the model
      inputImage: "source_image.png"
      # Specify the name for the output image
      outputImage: "contoured_image.png"
EOF
```
*Expected Result:* A file named `contour_flow.yml` is created.

---

### Step 4: Run the Flow

Now, execute the workflow. `ferri` will send the image and the prompt to the Gemini model, which will perform the analysis and editing in one step.

**Command:**
```bash
ferri flow run contour_flow.yml
```

**Expected Result:**
The command will run, showing the status of the step. It should complete without any errors.

---

### Step 5: Verify the Final Output

The flow is complete. Let's check the result.

**View the Final Image:**
Open the `contoured_image.png` file in your file explorer or an image viewer.

*   **Expected:** The image should be the original `source_image.png` but with a bright green outline drawn precisely around the surgical instrument. This confirms the model correctly identified the object and followed the editing instructions.

---

This concludes the walkthrough. If you have a file named `contoured_image.png` that shows the original image with the requested contours, the feature is working perfectly.
