use assert_cmd::Command;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("ferri-cli").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}
