mod common;

use std::process::Command;

use common::TempDir;

#[test]
fn glog_outside_a_repository_fails_with_gits_message() {
    let dir = TempDir::new();

    let output = Command::new(env!("CARGO_BIN_EXE_glog"))
        .current_dir(dir.path())
        .env_remove("GIT_DIR")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not a git repository"));
}

#[test]
fn glog_in_a_repository_without_commit_fails_with_its_own_message() {
    let dir = TempDir::new();
    dir.git(&["init"]);

    let output = Command::new(env!("CARGO_BIN_EXE_glog"))
        .current_dir(dir.path())
        .env_remove("GIT_DIR")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("no commits"));
}

#[test]
fn glog_in_a_repository_with_a_commit_does_not_report_an_empty_history() {
    let dir = TempDir::new();
    dir.git(&["init", "-q", "-b", "main"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "first"]);

    let output = Command::new(env!("CARGO_BIN_EXE_glog"))
        .current_dir(dir.path())
        .env_remove("GIT_DIR")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    assert!(!String::from_utf8_lossy(&output.stderr).contains("no commits"));
}

fn run_outside_a_repository(args: &[&str]) -> std::process::Output {
    let dir = TempDir::new();

    Command::new(env!("CARGO_BIN_EXE_glog"))
        .current_dir(dir.path())
        .env_remove("GIT_DIR")
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn version_flags_print_the_manifest_version_on_stdout() {
    for flag in ["--version", "-V"] {
        let output = run_outside_a_repository(&[flag]);

        assert!(output.status.success(), "{flag}");
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            format!("glog {}\n", env!("CARGO_PKG_VERSION")),
            "{flag}"
        );
    }
}

#[test]
fn an_unknown_argument_fails_with_code_2_and_a_message_on_stderr() {
    let output = run_outside_a_repository(&["--bogus"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--bogus"));
}

#[test]
fn help_flags_print_usage_description_and_keys_on_stdout() {
    for flag in ["--help", "-h"] {
        let output = run_outside_a_repository(&[flag]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success(), "{flag}");
        for expected in [
            "Usage: glog",
            "A read-only, coloured git log graph for the terminal",
            "j  ",
            "k  ",
            "h  ",
            "l  ",
            "g       Select the first commit",
            "G       Select the last loaded commit",
            "Ctrl+d  Move the selection down half a page",
            "Ctrl+u  Move the selection up half a page",
            "q       Quit",
            "Ctrl+C",
        ] {
            assert!(stdout.contains(expected), "{flag}: {expected:?} in {stdout}");
        }
    }
}
