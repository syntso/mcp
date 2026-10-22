# Contributing

## Branches

- `main` holds released code only.
- `dev` is where work comes together. Branch from it, and open pull requests into it.

Pull requests into `dev` are squash-merged, so the pull request title becomes the commit.

## Commits

[Conventional Commits](https://www.conventionalcommits.org), one line, no body: `feat: offer extension actions as tools`. The whole line is at most 72 characters. `git config core.hooksPath .githooks` turns on the local check.

## Code

The bridge follows the rules of the [core](https://github.com/syntso/synt/blob/dev/CONTRIBUTING.md): one main type per file, the folder's main type in the file beside the folder, errors with `thiserror`, no `unwrap()` outside tests, no comments.

- `invocation.rs` reads the command line.
- `link.rs` holds the connection to the core.
- `catalog.rs` turns the core's actions into MCP tools.
- `call.rs` runs a tool call as an action and turns the outcome into a tool result.
- `bridge.rs` is the MCP server that ties them together.

## Before you push

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs the same on macOS, Windows and Linux, plus `cargo deny check`.
