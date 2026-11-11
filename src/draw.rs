use ratatui::{
    Frame,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListState},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    date::relative,
    lanes::{GraphRow, place},
    nav::View,
    parse::{Commit, Ref, RefKind},
    theme::{LANE_COLORS, LAVENDER, OVERLAY, SUBTEXT, SURFACE1, TEXT, YELLOW},
};

const HASH_WIDTH: usize = 7;
const DATE_WIDTH: usize = 4;
const AUTHOR_WIDTH: usize = 20;
const SEPARATORS_WIDTH: usize = 4;
const CELLS_PER_LANE: usize = 2;
const ELLIPSIS: char = '…';
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

pub fn graph_width(commits: &[Commit]) -> usize {
    place(commits).first().map_or(0, |row| row.cells.len())
}

fn window_width(graph_width: usize, terminal_width: usize) -> usize {
    graph_width.min(terminal_width / 3)
}

pub fn max_offset(graph_width: usize, terminal_width: usize) -> usize {
    graph_width
        .saturating_sub(window_width(graph_width, terminal_width))
        .div_ceil(CELLS_PER_LANE)
}

fn ellipsis_span() -> Span<'static> {
    Span::styled(ELLIPSIS.to_string(), Style::new().fg(OVERLAY))
}

fn graph_spans(row: &GraphRow, offset: usize, window_width: usize) -> Vec<Span<'static>> {
    let start = offset * CELLS_PER_LANE;
    let mut spans: Vec<Span<'static>> = row
        .cells
        .iter()
        .skip(start)
        .take(window_width)
        .map(|cell| Span::styled(cell.symbol.to_string(), Style::new().fg(LANE_COLORS[cell.color % LANE_COLORS.len()])))
        .collect();
    if offset > 0
        && let Some(first) = spans.first_mut()
    {
        *first = ellipsis_span();
    }

    if start + window_width < row.cells.len()
        && let Some(last) = spans.last_mut()
    {
        *last = ellipsis_span();
    }
    spans.push(Span::raw(" ".repeat(window_width.saturating_sub(spans.len()))));

    spans
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

fn commit_line(commit: &Commit, row: &GraphRow, offset: usize, window_width: usize, now: i64, message_width: usize) -> Line<'static> {
    let mut spans = graph_spans(row, offset, window_width);
    spans.extend([
        Span::raw(" "),
        Span::styled(commit.hash.chars().take(HASH_WIDTH).collect::<String>(), Style::new().fg(OVERLAY)),
        Span::raw(" "),
        Span::styled(pad(&relative(now, commit.time), DATE_WIDTH), Style::new().fg(SUBTEXT)),
        Span::raw(" "),
        Span::styled(pad(&truncate(&commit.author, AUTHOR_WIDTH), AUTHOR_WIDTH), Style::new().fg(LAVENDER)),
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

pub fn render(frame: &mut Frame, commits: &[Commit], view: View, now: i64) {
    let rows = place(commits);
    let graph_width = rows.first().map_or(0, |row| row.cells.len());
    let terminal_width = usize::from(frame.area().width);
    let window_width = window_width(graph_width, terminal_width);
    let fixed_width = window_width + HASH_WIDTH + DATE_WIDTH + AUTHOR_WIDTH + SEPARATORS_WIDTH;
    let message_width = terminal_width.saturating_sub(fixed_width);

    let lines: Vec<Line> = commits
        .iter()
        .zip(&rows)
        .skip(view.top)
        .take(usize::from(frame.area().height))
        .map(|(commit, row)| commit_line(commit, row, view.offset, window_width, now, message_width))
        .collect();

    let list = List::new(lines).highlight_style(Style::new().bg(SURFACE1));
    let mut state = ListState::default().with_selected(Some(view.selected.saturating_sub(view.top)));

    frame.render_stateful_widget(list, frame.area(), &mut state);
}

#[cfg(test)]
mod tests;
