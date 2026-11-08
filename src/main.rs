use std::{io, path::Path, process::ExitCode};

use glog::{
    draw::render,
    parse::{Commit, parse},
    read::read_log,
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode},
};

fn event_loop(terminal: &mut DefaultTerminal, commits: &[Commit]) -> io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, commits))?;

        if let Event::Key(key) = event::read()?
            && key.code == KeyCode::Char('q')
        {
            return Ok(());
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
