const FIELD_SEPARATOR: char = '\x1f';

const SHORT_HASH_LENGTH: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RefKind {
    Head,
    Local,
    Remote,
    Tag,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ref {
    pub kind: RefKind,
    pub name: String,
}

pub struct Commit {
    pub hash: String,
    pub parents: Vec<String>,
    pub refs: Vec<Ref>,
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

fn named(kind: RefKind, name: &str) -> Ref {
    Ref { kind, name: name.to_string() }
}

fn local(name: &str) -> Ref {
    named(RefKind::Local, name.strip_prefix("refs/heads/").unwrap_or(name))
}

fn parse_ref(decoration: &str) -> Vec<Ref> {
    if decoration == "HEAD" {
        return vec![named(RefKind::Head, "HEAD")];
    }
    if let Some(branch) = decoration.strip_prefix("HEAD -> ") {
        return vec![named(RefKind::Head, "HEAD"), local(branch)];
    }
    if let Some(tag) = decoration.strip_prefix("tag: ") {
        return vec![named(RefKind::Tag, tag.strip_prefix("refs/tags/").unwrap_or(tag))];
    }
    if let Some(remote) = decoration.strip_prefix("refs/remotes/") {
        if remote.ends_with("/HEAD") {
            return Vec::new();
        }
        return vec![named(RefKind::Remote, remote)];
    }
    if decoration.starts_with("refs/heads/") {
        return vec![local(decoration)];
    }

    Vec::new()
}

fn parse_refs(refs: &str) -> Vec<Ref> {
    let mut parsed: Vec<Ref> = refs.split(", ").flat_map(parse_ref).collect();
    parsed.sort_by_key(|reference| reference.kind);

    parsed
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
        refs: parse_refs(refs),
        date: date.to_string(),
        author: author.to_string(),
        subject: subject.to_string(),
    })
}
