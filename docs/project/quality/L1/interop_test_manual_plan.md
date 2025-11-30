# Manual Test Plan: L1 Command Interoperability (`with --ctx`)

**Objective:** To manually reproduce the failure observed in the `ferri-cli/tests/interop.rs::test_l1_command_interop` test case. This test validates that the `ferri with` command correctly injects secrets as environment variables and file context into the prompt of a subcommand.

**Pre-requisites:**
- The `ferri` binary has been compiled (e.g., by running `cargo build`).
- You are in a shell/terminal.

---

### Test Steps

1.  **Create a Clean Test Environment**

    First, create a new, empty directory for the test and navigate into it. This ensures no other Ferri projects interfere.

    ```sh
    mkdir /tmp/ferri-interop-test
    cd /tmp/ferri-interop-test
    ```

2.  **Initialize a Ferri Project**

    Run the `init` command to create the `.ferri` directory structure.

    ```sh
    ferri init
    ```

3.  **Set a Secret**

    Use the `secrets set` command to store a dummy API key. The `with` command should make this available as an environment variable.

    ```sh
    ferri secrets set MY_API_KEY "fake-key-12345"
    ```

4.  **Create and Add a Context File**

    Create a simple file and add its path to the Ferri context. The `with --ctx` command should prepend the *content* of this file to the final prompt.

    ```sh
    # Create the file
    echo "print('This is my python code.')" > my_code.py

    # Add it to the context
    ferri ctx add my_code.py
    ```

5.  **Define the Verification Script**

    This is the shell script that the `ferri with` command will execute. It's designed to check if the secret and context were injected correctly. For clarity, we'll define it as a shell variable first.

    ```sh
    VERIFICATION_SCRIPT='
    # 1. Check if the secret was injected as an environment variable.
    if [ "$MY_API_KEY" != "fake-key-12345" ]; then
        echo "TEST FAILED: API key not set correctly."
        exit 1
    fi

    # 2. Check if the file context was prepended to the prompt argument ($1).
    prompt="$1"
    # This is the zsh-safe way to put single quotes inside a single-quoted string.
    expected_start="print('"'"'This is my python code.'"'"')"
    if [[ "$prompt" != "$expected_start"* ]]; then
        echo "TEST FAILED: Context not injected correctly."
        echo "Got prompt: $prompt"
        exit 1
    fi

    # 3. If both checks pass, the test succeeds.
    echo "Test passed"
    '
    ```
    *Note: Copy and paste the entire block above, including the single quotes, into your terminal and press Enter.*

6.  **Execute the `ferri with` Command**

    Now, run the final command that mirrors the failing test. This command tells Ferri to execute `sh` with our verification script, passing it a final prompt string.

    ```sh
    ferri with --ctx -- sh -c "$VERIFICATION_SCRIPT" sh "Final prompt part"
    ```

---

### Expected Result

The command should execute without errors. The `VERIFICATION_SCRIPT` should successfully validate both the secret and the context. The final output to the terminal should be:

```
Test passed
```

### Actual Result

The command fails immediately. The `sh -c` command does not receive its script argument correctly, resulting in the following error printed to `stderr`:

```
sh: -c: option requires an argument
Error: Command execution failed with status: exit status: 1
```
