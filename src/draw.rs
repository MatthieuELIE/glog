use ratatui::{
    Frame,
    style::Style,
    text::{Line, Span},
    widgets::{List, ListState},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    parse::Commit,
    theme::{MAUVE, OVERLAY, SUBTEXT, SURFACE0, TEXT},
};

const NODE: &str = "●";
const HEAD_NODE: &str = "○";
const HASH_WIDTH: usize = 7;
const SEPARATORS_WIDTH: usize = 4;
const ELLIPSIS: char = '…';

fn column_width(commits: &[Commit], field: fn(&Commit) -> &str) -> usize {
    commits
        .iter()
        .map(|commit| field(commit).width())
        .max()
        .unwrap_or(0)
}

fn node(commit: &Commit) -> &'static str {
    if commit.is_head {
        return HEAD_NODE;
    }

    NODE
}

fn pad(text: &str, width: usize) -> String {
    let padding = width.saturating_sub(text.width());

    format!("{text}{}", " ".repeat(padding))
}

fn truncate(text: &str, width: usize) -> String {
    if text.width() <= width {
        return text.to_string();
    }

    let mut kept = String::new();
    let mut kept_width = 0;
    for character in text.chars() {
        let character_width = character.width().unwrap_or(0);
        if kept_width + character_width >= width {
            break;
        }
        kept.push(character);
        kept_width += character_width;
    }

    if width > 0 {
        kept.push(ELLIPSIS);
    }

    kept
}

fn commit_line(
    commit: &Commit,
    date_width: usize,
    author_width: usize,
    message_width: usize,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(node(commit), Style::new().fg(MAUVE)),
        Span::raw(" "),
        Span::styled(commit.hash.clone(), Style::new().fg(OVERLAY)),
        Span::raw(" "),
        Span::styled(pad(&commit.date, date_width), Style::new().fg(SUBTEXT)),
        Span::raw(" "),
        Span::styled(pad(&commit.author, author_width), Style::new().fg(SUBTEXT)),
        Span::raw(" "),
        Span::styled(
            truncate(&commit.subject, message_width),
            Style::new().fg(TEXT),
        ),
    ])
}

pub fn render(frame: &mut Frame, commits: &[Commit], state: &mut ListState) {
    let date_width = column_width(commits, |commit| &commit.date);
    let author_width = column_width(commits, |commit| &commit.author);
    let fixed_width = 1 + HASH_WIDTH + date_width + author_width + SEPARATORS_WIDTH;
    let message_width = usize::from(frame.area().width).saturating_sub(fixed_width);

    let lines: Vec<Line> = commits
        .iter()
        .map(|commit| commit_line(commit, date_width, author_width, message_width))
        .collect();

    let list = List::new(lines).highlight_style(Style::new().bg(SURFACE0));

    frame.render_stateful_widget(list, frame.area(), state);
}

#[cfg(test)]
mod tests;
