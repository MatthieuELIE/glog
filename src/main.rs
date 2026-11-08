use std::{io, path::Path, process::ExitCode};

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

    loop {
        terminal.draw(|frame| render(frame, commits, &mut state))?;

        if let Event::Key(key) = event::read()?
            && let KeyCode::Char(character) = key.code
        {
            if character == 'q' {
                return Ok(());
            }
            nav::press(&mut state, character, commits.len());
        }
    }
}

fn main() -> ExitCode {
    let log = match read_log(Path::new(".")) {
        Ok(log) => log,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    let commits = parse(&log);
    match ratatui::run(|terminal| event_loop(terminal, &commits)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
