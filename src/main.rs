mod config;
mod utils;
mod commit;
mod log;

use config::{load_config, check_and_generate_hooks};
use utils::{pass_to_git};
use commit::gut_commit;
use log::{gut_log, gut_rlog, gut_tlog};
use std::env;

fn gut_branch(args: &[String]) {
    if args.is_empty() {
        eprintln!("gut branch <branch-name>");
        std::process::exit(1);
    }
    let branch = &args[0];
    pass_to_git(&["checkout".to_string(), "-b".to_string(), branch.clone()]);
}

fn gut_template(args: &[String]) {
    use std::fs;
    if args.is_empty() {
        eprintln!("gut template <template-repo-url> [dest]");
        std::process::exit(1);
    }
    let url = &args[0];
    let dest = args.get(1).map(|s| s.as_str()).unwrap_or(".");
    let status = std::process::Command::new("git").args(["clone", url, dest]).status().expect("failed to clone");
    if !status.success() { std::process::exit(1); }
    let git_dir = format!("{}/.git", dest);
    if fs::remove_dir_all(&git_dir).is_err() {
        eprintln!("Failed to remove .git directory");
        std::process::exit(1);
    }
    let status = std::process::Command::new("git").current_dir(dest).args(["init"]).status().expect("failed to re-init");
    if !status.success() { std::process::exit(1); }
    println!("Template repo initialized at {}", dest);
}

fn main() {
    let config = load_config();
    check_and_generate_hooks(&config);
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("\nUsage: gut <git-subcommand> [args...]\n");
        eprintln!("Gut is a CLI tool that wraps git, providing smart subcommand inference, commit message formatting, config-driven hooks, and convenient shortcuts.\n");
        eprintln!("Just repalce all your 'git' with 'gut' ! :)\n ");
        eprintln!("Main features:");
        eprintln!("  - Auto-infer git subcommands from short abbreviations or typos\n");
        eprintln!("  - 'gut commit' takes the last argument as the commit message");
        eprintln!("  - Auto-format commit messages: write 'feat:xxx' or 'feat(scope):xxx' and gut converts it to 'feat: <emoji> xxx' or 'feat(scope): <emoji> xxx'");
        eprintln!("  - Supports many conventional commit types (feat, fix, docs, refactor, test, chore, build, style, ci, perf, revert) and custom types via config");
        eprintln!("  - Supports commit message formatting modes: upper_case/lower_case (configurable)");
        eprintln!("  - Supports custom emoji mapping for commit types via gut.config.json\n");
        eprintln!("  - Create a repo via a 'template' (clone a repo, delete .git, re-init)\n");
        eprintln!("  - 'gut branch' auto-switches to the created branch\n");
        eprintln!("  - 'gut log' outputs a dense, configurable log");
        eprintln!("  - 'gut rlog' outputs a reversed log, following log config");
        eprintln!("  - 'gut tlog' outputs a tree log: latest N commits from all branches, current branch ranked first, dense or detailed (configurable)\n");
        eprintln!("  - Configurable global git hooks via gut.config.json (auto-generated in .git/hooks)\n");
        eprintln!("  - Other commands not changed by gut are passed directly to git with only typo/abbr inference\n");
        eprintln!("From more usage refer to https://github.com/elliot-zzh/gut\n");
        std::process::exit(1);
    }
    let sub = &args[0];
    // Find the subcommand with the shortest Levenshtein distance (including gut-only commands)
    const ALL_COMMANDS: &[&str] = &[
        "template", "rlog", "tlog",
        "init", "clone", "add", "commit", "restore", "rm", "mv", "status", "log", "diff", "show", "branch", "checkout", "merge", "rebase", "fast-forward", "tag", "stash", "pull", "fetch", "push", "remote", "submodule", "reset", "revert", "clean", "gc", "fsck", "archive", "blame", "bisect", "cherry-pick", "config", "help"
    ];
    let mut min_dist = usize::MAX;
    let mut best_cmd = None;
    for &cmd in ALL_COMMANDS {
        let dist = utils::levenshtein(sub, cmd);
        if dist < min_dist {
            min_dist = dist;
            best_cmd = Some(cmd);
        }
    }
    if let Some(cmd) = best_cmd {
        if min_dist > 3 {
            // Fallback: pass to git
            pass_to_git(&args);
            return;
        }
        if min_dist >= 1 {
            println!("[gut] subcommand smart infer: {} \x1b[32m=>\x1b[0m {}", sub, cmd);
        }
        match cmd {
            "template" => gut_template(&args[1..]),
            "rlog" => gut_rlog(&args[1..], &config),
            "tlog" => gut_tlog(&args[1..], &config),
            "commit" => gut_commit(&args[1..], &config),
            "branch" => gut_branch(&args[1..]),
            "log" => gut_log(&args[1..], &config),
            _ => {
                let mut git_args = vec![cmd.to_string()];
                git_args.extend(args[1..].iter().cloned());
                pass_to_git(&git_args);
            }
        }
        return;
    }
    // Fallback: pass to git
    pass_to_git(&args);
}
