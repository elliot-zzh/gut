mod config;
mod utils;
mod commit;
mod log;

use config::{load_config, check_and_generate_hooks};
use utils::{infer_subcommand, pass_to_git};
use commit::gut_commit;
use log::{gut_log, gut_rlog, gut_tlog};
use std::env;

const GIT_COMMANDS: &[&str] = &[
    "init", "clone", "add", "commit", "restore", "rm", "mv", "status", "log", "diff", "show", "branch", "checkout", "merge", "rebase", "fast-forward", "tag", "stash", "pull", "fetch", "push", "remote", "submodule", "reset", "revert", "clean", "gc", "fsck", "archive", "blame", "bisect", "cherry-pick", "config", "help"
];

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
        eprintln!("Usage: gut <git-subcommand> [args...]");
        std::process::exit(1);
    }
    let sub = &args[0];
    // Special handling for gut-only commands
    if infer_subcommand(sub, "template") {
        gut_template(&args[1..]);
        return;
    }
    // Special handling for log variants
    if infer_subcommand(sub, "rlog") {
        gut_rlog(&args[1..], &config);
        return;
    }
    if infer_subcommand(sub, "tlog") {
        gut_tlog(&args[1..], &config);
        return;
    }
    // Typo/abbr inference for all standard git commands
    for &cmd in GIT_COMMANDS {
        if infer_subcommand(sub, cmd) {
            match cmd {
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
    }
    // Fallback: pass to git
    pass_to_git(&args);
}
