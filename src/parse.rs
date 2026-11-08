pub const FIELD_SEPARATOR: char = '\x1f';

pub struct Commit {
    pub subject: String,
}

pub fn parse(log: &str) -> Vec<Commit> {
    log.lines().filter_map(parse_commit).collect()
}

fn parse_commit(line: &str) -> Option<Commit> {
    let subject = line.split(FIELD_SEPARATOR).nth(5)?;

    Some(Commit {
        subject: subject.to_string(),
    })
}
