# QA Walkthrough: Image Generation with `ferri with`

**Objective:** This document provides a step-by-step guide to verify the end-to-end functionality of generating an image using a remote Google model and saving it to a file.

**Goal:** Successfully run a `ferri with` command that generates an image and saves it locally, confirming the API call and file output are working correctly.

---

### Step 1: Initialize a Clean Ferri Project

First, create a new directory for our test and initialize `ferri` inside it. This ensures we are working in a clean environment.

**Commands:**
```bash
mkdir ferri-image-test
cd ferri-image-test
ferri init
```

**Expected Result:**
```
Successfully initialized Ferri project in ./.ferri
```

---

### Step 2: Set Your Google API Key

Next, securely store your Google API key. `ferri` will use this key to authenticate with the image generation API.

**Command:**
```bash
# Replace "your-google-api-key" with the key that worked in the experiment
ferri secrets set GOOGLE_API_KEY "your-google-api-key"
```

**Expected Result:**
```
Secret 'GOOGLE_API_KEY' set successfully.
```

---

### Step 3: Register the Image Generation Model

Now, we need to tell `ferri` about the specific Google model that can generate images. We will give it an alias, `gemini-image-generator`, and specify the correct provider.

**Command:**
```bash
ferri models add gemini-image-generator \
  --provider google-gemini-image \
  --api-key-secret GOOGLE_API_KEY \
  --model-name gemini-2.5-flash-image-preview
```

**Expected Result:**
```
Model 'gemini-image-generator' added successfully.
```

---

### Step 4: Generate and Save the Image

This is the final test. We will use `ferri with` to call the model we just registered, provide a prompt, and tell it where to save the resulting image with the `--output` flag.

**Command:**
```bash
ferri with --model gemini-image-generator --output "my_cat_photo.png" -- "a photorealistic picture of a cat sleeping on a couch"
```

**Expected Result:**
1.  The command should execute without any `429` or other API errors.
2.  You should see a success message printed to your terminal:
    ```
    Successfully saved image to my_cat_photo.png
    ```
3.  A new file named `my_cat_photo.png` will be created in your `ferri-image-test` directory.
4.  When you open `my_cat_photo.png`, it should be a valid image that matches the prompt.

---

This concludes the image generation walkthrough. If all steps were successful, the feature is working correctly.
