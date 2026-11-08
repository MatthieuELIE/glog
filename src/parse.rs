pub const FIELD_SEPARATOR: char = '\x1f';

const SHORT_HASH_LENGTH: usize = 7;

pub struct Commit {
    pub hash: String,
    pub date: String,
    pub author: String,
    pub subject: String,
}

pub fn parse(log: &str) -> Vec<Commit> {
    log.lines().filter_map(parse_commit).collect()
}

fn parse_commit(line: &str) -> Option<Commit> {
    let mut fields = line.split(FIELD_SEPARATOR);
    let hash = fields.next()?;
    let date = fields.nth(2)?;
    let author = fields.next()?;
    let subject = fields.next()?;

    Some(Commit {
        hash: hash.chars().take(SHORT_HASH_LENGTH).collect(),
        date: date.to_string(),
        author: author.to_string(),
        subject: subject.to_string(),
    })
}
