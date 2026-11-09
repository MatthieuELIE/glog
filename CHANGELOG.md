# Changelog

Notable changes to `glog`, most recent first.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.0] - 2026-11-09

### Added

- A graph of every ref in the repository, branches, remotes and tags.
- One colour per lane, so a branch can be followed from where it starts to where it merges.
- Badges on each commit for local branches, dimmed remote branches and tags (`◆`, in yellow).
- Aligned columns for the graph, the hash, the date, the author and the message.
- `j` and `k` move the selection, `q` and `Ctrl+C` quit.
- A one-line error on stderr outside a repository (git's own message), in a repository with no commit, and for an unknown argument.
- `glog --version` prints the version, and `glog --help` the usage and the keys, from any folder.

[0.1.0]: https://github.com/MatthieuELIE/glog/releases/tag/v0.1.0
