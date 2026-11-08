use ratatui::{
    Frame,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    parse::Commit,
    theme::{MAUVE, OVERLAY, SUBTEXT, TEXT},
};

const NODE: &str = "●";
const HASH_WIDTH: usize = 7;
const SEPARATORS_WIDTH: usize = 4;
const ELLIPSIS: char = '…';

fn column_width(commits: &[Commit], field: fn(&Commit) -> &str) -> usize {
    commits
        .iter()
        .map(|commit| field(commit).chars().count())
        .max()
        .unwrap_or(0)
}

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }

    let kept: String = text.chars().take(width.saturating_sub(1)).collect();

    format!("{kept}{ELLIPSIS}")
}

fn commit_line(commit: &Commit, date_width: usize, author_width: usize, message_width: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled(NODE, Style::new().fg(MAUVE)),
        Span::raw(" "),
        Span::styled(commit.hash.clone(), Style::new().fg(OVERLAY)),
        Span::raw(" "),
        Span::styled(format!("{:<date_width$}", commit.date), Style::new().fg(SUBTEXT)),
        Span::raw(" "),
        Span::styled(format!("{:<author_width$}", commit.author), Style::new().fg(SUBTEXT)),
        Span::raw(" "),
        Span::styled(truncate(&commit.subject, message_width), Style::new().fg(TEXT)),
    ])
}

pub fn render(frame: &mut Frame, commits: &[Commit]) {
    let date_width = column_width(commits, |commit| &commit.date);
    let author_width = column_width(commits, |commit| &commit.author);
    let fixed_width = 1 + HASH_WIDTH + date_width + author_width + SEPARATORS_WIDTH;
    let message_width = usize::from(frame.area().width).saturating_sub(fixed_width);

    let lines: Vec<Line> = commits
        .iter()
        .map(|commit| commit_line(commit, date_width, author_width, message_width))
        .collect();

    frame.render_widget(Paragraph::new(lines), frame.area());
}

#[cfg(test)]
mod tests;
