# Ferri Quick Test Plan

This document provides a quick self-test sequence you can run from your terminal.

**Prerequisites:**
*   `ferri` is installed globally.
*   Ollama is running with the `gemma:2b` model pulled (`ollama pull gemma:2b`).

---

### Test Sequence

**1. Setup a Test Project**
```bash
mkdir ~/ferri-quick-test
cd ~/ferri-quick-test
ferri init
```
*Verification: You should see a success message.*

**2. Create Context**
```bash
echo "My favorite color is blue." > context.txt
ferri ctx add context.txt
```
*Verification: You should see a success message.*

**3. Register a Model**
```bash
ferri models add gemma --provider ollama --model-name gemma:2b
```
*Verification: You should see a success message.*

**4. Test `ferri with` (Foreground)**
```bash
ferri with --model gemma --ctx "Based on the context, what is my favorite color?"
```
*Verification: You should get an immediate answer like: "Your favorite color is blue."*

**5. Test `ferri run` (Background)**
```bash
ferri run --model gemma --ctx "Based on the context, what is my favorite color?"
```
*Verification: This will give you a job ID, like `job-xxxxxx`.*

**6. Check the Result**
```bash
# Wait a few seconds for the model to finish
sleep 5 

# Check the job status
ferri ps

# Get the output (replace with your job ID)
ferri yank <job-id> 
```
*Verification: The `ps` command should show your job as "Completed", and the `yank` command should print the same answer you got from the `with` command.*
