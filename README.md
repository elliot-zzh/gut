# gut

A Rust CLI tool that wraps git, providing smart subcommand inference, commit message formatting, and convenient shortcuts.

## Features
- Auto-infer git subcommands from short abbreviations or typos
- `gut commit` takes the last argument as the commit message
- Auto-format commit messages: write `feat:xxx` and gut converts it to `<corresponding emoji> feat: xxx`
- Create a repo via a 'template' (clone a repo, delete .git, re-init)
- `gut branch` auto-switches to the created branch
- `gut rlog` = reversed log
- Other commands not changed by gut are passed directly to git

## Getting Started
1. Build: `cargo build --release`
2. Run: `cargo run -- <args>`

## License
MIT
