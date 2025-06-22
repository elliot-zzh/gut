use std::path::Path;

pub fn load_config() -> serde_json::Value {
    let config_path = Path::new("gut.config.json");
    if !config_path.exists() { return serde_json::json!({}); }
    let config = std::fs::read_to_string(config_path).unwrap_or_default();
    serde_json::from_str(&config).unwrap_or_default()
}

pub fn check_and_generate_hooks(config: &serde_json::Value) {
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
                    use std::io::Write;
                    let _ = f.write_all(script.as_bytes());
                    let _ = std::fs::set_permissions(&hook_path, std::os::unix::fs::PermissionsExt::from_mode(0o755));
                }
            }
        }
    }
}
