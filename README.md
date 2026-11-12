# glog

A read-only, coloured git log graph for the terminal: every branch, remote and tag of a repository in one scrollable view.

![The glog view on a repository with a feature branch, merged pull requests and remote branches: coloured branch pills, grey remote pills and yellow tag pills, a 15-cell author column, and the selected commit in the middle of the history on a soft background with a bold subject](docs/images/graph.png)

## Install

```sh
cargo install --locked --git https://github.com/MatthieuELIE/glog --tag v0.1.2
```

This puts `glog` in `~/.cargo/bin`. From a clone, run `cargo install --locked --path .` instead.
It is developed on macOS and tested on Linux; it has not been tried on Windows.

## Usage

Run `glog` in a git repository. It reads the current folder and changes nothing. It opens while the history is still loading, with a status line on the last row until it is done.

| Key      | Action                        |
| -------- | ----------------------------- |
| `j`      | Move the selection down       |
| `k`      | Move the selection up         |
| `h`      | Scroll the graph left         |
| `l`      | Scroll the graph right        |
| `g`      | Select the first commit       |
| `G`      | Select the last loaded commit |
| `Ctrl+d` | Move down half a page         |
| `Ctrl+u` | Move up half a page           |
| `q`      | Quit                          |
| `Ctrl+C` | Quit                          |

`glog --help` prints the same keys, and `glog --version` the installed version.

## Reading the graph

- `●` is a commit and `○` is HEAD.
- Each lane, a line of history that branches off and merges back, has its own colour.
- The graph takes at most a quarter of the terminal width. When lanes are hidden, the two cells on that edge of the graph fade to dark grey, on the left or the right: `h` and `l` scroll the graph one lane at a time.
- Badges are pills, a name on a coloured background, and name the refs on a commit: local branches in their lane's colour, remote branches in grey, and tags in yellow with a `◆`. The branch HEAD is on is the first pill, in bold; a detached HEAD shows a bold `HEAD` pill instead.
- Colour is kept for the graph and the badges: the hash and the date are the dimmest text, the author is a step lighter and the subject is the lightest. The selected row has a soft background and a bold subject.

## Requirements

- `git` in the `PATH`
- a terminal with truecolor support
- Rust 1.89 or later to build it

## License

MIT, see [LICENSE](LICENSE).
