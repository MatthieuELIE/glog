mod common;

use common::TempDir;
use glog::{
    parse::{Commit, RefKind},
    read::{Identity, Loaded, identity, start},
};
use std::path::Path;

fn events(dir: &Path) -> Vec<Loaded> {
    let reader = start(dir).unwrap();

    reader.events.iter().collect()
}

fn read_commits(dir: &Path) -> Vec<Commit> {
    events(dir)
        .into_iter()
        .flat_map(|event| match event {
            Loaded::Commits(commits) => commits,
            _ => Vec::new(),
        })
        .collect()
}

#[test]
fn reads_commits_of_every_ref_and_finds_head_in_real_refs() {
    let dir = TempDir::new();
    dir.git(&["init", "-q", "-b", "main"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "on main"]);
    dir.git(&["switch", "-q", "-c", "other"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "on other"]);
    dir.git(&["tag", "v1"]);
    dir.git(&["switch", "-q", "main"]);

    let commits = read_commits(dir.path());

    assert_eq!(commits.len(), 2);
    let head: Vec<_> = commits
        .iter()
        .filter(|commit| commit.refs.iter().any(|r| r.kind == RefKind::HeadBranch))
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

    let commits = read_commits(dir.path());
    let mut found = badges(&commits[0]);

    assert_eq!(found[0], (RefKind::HeadBranch, "main"));
    found.sort_by_key(|(_, name)| *name);
    assert_eq!(
        found,
        [
            (RefKind::Local, "feature/x"),
            (RefKind::HeadBranch, "main"),
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

    let commits = read_commits(dir.path());

    assert_eq!(badges(&commits[0]), [(RefKind::Head, "HEAD"), (RefKind::Local, "main")]);
}

#[test]
fn reads_real_commits_of_a_linear_repository() {
    let dir = TempDir::new();
    dir.git(&["init", "-q"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "first\n\nbody"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "second"]);

    let commits = read_commits(dir.path());

    let subjects: Vec<_> = commits.iter().map(|commit| commit.subject.as_str()).collect();
    assert_eq!(subjects, ["second", "first"]);
    assert!(commits.iter().all(|commit| commit.hash.len() == 40));
    assert!(commits.iter().all(|commit| commit.time > 0));
    assert!(commits.iter().all(|commit| commit.author == "Ann"));
}

fn commit_repository_of(count: usize) -> TempDir {
    let dir = TempDir::new();
    dir.git(&["init", "-q", "-b", "main"]);
    let stream: String = (0..count)
        .map(|n| {
            format!(
                "commit refs/heads/main\ncommitter Ann <ann@example.com> {} +0000\ndata 2\nc{}\n\n",
                1_000_000 + n,
                n % 10
            )
        })
        .collect();
    let status = std::process::Command::new("git")
        .current_dir(dir.path())
        .args(["fast-import", "--quiet"])
        .env_remove("GIT_DIR")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child.stdin.take().unwrap().write_all(stream.as_bytes())?;
            child.wait()
        })
        .unwrap();
    assert!(status.success());

    dir
}

fn event_sizes(dir: &Path) -> Vec<String> {
    events(dir)
        .iter()
        .map(|event| match event {
            Loaded::Commits(commits) => commits.len().to_string(),
            Loaded::Done => "done".to_string(),
            Loaded::Failed(message) => message.clone(),
        })
        .collect()
}

#[test]
fn reads_three_hundred_commits_as_a_first_batch_of_256_then_the_rest_then_done() {
    let dir = commit_repository_of(300);

    assert_eq!(event_sizes(dir.path()), ["256", "44", "done"]);
}

#[test]
fn reads_the_commits_after_the_first_batch_in_batches_of_4096() {
    let dir = commit_repository_of(600);

    assert_eq!(event_sizes(dir.path()), ["256", "344", "done"]);
}

#[test]
fn reading_outside_a_repository_starts_with_gits_message() {
    let dir = TempDir::new();

    let first = start(dir.path()).unwrap().events.recv().unwrap();

    let Loaded::Failed(message) = first else {
        panic!("expected Failed");
    };
    assert!(message.contains("not a git repository"));
}

#[test]
fn stopping_the_reader_closes_the_channel_without_done() {
    let dir = TempDir::new();
    dir.git(&["init", "-q", "-b", "main"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "first"]);
    let reader = start(dir.path()).unwrap();

    reader.stop();
    let rest: Vec<_> = reader.events.iter().collect();

    assert!(rest.iter().all(|event| matches!(event, Loaded::Commits(_))));
}

fn identity_of(dir: &Path) -> Identity {
    identity(dir).unwrap()
}

fn folder_name(dir: &Path) -> String {
    dir.file_name().unwrap().to_string_lossy().into_owned()
}

#[test]
fn reads_the_folder_name_and_the_branch_of_a_repository() {
    let dir = TempDir::new();
    dir.git(&["init", "-q", "-b", "main"]);

    let found = identity_of(dir.path());

    assert_eq!(found.name, folder_name(dir.path()));
    assert_eq!(found.branch, "main");
}

#[test]
fn reads_the_root_name_from_a_subfolder() {
    let dir = TempDir::new();
    dir.git(&["init", "-q", "-b", "main"]);
    let sub = dir.path().join("a").join("b");
    std::fs::create_dir_all(&sub).unwrap();

    assert_eq!(identity_of(&sub).name, folder_name(dir.path()));
}

#[test]
fn reads_the_main_repository_name_from_a_linked_worktree() {
    let dir = TempDir::new();
    dir.git(&["init", "-q", "-b", "main"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "first"]);
    let linked = TempDir::new();
    let path = linked.path().join("linked-folder");
    dir.git(&["worktree", "add", "-q", "-b", "side", path.to_str().unwrap()]);

    let found = identity_of(&path);

    assert_eq!(found.name, folder_name(dir.path()));
    assert_eq!(found.branch, "side");
}

#[test]
fn reads_a_bare_repository_name_without_its_git_suffix() {
    let dir = TempDir::new();
    let bare = dir.path().join("foo.git");
    dir.git(&["init", "-q", "--bare", "-b", "main", bare.to_str().unwrap()]);

    let found = identity_of(&bare);

    assert_eq!(found.name, "foo");
    assert_eq!(found.branch, "main");
}

#[test]
fn reads_the_first_seven_characters_of_the_hash_of_a_detached_head() {
    let dir = TempDir::new();
    dir.git(&["init", "-q", "-b", "main"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "first"]);
    dir.git(&["tag", "v1"]);
    dir.git(&["checkout", "-q", "--detach"]);
    let output = std::process::Command::new("git")
        .current_dir(dir.path())
        .args(["rev-parse", "HEAD"])
        .env_remove("GIT_DIR")
        .output()
        .unwrap();
    let hash = String::from_utf8(output.stdout).unwrap();

    assert_eq!(identity_of(dir.path()).branch, hash[..7]);
}

#[test]
fn reads_the_name_of_a_branch_without_commit_while_other_refs_have_some() {
    let dir = TempDir::new();
    dir.git(&["init", "-q", "-b", "main"]);
    dir.git(&["commit", "-q", "--allow-empty", "-m", "first"]);
    dir.git(&["switch", "-q", "--orphan", "fresh"]);

    assert_eq!(identity_of(dir.path()).branch, "fresh");
}

#[test]
fn reading_the_identity_outside_a_repository_gives_gits_message() {
    let dir = TempDir::new();

    let message = identity(dir.path()).unwrap_err();

    assert!(message.contains("not a git repository"));
}
