use std::env;
use std::process::{Command, exit};
use std::fs;
use std::path::Path;
use std::io::Write;

fn main() {
    let config = load_config();
    check_and_generate_hooks(&config);
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: gut <git-subcommand> [args...]");
        exit(1);
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

fn load_config() -> serde_json::Value {
    let config_path = Path::new("gut.config.json");
    if !config_path.exists() { return serde_json::json!({}); }
    let config = std::fs::read_to_string(config_path).unwrap_or_default();
    serde_json::from_str(&config).unwrap_or_default()
}

fn check_and_generate_hooks(config: &serde_json::Value) {
    let hooks = config.get("hooks").and_then(|v| v.as_array());
    let git_hooks_dir = Path::new(".git/hooks");
    if hooks.is_none() || !git_hooks_dir.exists() { return; }
    for hook in hooks.unwrap() {
        let name = hook.get("name").and_then(|v| v.as_str());
        let condition = hook.get("condition").and_then(|v| v.as_str()).unwrap_or("");
        let commands = hook.get("commands").and_then(|v| v.as_array());
        if let (Some(name), Some(commands)) = (name, commands) {
            let mut script = String::from("#!/bin/sh\nset -e\n");
            if !condition.is_empty() {
                script.push_str(&format!("if ! ({}); then exit 0; fi\n", condition));
            }
            for cmd in commands {
                if let Some(cmd_str) = cmd.as_str() {
                    script.push_str(cmd_str);
                    script.push('\n');
                }
            }
            let hook_path = git_hooks_dir.join(name);
            if !hook_path.exists() || std::fs::read_to_string(&hook_path).ok().as_deref() != Some(&script) {
                if let Ok(mut f) = std::fs::File::create(&hook_path) {
                    let _ = f.write_all(script.as_bytes());
                    let _ = std::fs::set_permissions(&hook_path, std::os::unix::fs::PermissionsExt::from_mode(0o755));
                }
            }
        }
    }
}

fn infer_subcommand(input: &str, target: &str) -> bool {
    // Accepts abbreviation or typo (Levenshtein distance <=1 or prefix)
    if target.starts_with(input) { return true; }
    levenshtein(input, target) <= 1
}

fn levenshtein(a: &str, b: &str) -> usize {
    let mut costs = vec![0; b.len() + 1];
    for j in 0..=b.len() { costs[j] = j; }
    for (i, ca) in a.chars().enumerate() {
        let mut last = i;
        costs[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let old = costs[j + 1];
            costs[j + 1] = std::cmp::min(
                std::cmp::min(costs[j] + 1, costs[j + 1] + 1),
                last + if ca == cb { 0 } else { 1 }
            );
            last = old;
        }
    }
    costs[b.len()]
}

fn gut_commit(args: &[String], config: &serde_json::Value) {
    if args.is_empty() {
        eprintln!("gut commit <msg>");
        exit(1);
    }
    let msg = &args[args.len() - 1];
    let formatted = format_commit_message(msg, config);
    let mut git_args = vec!["commit".to_string(), "-m".to_string(), formatted];
    if args.len() > 1 {
        git_args.splice(1..1, args[..args.len()-1].to_vec());
    }
    pass_to_git(&git_args);
}

fn format_commit_message(msg: &str, config: &serde_json::Value) -> String {
    let emoji = if msg.starts_with("feat:") { "✨" }
        else if msg.starts_with("fix:") { "🐛" }
        else if msg.starts_with("docs:") { "📝" }
        else if msg.starts_with("refactor:") { "♻️" }
        else if msg.starts_with("test:") { "✅" }
        else if msg.starts_with("chore:") { "🔧" }
        else { "" };
    let mut formatted = if let Some((typ, rest)) = msg.split_once(":") {
        format!("{} {}: {}", emoji, typ, rest.trim())
    } else {
        msg.to_string()
    };
    if let Some(commit_cfg) = config.get("commit") {
        if let Some(mode) = commit_cfg.get("format_mode").and_then(|v| v.as_str()) {
            match mode {
                "upper_case" => {
                    if let Some(first) = formatted.get_mut(0..1) {
                        first.make_ascii_uppercase();
                    }
                },
                "lower_case" => {
                    if let Some(first) = formatted.get_mut(0..1) {
                        first.make_ascii_lowercase();
                    }
                },
                _ => {}
            }
        }
    }
    formatted
}

fn gut_branch(args: &[String]) {
    if args.is_empty() {
        eprintln!("gut branch <branch-name>");
        exit(1);
    }
    let branch = &args[0];
    pass_to_git(&["checkout".to_string(), "-b".to_string(), branch.clone()]);
}

fn gut_rlog(args: &[String], config: &serde_json::Value) {
    // rlog always follows log config
    let log_cfg = config.get("log").unwrap_or(&serde_json::json!({}));
    let count = log_cfg.get("count").and_then(|v| v.as_u64()).unwrap_or(10);
    let info = log_cfg.get("info").and_then(|v| v.as_str()).unwrap_or("less");
    let pretty = if info == "more" {
        "%h %an %ad %s"
    } else {
        "%h %s"
    };
    let mut git_args = vec!["log".to_string(), "-n".to_string(), count.to_string(), "--reverse".to_string(), format!("--pretty=format:{}", pretty)];
    git_args.extend(args.iter().cloned());
    pass_to_git(&git_args);
}

fn gut_template(args: &[String]) {
    if args.is_empty() {
        eprintln!("gut template <template-repo-url> [dest]");
        exit(1);
    }
    let url = &args[0];
    let dest = args.get(1).map(|s| s.as_str()).unwrap_or(".");
    let status = Command::new("git").args(["clone", url, dest]).status().expect("failed to clone");
    if !status.success() { exit(1); }
    let git_dir = format!("{}/.git", dest);
    if fs::remove_dir_all(&git_dir).is_err() {
        eprintln!("Failed to remove .git directory");
        exit(1);
    }
    let status = Command::new("git").current_dir(dest).args(["init"]).status().expect("failed to re-init");
    if !status.success() { exit(1); }
    println!("Template repo initialized at {}", dest);
}

fn gut_log(_args: &[String], config: &serde_json::Value) {
    let log_cfg = config.get("log").unwrap_or(&serde_json::json!({}));
    let count = log_cfg.get("count").and_then(|v| v.as_u64()).unwrap_or(10);
    let info = log_cfg.get("info").and_then(|v| v.as_str()).unwrap_or("less");
    let pretty = if info == "more" {
        "%h %an %ad %s"
    } else {
        "%h %s"
    };
    let output = Command::new("git")
        .args(["log", "-n", &count.to_string(), "--pretty=format:".to_owned() + pretty])
        .output()
        .expect("failed to run git log");
    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("git log failed");
        exit(1);
    }
}

