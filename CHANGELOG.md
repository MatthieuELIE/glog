# Changelog

Notable changes to `glog`, most recent first.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.1] - 2026-11-09

### Added

- `h` and `l` scroll the graph one lane left or right.
- A dimmed `…` on the left or right edge of the graph when lanes are hidden on that side.

### Changed

- The graph takes at most a third of the terminal width.
- The selected row stands out more, on a lighter background (Surface1 instead of Surface0).

## [0.1.0] - 2026-11-09

### Added

- A graph of every ref in the repository, branches, remotes and tags.
- One colour per lane, so a branch can be followed from where it starts to where it merges.
- Badges on each commit for local branches, dimmed remote branches and tags (`◆`, in yellow).
- Aligned columns for the graph, the hash, the date, the author and the message.
- `j` and `k` move the selection, `q` and `Ctrl+C` quit.
- A one-line error on stderr outside a repository (git's own message), in a repository with no commit, and for an unknown argument.
- `glog --version` prints the version, and `glog --help` the usage and the keys, from any folder.

[0.1.1]: https://github.com/MatthieuELIE/glog/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/MatthieuELIE/glog/releases/tag/v0.1.0
