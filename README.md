# glog

A read-only, coloured git log graph for the terminal: every branch, remote and tag of a repository in one scrollable view.

![The glog view on a repository with a feature branch, merged pull requests and remote branches, with the selection on a commit in the middle of the history](docs/images/graph.png)

## Install

```sh
cargo install --locked --git https://github.com/MatthieuELIE/glog --tag v0.1.0
```

This puts `glog` in `~/.cargo/bin`. From a clone, run `cargo install --locked --path .` instead.
It is developed on macOS and tested on Linux; it has not been tried on Windows.
