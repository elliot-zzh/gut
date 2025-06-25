use serde_json::Value;
use crate::utils::pass_to_git;

pub fn gut_commit(args: &[String], config: &Value) {
    if args.is_empty() {
        eprintln!("gut commit <msg>");
        std::process::exit(1);
    }
    let msg = &args[args.len() - 1];
    // Check for conventional commit enforcement
    let require_conventional = config.get("commit")
        .and_then(|c| c.get("require_conventional"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if require_conventional && !is_conventional_commit(msg) {
        eprintln!("[gut] conventional commit is required");
        std::process::exit(1);
    }
    let formatted = format_commit_message(msg, config);
    let mut git_args = vec!["commit".to_string(), "-m".to_string(), formatted];
    if args.len() > 1 {
        git_args.splice(1..1, args[..args.len()-1].to_vec());
    }
    pass_to_git(&git_args);
}

fn is_conventional_commit(msg: &str) -> bool {
    // Accepts: type: desc or type(scope): desc
    let msg = msg.trim();
    if let Some((typ, rest)) = msg.split_once(":") {
        let typ = typ.trim();
        if typ.is_empty() || rest.trim().is_empty() {
            return false;
        }
        // Optionally allow scope: type(scope): desc
        if let Some(scope_start) = typ.find('(') {
            let scope_end = typ.find(')');
            if scope_end.is_none() || scope_end.unwrap() < scope_start {
                return false;
            }
        }
        return true;
    }
    false
}

pub fn format_commit_message(msg: &str, config: &Value) -> String {
    // Footer emoji mapping from config
    let mut footer_emoji_map = std::collections::HashMap::new();
    // Default mapping for commit type (footer)
    footer_emoji_map.insert("feat", "✨");
    footer_emoji_map.insert("fix", "🐛");
    footer_emoji_map.insert("docs", "📝");
    footer_emoji_map.insert("refactor", "♻️");
    footer_emoji_map.insert("test", "✅");
    footer_emoji_map.insert("chore", "🔧");
    footer_emoji_map.insert("build", "🏗️");
    footer_emoji_map.insert("style", "🎨");
    footer_emoji_map.insert("ci", "🔁");
    footer_emoji_map.insert("perf", "⚡");
    footer_emoji_map.insert("revert", "⏪");
    // Merge user config mapping
    if let Some(commit_cfg) = config.get("commit") {
        if let Some(footer_map) = commit_cfg.get("footer_emoji").and_then(|v| v.as_object()) {
            for (k, v) in footer_map.iter() {
                if let Some(emoji) = v.as_str() {
                    footer_emoji_map.insert(k.as_str(), emoji);
                }
            }
        }
    }
    // Emoji enabled config
    let emoji_enabled = config.get("commit")
        .and_then(|c| c.get("emoji_enabled"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    // Parse type/scope: desc
    let (typ, rest) = if let Some((typ, rest)) = msg.split_once(":") {
        (typ.trim(), rest.trim())
    } else {
        ("", msg)
    };
    let emoji = if emoji_enabled {
        footer_emoji_map.get(typ).copied().unwrap_or("")
    } else {
        ""
    };
    let mut formatted = if typ.is_empty() {
        msg.to_string()
    } else {
        if emoji_enabled && !emoji.is_empty() {
            format!("{} {}: {}", emoji, typ, rest)
        } else {
            format!("{}: {}", typ, rest)
        }
    };
    // Support scope: type(scope): desc
    if let Some((typ_scope, _desc)) = typ.split_once('(') {
        if let Some(scope) = typ_scope.split(')').next() {
            let typ_clean = typ_scope.trim_end_matches('(').trim();
            let emoji = if emoji_enabled {
                footer_emoji_map.get(typ_clean).copied().unwrap_or("")
            } else {
                ""
            };
            formatted = if emoji_enabled && !emoji.is_empty() {
                format!("{} {}({}): {}", emoji, typ_clean, scope.trim_end_matches(')'), rest)
            } else {
                format!("{}({}): {}", typ_clean, scope.trim_end_matches(')'), rest)
            };
        }
    }
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
