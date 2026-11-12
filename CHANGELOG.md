# Changelog

Notable changes to `glog`, most recent first.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed

- The status line on the last row is permanent, from launch to exit, and the list keeps the same height: it shows the repository name, the branch of HEAD (the short hash when detached) and the position of the selected commit on the commits loaded, like `42/1,204`, with a `…` while git reads, and `loading… N commits` is gone. A git failure still shows in red in its place.
- The graph follows the selected commit: when the selection changes, it shifts just enough to show the commit's node, and `h` and `l` still scroll it freely until the next change of selection.
- The graph takes at most a quarter of the terminal width instead of a third, which leaves more room to the subject.
- The author column has a width of fifteen cells instead of twenty.
- When lanes are hidden, the two cells on that edge of the graph fade to dark grey instead of a dimmed `…`.
- Badges are pills, a name on a coloured background instead of between brackets: local branches in their lane's colour, remote branches in grey, tags in yellow.
- On a branch, HEAD no longer has its own `[HEAD]` badge: the pill of its branch is first and in bold. A detached HEAD shows a bold `HEAD` pill.
- The hash and the date are the dimmest text, the author is a step lighter and the subject is the lightest: colour is kept for the graph and the badges.
- The selected row has a softer background and a bold subject.

## [0.1.2] - 2026-11-11

### Added

- `g` selects the first commit and `G` the last one loaded, `Ctrl+d` and `Ctrl+u` move half a page down and up.
- The view opens while the history is still loading, with a status line on the last row that counts the commits read, or shows git's error if it fails partway.
- `glog: reading history…` on stderr when the first commits take more than half a second to arrive.

### Changed

- The date is a compact relative time on four cells (`now`, `42m`, `5h`, `12d`, `3mo`, `2y`) instead of git's `3 days ago`.
- The author column has a fixed width of twenty cells, and a longer name ends with `…`.
- A large history opens much faster and uses far less memory: on 1.1 million commits, the first frame comes in under 2 s instead of over 10 s, a key answers in about 2 ms instead of seconds, and memory stays under 200 MB instead of over 1 GB.

### Fixed

- Two commits sharing their first seven hash characters are no longer taken for the same commit, which could link a commit to the wrong parent.

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

[0.1.2]: https://github.com/MatthieuELIE/glog/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/MatthieuELIE/glog/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/MatthieuELIE/glog/releases/tag/v0.1.0
