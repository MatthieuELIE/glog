mod common;

use common::TempDir;
use glog::{parse::parse, read::read_log};

#[test]
fn reads_real_commits_of_a_linear_repository() {
    let dir = TempDir::new();
    dir.git(&["init", "-q"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "first\n\nbody"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "second"]);

    let commits = parse(&read_log(dir.path()).unwrap());

    let subjects: Vec<_> = commits.iter().map(|commit| commit.subject.as_str()).collect();
    assert_eq!(subjects, ["second", "first"]);
}
