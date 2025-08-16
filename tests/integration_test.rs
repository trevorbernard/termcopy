use std::process::Command;

#[test]
fn test_version_flag() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let expected = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    assert_eq!(stdout.trim(), expected);
}

#[test]
fn test_version_flag_short() {
    let output = Command::new("cargo")
        .args(&["run", "--", "-v"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    let expected = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    assert_eq!(stdout.trim(), expected);
}
