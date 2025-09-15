use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_l1_command_interop() {
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    // 1. `init`: Initialize the project
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("init")
        .assert()
        .success();

    // 2. `secrets`: Store a fake API key
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("secrets")
        .arg("set")
        .arg("MY_API_KEY")
        .arg("fake-key-12345")
        .assert()
        .success();

    // 3. `ctx`: Create a dummy file and add it to the context
    let context_file = base_path.join("my_code.py");
    fs::write(&context_file, "print('This is my python code.')\n").unwrap();

    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("ctx")
        .arg("add")
        .arg("my_code.py")
        .assert()
        .success();

    // 4. `with`: Execute a command that uses both secrets and context
    // This test uses `sh -c` to create a small script on the fly.
    // It checks if the API key is set and if the context is prepended to the prompt.
    let script = r#"#
        if [ "$MY_API_KEY" != "fake-key-12345" ]; then
            echo "API key not set correctly"
            exit 1
        fi
        
        # The last argument is the prompt, which should have the context prepended.
        # We check if the prompt starts with the content of our context file.
        prompt="$1"
        expected_start="print('This is my python code.')"
        if [[ "$prompt" != "$expected_start"* ]]; then
            echo "Context not injected correctly. Got: $prompt"
            exit 1
        fi

        echo "Test passed"
    "#;
    
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("with")
        .arg("--")
        .arg("sh")
        .arg("-c")
        .arg(script)
        .arg("sh") // This is $0 for the script
        .arg("Final prompt part") // This is $1, where context gets prepended
        .assert()
        .success()
        .stdout("Test passed\n");
}
