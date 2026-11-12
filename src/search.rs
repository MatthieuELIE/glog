use std::ops::Range;

use crate::history::History;

#[derive(Clone, Debug, PartialEq)]
pub struct Search {
    pub query: String,
    pub matches: Vec<usize>,
    pub scanned: usize,
}

fn ignores_case(query: &str) -> bool {
    !query.chars().any(char::is_uppercase)
}

fn contains(field: &str, query: &str) -> bool {
    if ignores_case(query) {
        return field.to_lowercase().contains(query);
    }

    field.contains(query)
}

fn starts_with(query: &str, field: &str) -> bool {
    if ignores_case(query) {
        return query.starts_with(&field.to_lowercase());
    }

    query.starts_with(field)
}

pub fn find(field: &str, query: &str) -> Option<Range<usize>> {
    if query.is_empty() {
        return None;
    }

    if !ignores_case(query) {
        return field.find(query).map(|start| start..start + query.len());
    }

    let mut lowered = String::new();
    let mut origins = Vec::new();
    for (start, character) in field.char_indices() {
        let lower: String = character.to_lowercase().collect();
        origins.extend(lower.bytes().map(|_| start..start + character.len_utf8()));
        lowered.push_str(&lower);
    }
    let start = lowered.find(query)?;

    Some(origins[start].start..origins[start + query.len() - 1].end)
}

pub fn find_in_hash(hash: &str, query: &str) -> Option<Range<usize>> {
    if starts_with(query, hash) {
        return Some(0..hash.len());
    }

    find(hash, query)
}

fn is_match(history: &History, row: usize, query: &str) -> bool {
    let hash = history.short_hash(row);

    contains(history.subject(row), query)
        || contains(history.author(row), query)
        || contains(hash, query)
        || starts_with(query, hash)
        || history.refs(row).iter().any(|reference| contains(&reference.name, query))
}

pub fn scan(history: &History, query: &str) -> Search {
    Search {
        query: query.to_string(),
        matches: (0..history.len()).filter(|&row| is_match(history, row, query)).collect(),
        scanned: history.len(),
    }
}

pub fn complete(search: Search, history: &History) -> Search {
    let mut matches = search.matches;
    matches.extend((search.scanned..history.len()).filter(|&row| is_match(history, row, &search.query)));

    Search {
        matches,
        scanned: history.len(),
        ..search
    }
}

pub fn next_after(matches: &[usize], selected: usize) -> Option<usize> {
    matches.iter().copied().find(|&row| row > selected).or(matches.first().copied())
}

pub fn previous_before(matches: &[usize], selected: usize) -> Option<usize> {
    matches.iter().copied().rfind(|&row| row < selected).or(matches.last().copied())
}
