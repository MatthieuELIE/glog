use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::{
    date::relative,
    history::{History, SHORT_HASH_WIDTH},
    lanes::{Cell, advance, cells},
    nav::{Loading, Session},
    parse::{Ref, RefKind},
    read::Identity,
    search::Search,
    theme::{BASE, LANE_COLORS, OVERLAY, RED, SUBTEXT, SURFACE0, SURFACE1, TEXT, YELLOW},
};

const DATE_WIDTH: usize = 4;
const AUTHOR_WIDTH: usize = 15;
const SEPARATORS_WIDTH: usize = 4;
const CELLS_PER_LANE: usize = 2;
const ELLIPSIS: char = '…';
const STATUS_SEPARATOR: &str = "  ";
const NO_MATCH: &str = "no match";
const EDGE_CELLS: usize = 2;
const MIN_DIMMED_WINDOW: usize = 5;

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
    graph_width.min(terminal_width / 4)
}

pub fn max_offset(max_lanes: usize, terminal_width: usize) -> usize {
    let graph_width = graph_width(max_lanes);

    graph_width
        .saturating_sub(window_width(graph_width, terminal_width))
        .div_ceil(CELLS_PER_LANE)
}

pub fn reveal_offset(offset: usize, node_column: usize, max_lanes: usize, terminal_width: usize) -> usize {
    let graph_width = graph_width(max_lanes);
    let window_width = window_width(graph_width, terminal_width);
    let cell = node_column * CELLS_PER_LANE;
    let (first, last) = reveal_range(cell, graph_width, window_width);

    offset.max(first).min(last).min(max_offset(max_lanes, terminal_width))
}

fn reveal_range(cell: usize, graph_width: usize, window_width: usize) -> (usize, usize) {
    if window_width < MIN_DIMMED_WINDOW {
        return ((cell + 1).saturating_sub(window_width).div_ceil(CELLS_PER_LANE), cell / CELLS_PER_LANE);
    }

    let without_right_edge = graph_width.saturating_sub(window_width);
    let with_right_edge = (cell + 1 + EDGE_CELLS).saturating_sub(window_width);

    (
        without_right_edge.min(with_right_edge).div_ceil(CELLS_PER_LANE),
        cell.saturating_sub(EDGE_CELLS) / CELLS_PER_LANE,
    )
}

fn dim<'a>(spans: impl Iterator<Item = &'a mut Span<'static>>) {
    for span in spans {
        span.style = Style::new().fg(SURFACE1);
    }
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
    if window_width >= MIN_DIMMED_WINDOW {
        if offset > 0 {
            dim(spans.iter_mut().take(EDGE_CELLS));
        }

        if start + window_width < graph_width {
            dim(spans.iter_mut().rev().take(EDGE_CELLS));
        }
    }
    spans.push(Span::raw(" ".repeat(window_width.saturating_sub(spans.len()))));

    spans
}

fn badge_style(reference: &Ref, node_color: u32) -> Style {
    match reference.kind {
        RefKind::Head => Style::new().fg(BASE).bg(TEXT).add_modifier(Modifier::BOLD),
        RefKind::HeadBranch => Style::new().fg(BASE).bg(lane_color(node_color)).add_modifier(Modifier::BOLD),
        RefKind::Local => Style::new().fg(BASE).bg(lane_color(node_color)),
        RefKind::Remote => Style::new().fg(SUBTEXT).bg(SURFACE1),
        RefKind::Tag => Style::new().fg(BASE).bg(YELLOW),
    }
}

fn badge_label(reference: &Ref) -> String {
    match reference.kind {
        RefKind::Tag => format!(" ◆ {} ", reference.name),
        _ => format!(" {} ", reference.name),
    }
}

fn badge_spans(refs: &[Ref], node_color: u32) -> Vec<Span<'static>> {
    refs.iter()
        .flat_map(|reference| [Span::styled(badge_label(reference), badge_style(reference, node_color)), Span::raw(" ")])
        .collect()
}

