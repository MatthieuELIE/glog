use std::{
    io::{BufRead, BufReader, Read},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, channel},
    },
    thread,
    time::Duration,
};

use crate::{
    history::SHORT_HASH_WIDTH,
    parse::{Commit, parse_commit},
};

const LOG_FORMAT: &str = "--format=%H%x1f%P%x1f%D%x1f%at%x1f%an%x1f%s";
const FIRST_BATCH: usize = 256;
const BATCH: usize = 4096;
const WAIT_POLL: Duration = Duration::from_millis(5);

pub enum Loaded {
    Commits(Vec<Commit>),
    Done,
    Failed(String),
}

pub struct Reader {
    pub events: Receiver<Loaded>,
    child: Arc<Mutex<Child>>,
    stopped: Arc<AtomicBool>,
}

impl Reader {
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
        }
    }
}

fn batch_size(sent: usize) -> usize {
    if sent == 0 {
        return FIRST_BATCH;
    }

    BATCH
}

fn stream_commits(stdout: impl Read, events: &Sender<Loaded>) -> bool {
    let mut lines = BufReader::new(stdout);
    let mut line = Vec::new();
    let mut batch = Vec::new();
    let mut sent = 0;

    loop {
        line.clear();
        match lines.read_until(b'\n', &mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let text = String::from_utf8_lossy(&line);
        if let Some(commit) = parse_commit(text.trim_end_matches('\n')) {
            batch.push(commit);
        }
        if batch.len() >= batch_size(sent) {
            sent += batch.len();
            if events.send(Loaded::Commits(std::mem::take(&mut batch))).is_err() {
                return false;
            }
        }
    }

    batch.is_empty() || events.send(Loaded::Commits(batch)).is_ok()
}

fn finish(child: &Mutex<Child>, mut stderr: impl Read) -> Loaded {
    let mut message = String::new();
    let _ = stderr.read_to_string(&mut message);
    let status = loop {
        let Ok(waited) = child.lock().map(|mut child| child.try_wait()) else {
            return Loaded::Failed("git reader panicked".to_string());
        };
        match waited {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => thread::sleep(WAIT_POLL),
            Err(error) => break Err(error),
        }
    };

    match status {
        Ok(status) if status.success() => Loaded::Done,
        Ok(status) if message.trim().is_empty() => Loaded::Failed(format!("git exited with {status}")),
        Ok(_) => Loaded::Failed(message.trim_end().to_string()),
        Err(error) => Loaded::Failed(format!("cannot wait for git: {error}")),
    }
}

#[derive(Debug, Default)]
pub struct Identity {
    pub name: String,
    pub branch: String,
}

fn repo_name(common_dir: &Path) -> String {
    let folder = |path: &Path| path.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    if common_dir.file_name().is_some_and(|name| name == ".git") {
        return common_dir.parent().map(folder).unwrap_or_default();
    }

    let name = folder(common_dir);

    name.strip_suffix(".git").map(str::to_string).unwrap_or(name)
}

fn git_line(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(dir)
        .env("LC_ALL", "C")
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("cannot run git: {error}"))?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr);

        return Err(message.trim_end().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim_end().to_string())
}

pub fn identity(dir: &Path) -> Result<Identity, String> {
    let common_dir = git_line(dir, &["rev-parse", "--path-format=absolute", "--git-common-dir"])?;
    let branch = match git_line(dir, &["symbolic-ref", "-q", "--short", "HEAD"]) {
        Ok(branch) => branch,
        Err(message) if message.is_empty() => git_line(dir, &["rev-parse", "HEAD"])?.chars().take(SHORT_HASH_WIDTH).collect(),
        Err(message) => return Err(message),
    };

    Ok(Identity {
        name: repo_name(Path::new(&common_dir)),
        branch,
    })
}

pub fn start(dir: &Path) -> Result<Reader, String> {
    let mut child = Command::new("git")
        .current_dir(dir)
        .env("LC_ALL", "C")
        .args(["log", "--all", "--topo-order", "--decorate=full", LOG_FORMAT])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot run git: {error}"))?;
    let stdout = child.stdout.take().ok_or("cannot read git output")?;
    let stderr = child.stderr.take().ok_or("cannot read git errors")?;
    let child = Arc::new(Mutex::new(child));
    let stopped = Arc::new(AtomicBool::new(false));
    let (sender, events) = channel();

    let thread_child = Arc::clone(&child);
    let thread_stopped = Arc::clone(&stopped);
    thread::spawn(move || {
        let listening = stream_commits(stdout, &sender);
        let last = finish(&thread_child, stderr);
        if listening && !thread_stopped.load(Ordering::SeqCst) {
            let _ = sender.send(last);
        }
    });

    Ok(Reader { events, child, stopped })
}
