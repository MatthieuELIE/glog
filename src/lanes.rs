use std::collections::{BTreeSet, HashMap};

use crate::parse::Commit;

const NODE: char = '●';
const HEAD_NODE: char = '○';
const BLANK: char = ' ';

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub symbol: char,
    pub color: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Junction {
    pub column: u32,
    pub color: u32,
    pub opens: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LaneRow {
    pub node_column: u32,
    pub color: u32,
    pub keeps_lane: bool,
    pub junctions: Vec<Junction>,
    pub width: u32,
}

type Columns = Vec<Option<u32>>;

const BLANK_CELL: Cell = Cell { symbol: BLANK, color: 0 };

#[derive(Default)]
pub struct Placer {
    columns: Columns,
    waiting: HashMap<String, Vec<u32>>,
    free: BTreeSet<u32>,
    next_color: u32,
}

fn column_index(column: u32) -> usize {
    column as usize
}

fn column_number(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

impl Placer {
    pub fn lanes(&self) -> Columns {
        self.columns.clone()
    }

    fn take_color(&mut self) -> u32 {
        let color = self.next_color;
        self.next_color += 1;

        color
    }

    fn leftmost_free(&mut self) -> u32 {
        if let Some(column) = self.free.pop_first() {
            return column;
        }
        self.columns.push(None);

        column_number(self.columns.len() - 1)
    }

    fn occupy(&mut self, column: u32, color: u32, expects: &str) {
        self.columns[column_index(column)] = Some(color);
        self.free.remove(&column);
        self.waiting.entry(expects.to_string()).or_default().push(column);
    }

    fn release(&mut self, column: u32) {
        self.columns[column_index(column)] = None;
        self.free.insert(column);
    }

    fn lane_color(&self, column: u32) -> u32 {
        self.columns[column_index(column)].unwrap_or(0)
    }

    fn trim(&mut self) {
        while self.columns.last().is_some_and(Option::is_none) {
            self.columns.pop();
            self.free.remove(&column_number(self.columns.len()));
        }
    }

    pub fn place(&mut self, commit: &Commit) -> LaneRow {
        let before = self.columns.len();
        let mut joining = self.waiting.remove(&commit.hash).unwrap_or_default();
        joining.sort_unstable();

        let (node_column, color) = match joining.first() {
            Some(&column) => (column, self.lane_color(column)),
            None => (self.leftmost_free(), self.take_color()),
        };

        let mut junctions = Vec::new();
        for &column in joining.iter().skip(1) {
            junctions.push(Junction {
                column,
                color: self.lane_color(column),
                opens: false,
            });
            self.release(column);
        }

        let keeps_lane = !commit.parents.is_empty();
        match commit.parents.first() {
            Some(parent) => self.occupy(node_column, color, parent),
            None => self.release(node_column),
        }

        for parent in commit.parents.iter().skip(1) {
            let column = self.leftmost_free();
            let color = self.take_color();
            self.occupy(column, color, parent);
            junctions.push(Junction { column, color, opens: true });
        }

        let width = column_number(before.max(self.columns.len()));
        self.trim();

        LaneRow {
            node_column,
            color,
            keeps_lane,
            junctions,
            width,
        }
    }
}

fn set_column(lanes: &mut Columns, column: u32, color: Option<u32>) {
    let index = column_index(column);
    if index >= lanes.len() {
        lanes.resize(index + 1, None);
    }
    lanes[index] = color;
}

pub fn advance(lanes: Columns, row: &LaneRow) -> Columns {
    let mut lanes = lanes;
    set_column(&mut lanes, row.node_column, row.keeps_lane.then_some(row.color));
    for junction in &row.junctions {
        set_column(&mut lanes, junction.column, junction.opens.then_some(junction.color));
    }
    while lanes.last().is_some_and(Option::is_none) {
        lanes.pop();
    }

    lanes
}

fn end_symbol(row: &LaneRow, column: u32) -> Option<(char, u32)> {
    let ends = || row.junctions.iter().filter(move |junction| junction.column == column);
    let closes = ends().any(|junction| !junction.opens);
    let opening = ends().find(|junction| junction.opens);
    let toward_left = column < row.node_column;

    if let Some(junction) = opening {
        let symbol = match (closes, toward_left) {
            (true, true) => '├',
            (true, false) => '┤',
            (false, true) => '╭',
            (false, false) => '╮',
        };

        return Some((symbol, junction.color));
    }

    let closing = ends().next()?;
    let symbol = if toward_left { '╰' } else { '╯' };

    Some((symbol, closing.color))
}

fn crossing_color(row: &LaneRow, between: impl Fn(u32, u32) -> bool) -> Option<u32> {
    row.junctions
        .iter()
        .filter(|junction| between(row.node_column.min(junction.column), row.node_column.max(junction.column)))
        .max_by_key(|junction| std::cmp::Reverse(junction.column.abs_diff(row.node_column)))
        .map(|junction| junction.color)
}

fn cell_at(before: &[Option<u32>], row: &LaneRow, head: bool, index: usize) -> Cell {
    let column = column_number(index / 2);
    if index % 2 == 1 {
        let color = crossing_color(row, |from, to| from <= column && column < to);

        return color.map_or(BLANK_CELL, |color| Cell { symbol: '─', color });
    }

    if column == row.node_column {
        let symbol = if head { HEAD_NODE } else { NODE };

        return Cell { symbol, color: row.color };
    }

    if let Some((symbol, color)) = end_symbol(row, column) {
        return Cell { symbol, color };
    }

    if let Some(Some(color)) = before.get(column_index(column)) {
        return Cell {
            symbol: '│', color: *color
        };
    }

    let color = crossing_color(row, |from, to| from < column && column < to);

    color.map_or(BLANK_CELL, |color| Cell { symbol: '─', color })
}

pub fn cells(before: &[Option<u32>], row: &LaneRow, head: bool, offset: usize, window: usize) -> Vec<Cell> {
    let start = offset * 2;
    let row_cells = (2 * column_index(row.width)).saturating_sub(1);

    (start..start + window)
        .map(|index| {
            if index >= row_cells {
                return BLANK_CELL;
            }

            cell_at(before, row, head, index)
        })
        .collect()
}
