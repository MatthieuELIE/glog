use std::process::Command;

#[test]
fn glog_binary_runs_successfully() {
    let status = Command::new(env!("CARGO_BIN_EXE_glog")).status().unwrap();

    assert!(status.success());
}