fn gut_tlog(_args: &[String], config: &serde_json::Value) {
    let tlog_cfg = config.get("tlog").unwrap_or(&serde_json::json!({}));
    let count = tlog_cfg.get("count").and_then(|v| v.as_u64()).unwrap_or(20);
    let info = tlog_cfg.get("info").and_then(|v| v.as_str()).unwrap_or("less");
    let pretty = if info == "more" {
        "%h %an %ad %s"
    } else {
        "%h %s"
    };
    let current_branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).trim().to_string()) } else { None });
    let mut branches = vec![];
    let branch_output = Command::new("git")
        .args(["for-each-ref", "--format=%(refname:short)", "refs/heads/"])
        .output()
        .expect("failed to list branches");
    if branch_output.status.success() {
        for line in String::from_utf8_lossy(&branch_output.stdout).lines() {
            branches.push(line.to_string());
        }
    }
    if let Some(ref cur) = current_branch {
        branches.retain(|b| b != cur);
        branches.insert(0, cur.clone());
    }
    let mut seen = std::collections::HashSet::new();
    let mut all_commits = vec![];
    for branch in &branches {
        let output = Command::new("git")
            .args(["log", branch, "-n", &count.to_string(), "--pretty=format:".to_owned() + pretty])
            .output()
            .expect("failed to run git log");
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let mut parts = line.splitn(2, ' ');
                if let (Some(hash), Some(msg)) = (parts.next(), parts.next()) {
                    if seen.insert(hash.to_string()) {
                        all_commits.push((branch.clone(), hash.to_string(), msg.to_string()));
                    }
                }
            }
        }
    }
    all_commits.truncate(count as usize);
    for (i, branch) in branches.iter().enumerate() {
        let rank = i + 1;
        println!("{}. {}:", rank, branch);
        for (_b, hash, msg) in all_commits.iter().filter(|(b, _, _)| b == branch) {
            println!("    {} {}", hash, msg);
        }
    }
}

fn pass_to_git(args: &[String]) {
    let status = Command::new("git").args(args).status().expect("failed to run git");
    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
}
