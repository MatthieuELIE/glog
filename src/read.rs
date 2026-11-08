use std::{path::Path, process::Command};

const LOG_FORMAT: &str = "--format=%H%x1f%P%x1f%D%x1f%ar%x1f%an%x1f%s";

pub fn read_log(dir: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(dir)
        .env("LC_ALL", "C")
        .args(["log", "--all", "--topo-order", LOG_FORMAT])
        .output()
        .map_err(|error| format!("cannot run git: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim_end().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
