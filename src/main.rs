mod config;
mod utils;
mod commit;
mod log;

use config::{load_config, check_and_generate_hooks};
use utils::{infer_subcommand, pass_to_git};
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
        eprintln!("Usage: gut <git-subcommand> [args...]");
        std::process::exit(1);
    }
    let sub = &args[0];
    match sub.as_str() {
        s if infer_subcommand(s, "commit") => gut_commit(&args[1..], &config),
        s if infer_subcommand(s, "branch") => gut_branch(&args[1..]),
        s if infer_subcommand(s, "rlog") => gut_rlog(&args[1..], &config),
        s if infer_subcommand(s, "template") => gut_template(&args[1..]),
        s if infer_subcommand(s, "log") => gut_log(&args[1..], &config),
        s if infer_subcommand(s, "tlog") => gut_tlog(&args[1..], &config),
        _ => pass_to_git(&args),
    }
}
