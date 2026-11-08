const FIELD_SEPARATOR: char = '\x1f';

const SHORT_HASH_LENGTH: usize = 7;

pub struct Commit {
    pub hash: String,
    pub parents: Vec<String>,
    pub is_head: bool,
    pub date: String,
    pub author: String,
    pub subject: String,
}

pub fn parse(log: &str) -> Vec<Commit> {
    log.lines().filter_map(parse_commit).collect()
}

fn short_hash(hash: &str) -> String {
    hash.chars().take(SHORT_HASH_LENGTH).collect()
}

fn points_at_head(refs: &str) -> bool {
    refs.split(", ")
        .any(|name| name == "HEAD" || name.starts_with("HEAD -> "))
}

fn parse_commit(line: &str) -> Option<Commit> {
    let mut fields = line.split(FIELD_SEPARATOR);
    let hash = fields.next()?;
    let parents = fields.next()?;
    let refs = fields.next()?;
    let date = fields.next()?;
    let author = fields.next()?;
    let subject = fields.next()?;

    Some(Commit {
        hash: short_hash(hash),
        parents: parents.split_whitespace().map(short_hash).collect(),
        is_head: points_at_head(refs),
        date: date.to_string(),
        author: author.to_string(),
        subject: subject.to_string(),
    })
}
