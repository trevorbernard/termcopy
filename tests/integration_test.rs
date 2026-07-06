use std::io::Write;
use std::process::{Command, Stdio};

fn termcopy() -> Command {
    Command::new(env!("CARGO_BIN_EXE_termcopy"))
}

#[test]
fn test_version_flag() {
    let expected = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    for flag in ["--version", "-v"] {
        let output = termcopy().arg(flag).output().expect("failed to run binary");
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8");
        assert_eq!(stdout.trim(), expected);
    }
}

// The tests pin `--output stdout` so the escape sequence stays capturable:
// the default cascade would route it to the controlling terminal whenever
// the test runner has one.

#[test]
fn test_copies_stdin_as_osc52() {
    let mut child = termcopy()
        .args(["--output", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to run binary");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"hello world")
        .unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"\x1b]52;c;aGVsbG8gd29ybGQ=\x07");
}

#[test]
fn test_copies_file_as_osc52() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(b"hello world").unwrap();

    let output = termcopy()
        .args(["--output", "stdout"])
        .arg(file.path())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"\x1b]52;c;aGVsbG8gd29ybGQ=\x07");
}

#[test]
fn test_rejects_unknown_output_target() {
    let output = termcopy().args(["--output", "clipboard"]).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("Invalid UTF-8");
    assert!(stderr.contains("stdout") && stderr.contains("tty"));
}
