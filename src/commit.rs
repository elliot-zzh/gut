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
    // Parse type/scope: desc
    let (typ, rest) = if let Some((typ, rest)) = msg.split_once(":") {
        (typ.trim(), rest.trim())
    } else {
        ("", msg)
    };
    let emoji = footer_emoji_map.get(typ).copied().unwrap_or("");
    let mut formatted = if typ.is_empty() {
        msg.to_string()
    } else {
        format!("{} {}: {}", emoji, typ, rest)
    };
    // Support scope: type(scope): desc
    if let Some((typ_scope, _desc)) = typ.split_once('(') {
        if let Some(scope) = typ_scope.split(')').next() {
            let typ_clean = typ_scope.trim_end_matches('(').trim();
            let emoji = footer_emoji_map.get(typ_clean).copied().unwrap_or("");
            formatted = format!("{} {}({}): {}", emoji, typ_clean, scope.trim_end_matches(')'), rest);
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
