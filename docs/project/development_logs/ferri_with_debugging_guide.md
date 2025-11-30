# Ferri `with` Command Debugging Guide

Hey Jorge,

You're right, `ferri with` is broken, and the rabbit hole of fixing the `flow` tests wasn't the right path. Let's reset and tackle this properly. As requested, here's a detailed guide on the bugs, where to find them, and how to fix them, written for an experienced dev who's new to Rust.

The core issue is that `ferri with --ctx -- your_command` is failing to inject context when **not** using a `--model`. There are actually **three separate bugs** that combine to cause this failure.

---

### Bug #1: The Test is Lying

The primary integration test for this feature is flawed, which has been hiding the real issues.

-   **File:** `ferri-cli/tests/interop.rs`
-   **The Problem:** The test that's supposed to verify context injection for the `with` command is missing the actual `--ctx` flag. It's testing a scenario where context injection isn't even requested.
-   **Why it's a bug:** Without the `--ctx` flag, the command runs, finds no context, and "passes" by simply executing the command. The shell script inside the test then (correctly) fails because the context wasn't prepended. This gives us the failure output, but for the wrong reason.

**Code to Inspect (`ferri-cli/tests/interop.rs`):**

```rust
// ...
    // 4. `with`: Execute a command that uses both secrets and context
    // ...
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("with") // <--- BUG IS HERE! Missing .arg("--ctx")
        .arg("--")
        .arg("sh")
// ...
```

---

### Bug #2: `context.json` is Created with the Wrong Shape

The `ferri init` command creates a `context.json` file that our Rust code cannot read.

-   **File:** `ferri-core/src/lib.rs` (where the file is created)
-   **File:** `ferri-core/src/context.rs` (where the file is read)
-   **The Problem:**
    1.  In `context.rs`, the code expects to read a JSON object like `{"files": []}` into a `Context` struct.
    2.  However, in `lib.rs`, the `initialize_project` function creates the file with just an empty array `[]`.
-   **Why it's a bug:** When `get_full_context` is called, `serde_json` (the Rust JSON library) tries to deserialize `[]` into the `Context` struct and fails because it's expecting an object, not an array. This is the "invalid length 0, expected struct Context" error we saw.

**Code to Inspect (`ferri-core/src/lib.rs`):**

```rust
// in `initialize_project` function
pub fn initialize_project(base_path: &Path) -> std::io::Result<()> {
    // ...
    let context_path = ferri_dir.join("context.json");
    if !context_path.exists() {
        fs::write(context_path, "[]")?; // <--- BUG IS HERE!
    }
    // ...
}
```

**And the struct it's trying to become (`ferri-core/src/context.rs`):**

```rust
#[derive(Serialize, Deserialize, Debug)]
struct Context {
    files: Vec<String>, // This expects {"files": [...]}
}
```

---

### Bug #3: The Core Logic Flaw in `ferri with`

This is the original bug that started the whole investigation. The `with` command's logic completely ignores the `--ctx` flag if a `--model` is not also present.

-   **File:** `ferri-core/src/execute.rs`
-   **The Problem:** In the `prepare_command` function, there's a large `if let Some(model_alias) = &args.model` block. This handles all logic for model-based commands, and it correctly checks for `args.use_context`. However, the `else` block at the end, which handles simple, non-model commands, **never checks for `args.use_context`**. It just takes the command and runs it, completely ignoring any context.
-   **Why it's a bug:** This is the main functional failure. The `else` block needs to perform the same context-gathering logic as the `if` block.

**Code to Inspect (`ferri-core/src/execute.rs`):**

```rust
// in `prepare_command` function
pub fn prepare_command(
    // ...
) -> io::Result<(PreparedCommand, HashMap<String, String>)> {
    // ...
    if let Some(model_alias) = &args.model {
        // ... LOTS OF LOGIC ...
        // This block correctly handles context injection for models
        // ...
    } else {
        // THIS IS THE BUGGY BLOCK
        let command_name = &final_command_with_args[0];
        let command_args = &final_command_with_args[1..];
        let mut command = Command::new(command_name);
        command.args(command_args);
        Ok((PreparedCommand::Local(command), decrypted_secrets))
    }
}
```

---

### Recommended Fix Order

1.  **Fix the Test (`interop.rs`):** Add `.arg("--ctx")` to the test command. This makes the test valid.
2.  **Fix `init` (`lib.rs`):** Change the `fs::write` for `context.json` to write `{"files": []}`.
3.  **Fix `with` (`execute.rs`):** Add logic to the `else` block to check for the `use_context` flag, load the context, and prepend it to the command arguments.

Let me know when you're ready, and we can walk through fixing them one by one.
# 