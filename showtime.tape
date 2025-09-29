# showtime.tape - A complete demo of the Ferri L1 workflow.

# 1. Configure the output and terminal appearance.
Output showtime.gif
Set FontSize 22
Set Width 1200
Set Height 800
Set TypingSpeed 50ms

# 2. Setup Your Environment
Type `mkdir ferri-quickstart && cd ferri-quickstart`
Enter
Sleep 1s
Type `echo "Ferri is a local-first AI toolkit." > project_overview.txt`
Enter
Sleep 1s

# 3. Initialize Ferri
Type "ferri init"
Enter
Sleep 1s

# 4. Configure Models
# Use Hide and Show to enter the secret off-screen.
Hide
Type "ferri secrets set GOOGLE_API_KEY"
Enter
Sleep 500ms
# IMPORTANT: Paste your actual, real API key on the next line.
Type "AIzaSyBFTGCFRI7c6SkaIXMFLPkgRcA3BjKY7-0" 
Enter
Show

# Corrected single-line commands
Type "ferri models add gemini-pro --provider google --api-key-secret GOOGLE_API_KEY --model-name gemini-2.5-pro"
Enter
Sleep 1s
Type "ferri models add gemma --provider ollama --model-name gemma:2b"
Enter
Sleep 1s

# 5. Add Context
Type "ferri ctx add project_overview.txt"
Enter
Sleep 1s

# 6. Run Queries
Type `echo "--- Running with local model (Gemma) ---"`
Enter
Sleep 1s
Type `ferri with --ctx --model gemma "Based on the context, what is Ferri?"`
Enter
Sleep 3s

Type `echo "\n--- Running with remote model (Gemini Pro) ---"`
Enter
Sleep 1s
Type `ferri with --ctx --model gemini-pro "Based on the context, what is Ferri?"`
Enter
Sleep 5s