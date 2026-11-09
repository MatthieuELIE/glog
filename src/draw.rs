use ratatui::{
    Frame,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListState},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    lanes::{GraphRow, place},
    parse::{Commit, Ref, RefKind},
    theme::{LANE_COLORS, LAVENDER, OVERLAY, SUBTEXT, SURFACE1, TEXT, YELLOW},
};

const HASH_WIDTH: usize = 7;
const SEPARATORS_WIDTH: usize = 4;
const ELLIPSIS: char = '…';
fn column_width(commits: &[Commit], field: fn(&Commit) -> &str) -> usize {
    commits.iter().map(|commit| field(commit).width()).max().unwrap_or(0)
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

fn graph_spans(row: &GraphRow, window_width: usize) -> Vec<Span<'static>> {
    row.cells
        .iter()
        .take(window_width)
        .map(|cell| Span::styled(cell.symbol.to_string(), Style::new().fg(LANE_COLORS[cell.color % LANE_COLORS.len()])))
        .collect()
}

fn badge_style(reference: &Ref, node_color: usize) -> Style {
    match reference.kind {
        RefKind::Head => Style::new().fg(TEXT).add_modifier(Modifier::BOLD),
        RefKind::Local => Style::new().fg(LANE_COLORS[node_color % LANE_COLORS.len()]),
        RefKind::Remote => Style::new().fg(OVERLAY),
        RefKind::Tag => Style::new().fg(YELLOW),
    }
}

fn badge_label(reference: &Ref) -> String {
    match reference.kind {
        RefKind::Tag => format!("[◆ {}]", reference.name),
        _ => format!("[{}]", reference.name),
    }
}

fn badge_spans(commit: &Commit, row: &GraphRow) -> Vec<Span<'static>> {
    commit
        .refs
        .iter()
        .flat_map(|reference| {
            [
                Span::styled(badge_label(reference), badge_style(reference, row.node_color)),
                Span::raw(" "),
            ]
        })
        .collect()
}

fn commit_line(commit: &Commit, row: &GraphRow, window_width: usize, date_width: usize, author_width: usize, message_width: usize) -> Line<'static> {
    let mut spans = graph_spans(row, window_width);
    spans.extend([
        Span::raw(" "),
        Span::styled(commit.hash.clone(), Style::new().fg(OVERLAY)),
        Span::raw(" "),
        Span::styled(pad(&commit.date, date_width), Style::new().fg(SUBTEXT)),
        Span::raw(" "),
        Span::styled(pad(&commit.author, author_width), Style::new().fg(LAVENDER)),
        Span::raw(" "),
    ]);
    let badges = badge_spans(commit, row);
    let badges_width: usize = badges.iter().map(Span::width).sum();
    spans.extend(badges);
    spans.push(Span::styled(
        truncate(&commit.subject, message_width.saturating_sub(badges_width)),
        Style::new().fg(TEXT),
    ));

    Line::from(spans)
}

pub fn render(frame: &mut Frame, commits: &[Commit], state: &mut ListState) {
    let date_width = column_width(commits, |commit| &commit.date);
    let author_width = column_width(commits, |commit| &commit.author);
    let rows = place(commits);
    let graph_width = rows.first().map_or(0, |row| row.cells.len());
    let terminal_width = usize::from(frame.area().width);
    let window_width = graph_width.min(terminal_width / 3);
    let fixed_width = window_width + HASH_WIDTH + date_width + author_width + SEPARATORS_WIDTH;
    let message_width = terminal_width.saturating_sub(fixed_width);

    let lines: Vec<Line> = commits
        .iter()
        .zip(&rows)
        .map(|(commit, row)| commit_line(commit, row, window_width, date_width, author_width, message_width))
        .collect();

    let list = List::new(lines).highlight_style(Style::new().bg(SURFACE1));

    frame.render_stateful_widget(list, frame.area(), state);
}

#[cfg(test)]
mod tests;
