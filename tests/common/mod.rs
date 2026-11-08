#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU32, Ordering},
};

static COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new() -> Self {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!("glog-test-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).unwrap();

        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn git(&self, args: &[&str]) {
        let status = Command::new("git")
            .current_dir(&self.0)
            .args(["-c", "user.name=Ann", "-c", "user.email=ann@example.com"])
            .args(args)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
