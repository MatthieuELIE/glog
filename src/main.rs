use std::{
    io,
    path::Path,
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use glog::{
    draw::{graph_width, max_offset, render},
    nav::{self, View, fit, press},
    parse::{Commit, parse},
    read::read_log,
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode},
};

fn clock_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| i64::try_from(elapsed.as_secs()).unwrap_or(i64::MAX))
}

fn event_loop(terminal: &mut DefaultTerminal, commits: &[Commit], now: i64) -> io::Result<()> {
    let graph_width = graph_width(commits);

    let mut view = View::default();

    loop {
        let size = terminal.size()?;
        view = fit(
            view,
            commits.len(),
            usize::from(size.height),
            max_offset(graph_width, usize::from(size.width)),
        );
        terminal.draw(|frame| render(frame, commits, view, now))?;

        if let Event::Key(key) = event::read()?
            && let KeyCode::Char(character) = key.code
        {
            if nav::quits(character, key.modifiers) {
                return Ok(());
            }
            view = press(view, character, commits.len());
        }
    }
}

#[derive(Parser)]
#[command(
    version,
    about = "A read-only, coloured git log graph for the terminal",
    after_help = "Keys:\n  j       Move the selection down\n  k       Move the selection up\n  h       Scroll the graph left\n  l       Scroll the graph right\n  q       Quit\n  Ctrl+C  Quit"
)]
struct Cli {}

fn main() -> ExitCode {
    Cli::parse();

    let log = match read_log(Path::new(".")) {
        Ok(log) => log,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    let commits = parse(&log);
    if commits.is_empty() {
        eprintln!("glog: no commits yet");
        return ExitCode::FAILURE;
    }

    let now = clock_seconds();

    match ratatui::run(|terminal| event_loop(terminal, &commits, now)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
