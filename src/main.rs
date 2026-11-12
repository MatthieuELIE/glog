use std::{
    io::{self, IsTerminal, Write},
    path::Path,
    process::ExitCode,
    sync::mpsc::{RecvTimeoutError, TryRecvError},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use glog::{
    draw::{max_offset, render, reveal_offset},
    history::History,
    nav::{self, Loading, View, fit, follow, list_height, page, press},
    read::{Loaded, Reader, start},
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode},
};

const SLOW_START: Duration = Duration::from_millis(500);
const POLL: Duration = Duration::from_millis(50);

fn clock_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX))
}

fn drain(reader: &Reader, history: &mut History, loading: Loading) -> Loading {
    let mut loading = loading;

    while matches!(loading, Loading::Reading) {
        match reader.events.try_recv() {
            Ok(Loaded::Commits(commits)) => commits.into_iter().for_each(|commit| history.push(commit)),
            Ok(Loaded::Done) => loading = Loading::Done,
            Ok(Loaded::Failed(message)) => loading = Loading::Failed(message),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => loading = Loading::Failed("git stopped unexpectedly".to_string()),
        }
    }

    loading
}

fn key_pressed(loading: &Loading) -> io::Result<Option<event::KeyEvent>> {
    if matches!(loading, Loading::Reading) && !event::poll(POLL)? {
        return Ok(None);
    }

    match event::read()? {
        Event::Key(key) => Ok(Some(key)),
        _ => Ok(None),
    }
}

fn event_loop(terminal: &mut DefaultTerminal, reader: &Reader, history: &mut History, now: i64) -> io::Result<()> {
    let mut view = View::default();
    let mut loading = Loading::Reading;

    loop {
        loading = drain(reader, history, loading);
        let size = terminal.size()?;
        view = fit(
            view,
            history.len(),
            list_height(usize::from(size.height)),
            max_offset(history.max_lanes(), usize::from(size.width)),
        );
        terminal.draw(|frame| render(frame, history, view, &loading, now))?;

        if let Some(key) = key_pressed(&loading)?
            && let KeyCode::Char(character) = key.code
        {
            if nav::quits(character, key.modifiers) {
                return Ok(());
            }
            let after = press(view, character, key.modifiers, history.len(), page(usize::from(size.height)));
            let node_column = history.lane_row(after.selected).node_column as usize;
            let node_offset = reveal_offset(after.offset, node_column, history.max_lanes(), usize::from(size.width));
            view = follow(view, after, node_offset);
        }
    }
}

fn first_event(reader: &Reader) -> Loaded {
    let waiting = io::stderr().is_terminal();
    let event = match reader.events.recv_timeout(SLOW_START) {
        Ok(event) => Ok(event),
        Err(RecvTimeoutError::Timeout) => {
            if waiting {
                eprint!("glog: reading history…");
                let _ = io::stderr().flush();
            }
            reader.events.recv()
        }
        Err(RecvTimeoutError::Disconnected) => Err(std::sync::mpsc::RecvError),
    };
    if waiting {
        eprint!("\r\x1b[2K");
    }

    event.unwrap_or_else(|_| Loaded::Failed("git stopped unexpectedly".to_string()))
}

#[derive(Parser)]
#[command(
    version,
    about = "A read-only, coloured git log graph for the terminal",
    after_help = "Keys:\n  j       Move the selection down\n  k       Move the selection up\n  h       Scroll the graph left\n  l       Scroll the graph right\n  g       Select the first commit\n  G       Select the last loaded commit\n  Ctrl+d  Move the selection down half a page\n  Ctrl+u  Move the selection up half a page\n  q       Quit\n  Ctrl+C  Quit"
)]
struct Cli {}

fn main() -> ExitCode {
    Cli::parse();

    let reader = match start(Path::new(".")) {
        Ok(reader) => reader,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    let mut history = History::default();
    match first_event(&reader) {
        Loaded::Commits(commits) => commits.into_iter().for_each(|commit| history.push(commit)),
        Loaded::Done => {
            eprintln!("glog: no commits yet");
            return ExitCode::FAILURE;
        }
        Loaded::Failed(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    }

    let now = clock_seconds();
    let result = ratatui::run(|terminal| event_loop(terminal, &reader, &mut history, now));
    reader.stop();

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