fn subject_style(selected: bool) -> Style {
    let style = Style::new().fg(TEXT);
    if selected {
        return style.add_modifier(Modifier::BOLD);
    }

    style
}

fn commit_line(
    history: &History,
    row: usize,
    graph: Vec<Span<'static>>,
    node_color: u32,
    now: i64,
    message_width: usize,
    selected: bool,
) -> Line<'static> {
    let mut spans = graph;
    spans.extend([
        Span::raw(" "),
        Span::styled(history.short_hash(row).to_string(), Style::new().fg(OVERLAY)),
        Span::raw(" "),
        Span::styled(pad(&relative(now, history.time(row)), DATE_WIDTH), Style::new().fg(OVERLAY)),
        Span::raw(" "),
        Span::styled(pad(&truncate(history.author(row), AUTHOR_WIDTH), AUTHOR_WIDTH), Style::new().fg(SUBTEXT)),
        Span::raw(" "),
    ]);
    let badges = badge_spans(history.refs(row), node_color);
    let badges_width: usize = badges.iter().map(Span::width).sum();
    spans.extend(badges);
    spans.push(Span::styled(
        truncate(history.subject(row), message_width.saturating_sub(badges_width)),
        subject_style(selected),
    ));

    Line::from(spans)
}

fn is_head(refs: &[Ref]) -> bool {
    refs.iter().any(|reference| matches!(reference.kind, RefKind::Head | RefKind::HeadBranch))
}

fn thousands(number: usize) -> String {
    let digits = number.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }

    grouped
}

fn repo_width(identity: &Identity) -> usize {
    if identity.name.is_empty() {
        return 0;
    }

    identity.name.width() + STATUS_SEPARATOR.width()
}

fn branch_width(identity: &Identity) -> usize {
    if identity.branch.is_empty() {
        return 0;
    }

    pill_label(&identity.branch).width() + STATUS_SEPARATOR.width()
}

fn pill_label(name: &str) -> String {
    format!(" {name} ")
}

fn shown_parts(identity: &Identity, tail_width: usize, width: usize) -> (bool, bool) {
    if repo_width(identity) + branch_width(identity) + tail_width <= width {
        return (true, true);
    }

    (false, branch_width(identity) + tail_width <= width)
}

fn identity_spans(identity: &Identity, width: usize, tail_width: usize) -> Vec<Span<'static>> {
    let (show_repo, show_branch) = shown_parts(identity, tail_width, width);
    let mut spans = Vec::new();
    if show_repo && repo_width(identity) > 0 {
        spans.push(Span::styled(identity.name.clone(), Style::new().fg(SUBTEXT)));
        spans.push(Span::raw(STATUS_SEPARATOR));
    }
    if show_branch && branch_width(identity) > 0 {
        let pill = Style::new().fg(BASE).bg(TEXT).add_modifier(Modifier::BOLD);
        spans.push(Span::styled(pill_label(&identity.branch), pill));
        spans.push(Span::raw(STATUS_SEPARATOR));
    }

    spans
}

fn status_line(identity: &Identity, rank: usize, total: usize, loading: &Loading, width: usize, reserved: usize) -> Line<'static> {
    if let Loading::Failed(message) = loading {
        let first_line = message.lines().next().unwrap_or_default();
        let mut spans = identity_spans(identity, width, first_line.width() + reserved);
        let room = width.saturating_sub(spans.iter().map(Span::width).sum::<usize>() + reserved);
        spans.push(Span::styled(truncate(first_line, room), Style::new().fg(RED)));
        if reserved > 0 {
            return Line::from(spans).right_aligned();
        }

        return Line::from(spans);
    }

    let mut position = format!("{}/{}", thousands(rank), thousands(total));
    if matches!(loading, Loading::Reading) {
        position.push(ELLIPSIS);
    }
    let mut spans = identity_spans(identity, width, position.width() + reserved);
    spans.push(Span::styled(position, Style::new().fg(OVERLAY)));

    Line::from(spans).right_aligned()
}

