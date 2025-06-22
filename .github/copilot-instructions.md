<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

This is a Rust CLI project named 'gut'. It is a git wrapper with the following features:
- Auto-infer git subcommands from short abbreviations or typos.
- 'gut commit' takes the last argument as the commit message.
- Auto-format commit messages: user can write 'feat:xxx' and gut converts it to '<corresponding emoji> feat: xxx'.
- Can create a repo via a 'template' (clone a repo, delete .git, re-init).
- 'gut branch' auto-switches to the created branch.
- 'gut rlog' = reversed log.
- Other commands not changed by gut are passed directly to git.
