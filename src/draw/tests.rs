use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

use crate::{draw::render, parse::parse};

const SEP: &str = "\x1f";

fn line(hash: &str, parents: &str, refs: &str, date: &str, author: &str, subject: &str) -> String {
    [hash, parents, refs, date, author, subject].join(SEP)
}

fn draw(log: &str, width: u16, height: u16) -> Buffer {
    let commits = parse(log);
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| render(frame, &commits)).unwrap();

    terminal.backend().buffer().clone()
}

fn row_text(buffer: &Buffer, y: u16) -> String {
    (0..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol().to_string())
        .collect()
}

#[test]
fn shows_one_row_per_commit_newest_first() {
    let log = [
        line("bbbbbbbb", "aaaaaaaa", "", "1 day ago", "Ann", "second"),
        line("aaaaaaaa", "", "", "2 days ago", "Ann", "first"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 3);

    assert!(row_text(&buffer, 0).contains("second"));
    assert!(row_text(&buffer, 1).contains("first"));
    assert_eq!(row_text(&buffer, 2).trim(), "");
}