struct Left {
    query: String,
    counter: Option<(String, Color)>,
}

fn counter(search: &Search, selected: usize) -> (String, Color) {
    if search.matches.is_empty() {
        return (NO_MATCH.to_string(), RED);
    }

    let total = thousands(search.matches.len());
    match search.matches.binary_search(&selected) {
        Ok(index) => (format!("{}/{total}", thousands(index + 1)), OVERLAY),
        Err(_) => (total, OVERLAY),
    }
}

fn left_of(session: &Session) -> Option<Left> {
    if let Some(input) = &session.input {
        return Some(Left {
            query: format!("/{input}"),
            counter: None,
        });
    }

    let search = session.search.as_ref()?;

    Some(Left {
        query: format!("/{}", search.query),
        counter: Some(counter(search, session.view.selected)),
    })
}

fn counter_width(left: &Left) -> usize {
    left.counter.as_ref().map_or(0, |(counter, _)| STATUS_SEPARATOR.width() + counter.width())
}

fn left_line(left: &Left, room: usize) -> Line<'static> {
    let query = truncate(&left.query, room.saturating_sub(counter_width(left)));
    let mut spans = vec![Span::styled(query, Style::new().fg(TEXT))];
    if let Some((counter, color)) = &left.counter {
        spans.push(Span::raw(STATUS_SEPARATOR));
        spans.push(Span::styled(counter.clone(), Style::new().fg(*color)));
    }

    Line::from(spans)
}

pub fn render(frame: &mut Frame, history: &History, session: &Session, loading: &Loading, identity: &Identity, now: i64) {
    let view = session.view;
    let full = frame.area();
    let [list_area, status_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(full);
    let rank = (view.selected + 1).min(history.len());
    let left = left_of(session);
    let reserved = left
        .as_ref()
        .map_or(0, |left| left.query.width() + counter_width(left) + STATUS_SEPARATOR.width());
    let status = status_line(identity, rank, history.len(), loading, usize::from(full.width), reserved);
    if let Some(left) = &left {
        let room = usize::from(full.width).saturating_sub(status.width() + STATUS_SEPARATOR.width());
        let line = left_line(left, room);
        if session.input.is_some() {
            frame.set_cursor_position((status_area.x + u16::try_from(line.width()).unwrap_or(u16::MAX), status_area.y));
        }
        frame.render_widget(Paragraph::new(line), status_area);
    }
    frame.render_widget(Paragraph::new(status), status_area);

    let graph_width = graph_width(history.max_lanes());
    let terminal_width = usize::from(full.width);
    let window_width = window_width(graph_width, terminal_width);
    let fixed_width = window_width + SHORT_HASH_WIDTH + DATE_WIDTH + AUTHOR_WIDTH + SEPARATORS_WIDTH;
    let message_width = terminal_width.saturating_sub(fixed_width);
    let visible = view.top..history.len().min(view.top + usize::from(list_area.height));
    let drawn_cells = window_width.min(graph_width.saturating_sub(view.offset * CELLS_PER_LANE));

    let mut lanes = history.lanes_before(view.top);
    let mut items = Vec::new();
    for row in visible {
        let lane_row = history.lane_row(row);
        let cells = cells(&lanes, &lane_row, is_head(history.refs(row)), view.offset, drawn_cells);
        let graph = graph_spans(&cells, view.offset, graph_width, window_width);
        let selected = row == view.selected;
        let mut item = ListItem::new(commit_line(history, row, graph, lane_row.color, now, message_width, selected));
        if selected {
            item = item.style(Style::new().bg(SURFACE0));
        }
        items.push(item);
        lanes = advance(lanes, &lane_row);
    }

    frame.render_widget(List::new(items), list_area);
}

#[cfg(test)]
mod tests;
