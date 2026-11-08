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
