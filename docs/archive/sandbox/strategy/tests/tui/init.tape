# init.tape - A simple test for the `ferri init` command.

# 1. Configure the output. This tells VHS to record a GIF of the test.
Output init.gif

# 2. Set up the terminal window for a clean recording.
Set FontSize 20
Set Width 1200
Set Height 600

# 3. Use `Hide` to run setup commands without showing them in the GIF.
Hide
Type "rm -rf .ferri" # Ensure we start with a clean directory
Enter
Show

# 4. Simulate the user typing the command.
Type "ferri init"
Enter

# 5. Wait for the command to finish and the UI to render.
Sleep 2s