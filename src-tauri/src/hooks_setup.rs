use std::fs;
use std::path::PathBuf;

/// Auto-configure Claude Code hooks in ~/.claude/settings.json on startup.
/// Only adds hooks if they don't already point to our server.
pub fn ensure_claude_hooks_configured(port: u16) {
    let settings_path = claude_settings_path();
    let hook_cmd = format!(
        "bash -c 'data=$(cat); curl -sf --connect-timeout 2 --max-time 3 -X POST http://127.0.0.1:{}/notify -H \"Content-Type: application/json\" -d \"$data\" 2>/dev/null & disown'",
        port
    );

    let mut settings: serde_json::Value = if settings_path.exists() {
        match fs::read_to_string(&settings_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({})),
            Err(_) => return,
        }
    } else {
        serde_json::json!({})
    };

    // Check if hooks already reference our server
    let marker = format!("127.0.0.1:{}/notify", port);
    if let Some(hooks) = settings.get("hooks") {
        let hooks_str = hooks.to_string();
        if hooks_str.contains(&marker) {
            log::info!("Claude Code hooks already configured");
            return;
        }
    }

    // Add our hooks
    let hooks = settings.as_object_mut().unwrap()
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let hooks_obj = hooks.as_object_mut().unwrap();

    // Add Notification hook
    let notification_hook = serde_json::json!([{
        "matcher": "",
        "hooks": [{
            "type": "command",
            "command": &hook_cmd
        }]
    }]);

    // Merge: append to existing array or create new
    if let Some(existing) = hooks_obj.get_mut("Notification") {
        if let Some(arr) = existing.as_array_mut() {
            // Check if we're already in there
            let already = arr.iter().any(|h| h.to_string().contains(&marker));
            if !already {
                if let Some(new_hook) = notification_hook.as_array() {
                    arr.extend(new_hook.iter().cloned());
                }
            }
        }
    } else {
        hooks_obj.insert("Notification".to_string(), notification_hook);
    }

    // Add Stop hook
    let stop_hook = serde_json::json!([{
        "hooks": [{
            "type": "command",
            "command": &hook_cmd
        }]
    }]);

    if let Some(existing) = hooks_obj.get_mut("Stop") {
        if let Some(arr) = existing.as_array_mut() {
            let already = arr.iter().any(|h| h.to_string().contains(&marker));
            if !already {
                if let Some(new_hook) = stop_hook.as_array() {
                    arr.extend(new_hook.iter().cloned());
                }
            }
        }
    } else {
        hooks_obj.insert("Stop".to_string(), stop_hook);
    }

    // Write back
    if let Some(parent) = settings_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(&settings) {
        Ok(json) => {
            if let Err(e) = fs::write(&settings_path, json) {
                log::error!("Failed to write Claude hooks: {}", e);
            } else {
                log::info!("Claude Code hooks configured at {:?}", settings_path);
            }
        }
        Err(e) => log::error!("Failed to serialize Claude hooks: {}", e),
    }
}

fn claude_settings_path() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".claude").join("settings.json")
}
