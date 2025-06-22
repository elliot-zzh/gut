use serde_json::Value;
use crate::utils::pass_to_git;

pub fn gut_commit(args: &[String], config: &Value) {
    if args.is_empty() {
        eprintln!("gut commit <msg>");
        std::process::exit(1);
    }
    let msg = &args[args.len() - 1];
    let formatted = format_commit_message(msg, config);
    let mut git_args = vec!["commit".to_string(), "-m".to_string(), formatted];
    if args.len() > 1 {
        git_args.splice(1..1, args[..args.len()-1].to_vec());
    }
    pass_to_git(&git_args);
}

pub fn format_commit_message(msg: &str, config: &Value) -> String {
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
