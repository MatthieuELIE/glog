use std::{io, path::Path, process::ExitCode};

use clap::Parser;
use glog::{
    draw::render,
    nav,
    parse::{Commit, parse},
    read::read_log,
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode},
    widgets::ListState,
};

fn event_loop(terminal: &mut DefaultTerminal, commits: &[Commit]) -> io::Result<()> {
    let mut state = ListState::default().with_selected(Some(0));

    let mut offset = 0;

    loop {
        terminal.draw(|frame| render(frame, commits, &mut state, &mut offset))?;

        if let Event::Key(key) = event::read()?
            && let KeyCode::Char(character) = key.code
        {
            if nav::quits(character, key.modifiers) {
                return Ok(());
            }
            nav::press(&mut state, &mut offset, character, commits.len());
        }
    }
}

#[derive(Parser)]
#[command(
    version,
    about = "A read-only, coloured git log graph for the terminal",
    after_help = "Keys:\n  j       Move the selection down\n  k       Move the selection up\n  q       Quit\n  Ctrl+C  Quit"
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

    match ratatui::run(|terminal| event_loop(terminal, &commits)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
