use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, style::Color};

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

fn two_commits() -> String {
    [
        line("1234567890abcdef", "fedcba0987654321", "", "3 days ago", "Ann", "short subject"),
        line("fedcba0987654321", "", "", "2 weeks ago", "Bartholomew", "older"),
    ]
    .join("\n")
}

#[test]
fn aligns_graph_hash_date_author_and_message_columns() {
    let buffer = draw(&two_commits(), 60, 2);

    assert_eq!(row_text(&buffer, 0).trim_end(), "● 1234567 3 days ago  Ann         short subject");
    assert_eq!(row_text(&buffer, 1).trim_end(), "● fedcba0 2 weeks ago Bartholomew older");
}

#[test]
fn draws_head_as_a_hollow_node_and_other_commits_as_filled_nodes() {
    let log = [
        line("bbbbbbbb", "aaaaaaaa", "origin/main", "1 day ago", "Ann", "second"),
        line("aaaaaaaa", "", "HEAD -> main, tag: v1", "2 days ago", "Ann", "first"),
        line("cccccccc", "", "HEAD", "3 days ago", "Ann", "detached"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 3);

    assert!(row_text(&buffer, 0).starts_with('●'));
    assert!(row_text(&buffer, 1).starts_with('○'));
    assert!(row_text(&buffer, 2).starts_with('○'));
}

#[test]
fn aligns_columns_by_display_width_for_wide_characters() {
    let log = [
        line("1234567890abcdef", "", "", "3 days ago", "山田", "Wide"),
        line("fedcba0987654321", "", "", "3 days ago", "Ann", "Narrow"),
    ]
    .join("\n");

    let buffer = draw(&log, 40, 2);

    let column_of = |y: u16, symbol: &str| (0..40).find(|&x| buffer[(x, y)].symbol() == symbol);

    assert_eq!(column_of(0, "W"), column_of(1, "N"));
    assert!(column_of(0, "W").is_some());
}

#[test]
fn truncates_the_message_first_in_a_narrow_terminal() {
    let buffer = draw(&two_commits(), 38, 2);

    assert_eq!(row_text(&buffer, 0), "● 1234567 3 days ago  Ann         sho…");
}

#[test]
fn colours_hash_in_overlay_and_text_in_text_and_subtext() {
    let buffer = draw(&two_commits(), 60, 1);

    assert_eq!(buffer[(2, 0)].fg, Color::Rgb(0x6c, 0x70, 0x86));
    assert_eq!(buffer[(10, 0)].fg, Color::Rgb(0xa6, 0xad, 0xc8));
    assert_eq!(buffer[(34, 0)].fg, Color::Rgb(0xcd, 0xd6, 0xf4));
    assert_eq!(buffer[(34, 0)].bg, Color::Reset);
}
