use assert_cmd::Command;
use tempfile::tempdir;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};
use base64::Engine as _;

#[test]
fn test_with_image_generation() {
    // 1. Start a mock server
    let server = MockServer::start_blocking();
    let b64_image = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/wcAAwAB/epv2AAAAABJRU5ErkJggg==";
    let response_body = serde_json::json!([{
        "candidates": [{
            "content": {
                "parts": [{
                    "inlineData": {
                        "mimeType": "image/png",
                        "data": b64_image
                    }
                }]
            }
        }]
    }]);

    Mock::given(method("POST"))
        .and(path_regex("/v1beta/models/gemini-pro:streamGenerateContent.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount_blocking(&server);

    // 2. Setup Ferri project in a temp directory
    let dir = tempdir().unwrap();
    let base_path = dir.path();
    let output_image_path = base_path.join("test.png");

    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("secrets")
        .arg("set")
        .arg("GOOGLE_API_KEY")
        .arg("test-key")
        .assert()
        .success();

    // Use the mock server's URI in the model definition
    let model_name_with_uri = format!("{}/gemini-pro", server.uri());
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("models")
        .arg("add")
        .arg("gemini-test")
        .arg("--provider")
        .arg("google")
        .arg("--model-name")
        .arg(model_name_with_uri) // This is a trick to redirect the request
        .arg("--api-key-secret")
        .arg("GOOGLE_API_KEY")
        .assert()
        .success();

    // 3. Run the `with` command
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("with")
        .arg("--model")
        .arg("gemini-test")
        .arg("--output")
        .arg(&output_image_path)
        .arg("--")
        .arg("a picture of a cat")
        .assert()
        .success();

    // 4. Verify the output
    assert!(output_image_path.exists());
    let expected_bytes = base64::engine::general_purpose::STANDARD.decode(b64_image).unwrap();
    let file_bytes = std::fs::read(&output_image_path).unwrap();
    assert_eq!(file_bytes, expected_bytes);
}


#[test]
fn test_with_simple_command() {
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("init")
        .assert()
        .success();

    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("with")
        .arg("--")
        .arg("echo")
        .arg("it works")
        .assert()
        .success()
        .stdout("it works\n");
}

#[test]
fn test_with_secret_injection() {
    let dir = tempdir().unwrap();
    let base_path = dir.path();

    // 1. Initialize
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("init")
        .assert()
        .success();

    // 2. Set a secret
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("secrets")
        .arg("set")
        .arg("MY_TEST_SECRET")
        .arg("hello_from_secret")
        .assert()
        .success();

    // 3. Run a command that prints the environment variable
    // Note: `printenv` is not available on Windows. A more robust test
    // would use a custom script.
    if cfg!(windows) {
        println!("Skipping `printenv` test on Windows.");
        return;
    }
    
    Command::cargo_bin("ferri").unwrap()
        .current_dir(base_path)
        .arg("with")
        .arg("--")
        .arg("printenv")
        .arg("MY_TEST_SECRET")
        .assert()
        .success()
        .stdout("hello_from_secret\n");
}

