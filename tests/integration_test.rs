use std::process::Command;

fn run_and_get_stdout(args: &[&str]) -> String {
    let output = Command::new("cargo")
        .args(args)
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
    String::from_utf8(output.stdout).expect("Invalid UTF-8")
}

fn expected_version() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

#[test]
fn test_version_flag() {
    let stdout = run_and_get_stdout(&["run", "--", "--version"]);
    assert_eq!(stdout.trim(), expected_version());
}

#[test]
fn test_version_flag_short() {
    let stdout = run_and_get_stdout(&["run", "--", "-v"]);
    assert_eq!(stdout.trim(), expected_version());
}
