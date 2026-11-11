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

use crate::parse::{Commit, parse_commit};

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
