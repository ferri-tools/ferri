# showtime.tape - A final, polished demo of the Ferri L1 workflow.

# 1. Configure the output and terminal appearance.
Output showtime.gif
# Adjust font size and width to prevent text overflow
Set FontSize 20
Set Width 1400
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
Type "ferri secrets set GOOGLE_API_KEY"
Enter
Sleep 500ms
Hide
Type "AIzaSyBFTGCFRI7c6SkaIXMFLPkgRcA3BjKY7-0" 
Enter
Show
Sleep 1s

Type "ferri models add gemini-pro --provider google --api-key-secret GOOGLE_API_KEY --model-name gemini-2.5-pro"
Enter
Sleep 1s
Type "ferri models add gemma --provider ollama --model-name gemma:2b"
Enter
Sleep 1s

# 5. Add Context
Type "ferri ctx add project_overview.txt"
Enter
Sleep 2s

# 6. Run Queries
Type `ferri with --ctx --model gemma "Based on the context, what is Ferri?"`
Enter
Sleep 3s

Type `ferri with --ctx --model gemini-pro "Based on the context, what is Ferri?"`
Enter

# 7. Final Pause
# Add a long pause at the end so the final state is visible before the GIF loops.
Sleep 10s