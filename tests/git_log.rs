mod common;

use common::TempDir;
use glog::{
    parse::{RefKind, parse},
    read::read_log,
};

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
    let head: Vec<_> = commits
        .iter()
        .filter(|commit| commit.refs.iter().any(|r| r.kind == RefKind::Head))
        .collect();
    assert_eq!(head.len(), 1);
    assert_eq!(head[0].subject, "on main");
}

fn badges(commit: &glog::parse::Commit) -> Vec<(RefKind, &str)> {
    commit.refs.iter().map(|r| (r.kind, r.name.as_str())).collect()
}

#[test]
fn reads_local_branches_remote_branches_tags_and_head_from_real_refs() {
    let dir = TempDir::new();
    dir.git(&["init", "-q", "-b", "main"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "first"]);
    dir.git(&["tag", "v1"]);
    dir.git(&["update-ref", "refs/remotes/origin/main", "HEAD"]);
    dir.git(&["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/main"]);
    dir.git(&["branch", "feature/x"]);

    let commits = parse(&read_log(dir.path()).unwrap());
    let mut found = badges(&commits[0]);

    assert_eq!(found[0], (RefKind::Head, "HEAD"));
    found.sort_by_key(|(_, name)| *name);
    assert_eq!(
        found,
        [
            (RefKind::Head, "HEAD"),
            (RefKind::Local, "feature/x"),
            (RefKind::Local, "main"),
            (RefKind::Remote, "origin/main"),
            (RefKind::Tag, "v1"),
        ]
    );
}

#[test]
fn reads_a_detached_head_as_a_head_badge_alone() {
    let dir = TempDir::new();
    dir.git(&["init", "-q", "-b", "main"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "first"]);
    dir.git(&["checkout", "-q", "--detach"]);

    let commits = parse(&read_log(dir.path()).unwrap());

    assert_eq!(badges(&commits[0]), [(RefKind::Head, "HEAD"), (RefKind::Local, "main")]);
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
