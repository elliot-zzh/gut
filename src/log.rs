use crate::utils::pass_to_git;
use serde_json::Value;

pub fn gut_log(_args: &[String], config: &Value) {
    let binding = serde_json::json!({});
    let log_cfg = config.get("log").unwrap_or(&binding);
    let count = log_cfg.get("count").and_then(|v| v.as_u64()).unwrap_or(10);
    let info = log_cfg.get("info").and_then(|v| v.as_str()).unwrap_or("less");
    let pretty = if info == "more" {
        "%h %an %ad %s"
    } else {
        "%h %s"
    };
    let pretty_string = format!("--pretty=format:{}", pretty);
    let output = std::process::Command::new("git")
        .args(["log", "-n", &count.to_string(), &pretty_string])
        .output()
        .expect("failed to run git log");
    if output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("git log failed");
        std::process::exit(1);
    }
}

pub fn gut_rlog(args: &[String], config: &Value) {
    let binding = serde_json::json!({});
    let log_cfg = config.get("log").unwrap_or(&binding);
    let count = log_cfg.get("count").and_then(|v| v.as_u64()).unwrap_or(10);
    let info = log_cfg.get("info").and_then(|v| v.as_str()).unwrap_or("less");
    let pretty = if info == "more" {
        "%h %an %ad %s"
    } else {
        "%h %s"
    };
    let pretty_string = format!("--pretty=format:{}", pretty);
    let mut git_args = vec!["log".to_string(), "-n".to_string(), count.to_string(), "--reverse".to_string(), pretty_string];
    git_args.extend(args.iter().cloned());
    pass_to_git(&git_args);
}

pub fn gut_tlog(_args: &[String], config: &Value) {
    let binding = serde_json::json!({});
    let tlog_cfg = config.get("tlog").unwrap_or(&binding);
    let count = tlog_cfg.get("count").and_then(|v| v.as_u64()).unwrap_or(20);
    let info = tlog_cfg.get("info").and_then(|v| v.as_str()).unwrap_or("less");
    let pretty = if info == "more" {
        "%h %an %ad %s"
    } else {
        "%h %s"
    };
    let current_branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .and_then(|o| if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).trim().to_string()) } else { None });
    let mut branches = vec![];
    let branch_output = std::process::Command::new("git")
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
        let pretty_string = format!("--pretty=format:{}", pretty);
        let output = std::process::Command::new("git")
            .args(["log", branch, "-n", &count.to_string(), &pretty_string])
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
