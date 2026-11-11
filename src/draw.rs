use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListState, Paragraph},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    date::relative,
    history::{History, SHORT_HASH_WIDTH},
    lanes::{Cell, advance, cells},
    nav::{Loading, View},
    parse::{Ref, RefKind},
    theme::{LANE_COLORS, LAVENDER, OVERLAY, RED, SUBTEXT, SURFACE1, TEXT, YELLOW},
};

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

fn graph_width(max_lanes: usize) -> usize {
    (CELLS_PER_LANE * max_lanes).saturating_sub(1)
}

fn window_width(graph_width: usize, terminal_width: usize) -> usize {
    graph_width.min(terminal_width / 3)
}

pub fn max_offset(max_lanes: usize, terminal_width: usize) -> usize {
    let graph_width = graph_width(max_lanes);

    graph_width
        .saturating_sub(window_width(graph_width, terminal_width))
        .div_ceil(CELLS_PER_LANE)
}

fn ellipsis_span() -> Span<'static> {
    Span::styled(ELLIPSIS.to_string(), Style::new().fg(OVERLAY))
}

fn lane_color(color: u32) -> Color {
    LANE_COLORS[color as usize % LANE_COLORS.len()]
}

fn graph_spans(cells: &[Cell], offset: usize, graph_width: usize, window_width: usize) -> Vec<Span<'static>> {
    let start = offset * CELLS_PER_LANE;
    let mut spans: Vec<Span<'static>> = cells
        .iter()
        .map(|cell| Span::styled(cell.symbol.to_string(), Style::new().fg(lane_color(cell.color))))
        .collect();
    if offset > 0
        && let Some(first) = spans.first_mut()
    {
        *first = ellipsis_span();
    }

    if start + window_width < graph_width
        && let Some(last) = spans.last_mut()
    {
        *last = ellipsis_span();
    }
    spans.push(Span::raw(" ".repeat(window_width.saturating_sub(spans.len()))));

    spans
}

fn badge_style(reference: &Ref, node_color: u32) -> Style {
    match reference.kind {
        RefKind::Head => Style::new().fg(TEXT).add_modifier(Modifier::BOLD),
        RefKind::Local => Style::new().fg(lane_color(node_color)),
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

fn badge_spans(refs: &[Ref], node_color: u32) -> Vec<Span<'static>> {
    refs.iter()
        .flat_map(|reference| [Span::styled(badge_label(reference), badge_style(reference, node_color)), Span::raw(" ")])
        .collect()
}

fn commit_line(history: &History, row: usize, graph: Vec<Span<'static>>, node_color: u32, now: i64, message_width: usize) -> Line<'static> {
    let mut spans = graph;
    spans.extend([
        Span::raw(" "),
        Span::styled(history.short_hash(row).to_string(), Style::new().fg(OVERLAY)),
        Span::raw(" "),
        Span::styled(pad(&relative(now, history.time(row)), DATE_WIDTH), Style::new().fg(SUBTEXT)),
        Span::raw(" "),
        Span::styled(pad(&truncate(history.author(row), AUTHOR_WIDTH), AUTHOR_WIDTH), Style::new().fg(LAVENDER)),
        Span::raw(" "),
    ]);
    let badges = badge_spans(history.refs(row), node_color);
    let badges_width: usize = badges.iter().map(Span::width).sum();
    spans.extend(badges);
    spans.push(Span::styled(
        truncate(history.subject(row), message_width.saturating_sub(badges_width)),
        Style::new().fg(TEXT),
    ));

    Line::from(spans)
}

fn is_head(refs: &[Ref]) -> bool {
    refs.iter().any(|reference| reference.kind == RefKind::Head)
}

fn status_line(history: &History, loading: &Loading) -> Option<Span<'static>> {
    match loading {
        Loading::Done => None,
        Loading::Reading => Some(Span::styled(format!("loading… {} commits", history.len()), Style::new().fg(OVERLAY))),
        Loading::Failed(message) => Some(Span::styled(message.lines().next().unwrap_or_default().to_string(), Style::new().fg(RED))),
    }
}

pub fn render(frame: &mut Frame, history: &History, view: View, loading: &Loading, now: i64) {
    let full = frame.area();
    let mut list_area = full;
    if let Some(span) = status_line(history, loading) {
        let [list, status] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(full);
        list_area = list;
        frame.render_widget(Paragraph::new(Line::from(span)), status);
    }

    let graph_width = graph_width(history.max_lanes());
    let terminal_width = usize::from(full.width);
    let window_width = window_width(graph_width, terminal_width);
    let fixed_width = window_width + SHORT_HASH_WIDTH + DATE_WIDTH + AUTHOR_WIDTH + SEPARATORS_WIDTH;
    let message_width = terminal_width.saturating_sub(fixed_width);
    let visible = view.top..history.len().min(view.top + usize::from(list_area.height));
    let drawn_cells = window_width.min(graph_width.saturating_sub(view.offset * CELLS_PER_LANE));

    let mut lanes = history.lanes_before(view.top);
    let mut lines = Vec::new();
    for row in visible {
        let lane_row = history.lane_row(row);
        let cells = cells(&lanes, &lane_row, is_head(history.refs(row)), view.offset, drawn_cells);
        let graph = graph_spans(&cells, view.offset, graph_width, window_width);
        lines.push(commit_line(history, row, graph, lane_row.color, now, message_width));
        lanes = advance(lanes, &lane_row);
    }

    let list = List::new(lines).highlight_style(Style::new().bg(SURFACE1));
    let mut state = ListState::default().with_selected(Some(view.selected.saturating_sub(view.top)));

    frame.render_stateful_widget(list, list_area, &mut state);
}

#[cfg(test)]
mod tests;
