mod common;

use common::TempDir;
use glog::{parse::parse, read::read_log};

#[test]
fn reads_commits_of_every_ref_and_finds_head_in_real_refs() {
    let dir = TempDir::new();
    dir.git(&["init", "-q", "-b", "main"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "on main"]);
    dir.git(&["switch", "-q", "-c", "other"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "on other"]);
    dir.git(&["tag", "v1"]);
    dir.git(&["switch", "-q", "main"]);

    let commits = parse(&read_log(dir.path()).unwrap());

    assert_eq!(commits.len(), 2);
    let head: Vec<_> = commits.iter().filter(|commit| commit.is_head).collect();
    assert_eq!(head.len(), 1);
    assert_eq!(head[0].subject, "on main");
}

#[test]
fn reads_real_commits_of_a_linear_repository() {
    let dir = TempDir::new();
    dir.git(&["init", "-q"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "first\n\nbody"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "second"]);

    let commits = parse(&read_log(dir.path()).unwrap());

    let subjects: Vec<_> = commits.iter().map(|commit| commit.subject.as_str()).collect();
    assert_eq!(subjects, ["second", "first"]);
    assert!(commits.iter().all(|commit| commit.hash.len() == 7));
    assert!(commits.iter().all(|commit| commit.date.ends_with("ago")));
    assert!(commits.iter().all(|commit| commit.author == "Ann"));
}
