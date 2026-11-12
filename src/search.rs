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

pub fn next_after(matches: &[usize], selected: usize) -> Option<usize> {
    matches.iter().copied().find(|&row| row > selected).or(matches.first().copied())
}

pub fn previous_before(matches: &[usize], selected: usize) -> Option<usize> {
    matches.iter().copied().rfind(|&row| row < selected).or(matches.last().copied())
}
