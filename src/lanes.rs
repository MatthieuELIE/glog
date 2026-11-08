use std::cmp::Reverse;

use crate::parse::{Commit, RefKind};

const NODE: char = '●';
const HEAD_NODE: char = '○';
const BLANK: char = ' ';

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub symbol: char,
    pub color: usize,
}

#[derive(Debug)]
pub struct GraphRow {
    pub cells: Vec<Cell>,
    pub node_color: usize,
}

#[derive(Clone)]
struct Lane {
    expects: String,
    color: usize,
}

type Columns = Vec<Option<Lane>>;

const BLANK_CELL: Cell = Cell { symbol: BLANK, color: 0 };

pub fn place(commits: &[Commit]) -> Vec<GraphRow> {
    let mut columns: Columns = Vec::new();
    let mut next_color = 0;
    let mut rows = Vec::new();

    for commit in commits {
        let (row, next_columns, following_color) = place_row(&columns, next_color, commit);
        rows.push(row);
        columns = next_columns;
        next_color = following_color;
    }

    pad_rows(rows)
}

fn pad_rows(rows: Vec<GraphRow>) -> Vec<GraphRow> {
    let width = rows.iter().map(|row| row.cells.len()).max().unwrap_or(0);

    rows.into_iter()
        .map(|mut row| {
            row.cells.resize(width, BLANK_CELL);
            row
        })
        .collect()
}

fn leftmost_free(columns: &Columns) -> usize {
    columns.iter().position(Option::is_none).unwrap_or(columns.len())
}

fn claim(columns: &mut Columns, lane: Lane) -> usize {
    let column = leftmost_free(columns);
    if column == columns.len() {
        columns.push(None);
    }
    columns[column] = Some(lane);

    column
}

fn joining_columns(columns: &Columns, hash: &str) -> Vec<usize> {
    columns
        .iter()
        .enumerate()
        .filter(|(_, lane)| lane.as_ref().is_some_and(|lane| lane.expects == hash))
        .map(|(column, _)| column)
        .collect()
}

fn lane_color(columns: &Columns, column: usize) -> usize {
    columns[column].as_ref().map_or(0, |lane| lane.color)
}

fn place_row(columns: &Columns, next_color: usize, commit: &Commit) -> (GraphRow, Columns, usize) {
    let mut next = columns.clone();
    let mut next_color = next_color;
    let joining = joining_columns(columns, &commit.hash);

    let (node_column, color) = match joining.first() {
        Some(&column) => (column, lane_color(columns, column)),
        None => {
            let color = next_color;
            next_color += 1;
            (leftmost_free(columns), color)
        }
    };
    if node_column == next.len() {
        next.push(None);
    }
    next[node_column] = commit.parents.first().map(|parent| Lane {
        expects: parent.clone(),
        color,
    });

    let joined: Vec<(usize, usize)> = joining.iter().skip(1).map(|&column| (column, lane_color(columns, column))).collect();
    for &(column, _) in &joined {
        next[column] = None;
    }

    let mut opened = Vec::new();
    for parent in commit.parents.iter().skip(1) {
        let lane = Lane {
            expects: parent.clone(),
            color: next_color,
        };
        opened.push((claim(&mut next, lane), next_color));
        next_color += 1;
    }

    let cells = draw_cells(columns, next.len(), node_column, commit, color, &joining, &opened, &joined);

    while next.last().is_some_and(Option::is_none) {
        next.pop();
    }

    (GraphRow { cells, node_color: color }, next, next_color)
}

fn draw_cells(
    before: &Columns,
    after_len: usize,
    node_column: usize,
    commit: &Commit,
    color: usize,
    joining: &[usize],
    opened: &[(usize, usize)],
    joined: &[(usize, usize)],
) -> Vec<Cell> {
    let columns = before.len().max(after_len);
    let mut cells = vec![BLANK_CELL; (2 * columns).saturating_sub(1)];

    let ends = joined
        .iter()
        .map(|&(column, color)| (column, color, '╰', '╯'))
        .chain(opened.iter().map(|&(column, color)| {
            if joined.iter().any(|&(joined_column, _)| joined_column == column) {
                return (column, color, '├', '┤');
            }

            (column, color, '╭', '╮')
        }));
    let mut ends: Vec<_> = ends.collect();
    ends.sort_by_key(|&(column, ..)| Reverse(column.abs_diff(node_column)));

    for &(column, color, _, _) in &ends {
        let (from, to) = (node_column.min(column), node_column.max(column));
        for cell in &mut cells[2 * from + 1..2 * to] {
            *cell = Cell { symbol: '─', color };
        }
    }
    for (column, lane) in before.iter().enumerate() {
        if let Some(lane) = lane
            && !joining.contains(&column)
        {
            cells[2 * column] = Cell {
                symbol: '│',
                color: lane.color,
            };
        }
    }
    for &(column, color, left, right) in &ends {
        let symbol = if column < node_column { left } else { right };
        cells[2 * column] = Cell { symbol, color };
    }

    let symbol = if commit.refs.iter().any(|reference| reference.kind == RefKind::Head) {
        HEAD_NODE
    } else {
        NODE
    };
    cells[2 * node_column] = Cell { symbol, color };

    cells
}
