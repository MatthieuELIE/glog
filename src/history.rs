use std::collections::HashMap;

use crate::{
    lanes::{Junction, LaneRow, Placer, advance},
    parse::{Commit, Ref},
};

const CHECKPOINT_EVERY: usize = 1_024;
pub const SHORT_HASH_WIDTH: usize = 7;

fn offset(position: usize) -> u32 {
    u32::try_from(position).unwrap_or(u32::MAX)
}

#[derive(Default)]
struct Texts {
    text: String,
    ends: Vec<u32>,
}

impl Texts {
    fn push(&mut self, piece: &str) {
        self.text.push_str(piece);
        self.ends.push(offset(self.text.len()));
    }

    fn get(&self, row: usize) -> &str {
        let start = row.checked_sub(1).map_or(0, |previous| self.ends[previous] as usize);

        &self.text[start..self.ends[row] as usize]
    }
}

#[derive(Default)]
struct Authors {
    names: Vec<String>,
    ids: HashMap<String, u32>,
}

impl Authors {
    fn intern(&mut self, name: String) -> u32 {
        if let Some(&id) = self.ids.get(&name) {
            return id;
        }
        let id = offset(self.names.len());
        self.names.push(name.clone());
        self.ids.insert(name, id);

        id
    }
}

#[derive(Default)]
pub struct History {
    placer: Placer,
    hashes: Texts,
    subjects: Texts,
    times: Vec<i64>,
    authors: Authors,
    author_of: Vec<u32>,
    refs: HashMap<u32, Vec<Ref>>,
    node_columns: Vec<u32>,
    colors: Vec<u32>,
    keeps_lane: Vec<bool>,
    widths: Vec<u32>,
    junction_ends: Vec<u32>,
    junctions: Vec<Junction>,
    checkpoints: Vec<Vec<Option<u32>>>,
    max_lanes: usize,
}

impl History {
    pub fn push(&mut self, commit: Commit) {
        if self.len().is_multiple_of(CHECKPOINT_EVERY) {
            self.checkpoints.push(self.placer.lanes());
        }
        let row = self.placer.place(&commit);

        self.hashes.push(&commit.hash.chars().take(SHORT_HASH_WIDTH).collect::<String>());
        self.subjects.push(&commit.subject);
        self.times.push(commit.time);
        let author = self.authors.intern(commit.author);
        self.author_of.push(author);
        if !commit.refs.is_empty() {
            self.refs.insert(offset(self.times.len() - 1), commit.refs);
        }

        self.node_columns.push(row.node_column);
        self.colors.push(row.color);
        self.keeps_lane.push(row.keeps_lane);
        self.widths.push(row.width);
        self.junctions.extend(row.junctions);
        self.junction_ends.push(offset(self.junctions.len()));
        self.max_lanes = self.max_lanes.max(row.width as usize);
    }

    pub fn len(&self) -> usize {
        self.times.len()
    }

    pub fn is_empty(&self) -> bool {
        self.times.is_empty()
    }

    pub fn max_lanes(&self) -> usize {
        self.max_lanes
    }

    pub fn short_hash(&self, row: usize) -> &str {
        self.hashes.get(row)
    }

    pub fn time(&self, row: usize) -> i64 {
        self.times[row]
    }

    pub fn author(&self, row: usize) -> &str {
        &self.authors.names[self.author_of[row] as usize]
    }

    pub fn subject(&self, row: usize) -> &str {
        self.subjects.get(row)
    }

    pub fn refs(&self, row: usize) -> &[Ref] {
        self.refs.get(&offset(row)).map_or(&[], Vec::as_slice)
    }

    pub fn lane_row(&self, row: usize) -> LaneRow {
        let start = row.checked_sub(1).map_or(0, |previous| self.junction_ends[previous] as usize);

        LaneRow {
            node_column: self.node_columns[row],
            color: self.colors[row],
            keeps_lane: self.keeps_lane[row],
            junctions: self.junctions[start..self.junction_ends[row] as usize].to_vec(),
            width: self.widths[row],
        }
    }

    pub fn lanes_before(&self, row: usize) -> Vec<Option<u32>> {
        let checkpoint = row / CHECKPOINT_EVERY;
        let lanes = self.checkpoints.get(checkpoint).cloned().unwrap_or_default();

        (checkpoint * CHECKPOINT_EVERY..row).fold(lanes, |lanes, previous| advance(lanes, &self.lane_row(previous)))
    }
}
