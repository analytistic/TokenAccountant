use anyhow::Result;
use serde_json::{Value, json};
use std::path::PathBuf;

pub fn claude_settings_path() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".claude").join("settings.json"))
        .unwrap_or_else(|| PathBuf::from(".claude/settings.json"))
}

/// Write Claude Code settings.json to point to our proxy.
/// Preserves all existing env fields (model mappings, effort level, etc.)
pub fn write_claude_settings(proxy_port: u16, api_key: &str) -> Result<()> {
    let path = claude_settings_path();
    let mut settings = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str::<Value>(&content).unwrap_or(json!({}))
    } else {
        json!({})
    };

    // Merge into existing env instead of replacing
    let mut env = settings
        .get("env")
        .and_then(|v| v.as_object())
        .map(|obj| {
            let mut m = serde_json::Map::new();
            for (k, v) in obj {
                m.insert(k.clone(), v.clone());
            }
            m
        })
        .unwrap_or_else(|| serde_json::Map::new());

    env.insert("ANTHROPIC_BASE_URL".into(), json!(format!("http://localhost:{}", proxy_port)));
    env.insert("ANTHROPIC_AUTH_TOKEN".into(), json!(api_key));
    settings["env"] = Value::Object(env);

    // Atomic write: temp file + rename
    let parent = path.parent().unwrap();
    std::fs::create_dir_all(parent)?;
    let tmp_path = parent.join("settings.json.tmp");
    let content = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&tmp_path, &content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600))?;
    }
    std::fs::rename(&tmp_path, &path)?;

    Ok(())
}

/// Restore env settings after proxy stops.
/// If `original_env` is provided, restore it exactly; otherwise just remove proxy fields.
pub fn restore_claude_settings(original_env: Option<&Value>) -> Result<()> {
    let path = claude_settings_path();
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&path)?;
    let mut settings: Value = serde_json::from_str(&content)?;

    if let Some(orig) = original_env {
        settings["env"] = orig.clone();
    } else if let Some(env) = settings.get_mut("env").and_then(|e| e.as_object_mut()) {
        env.remove("ANTHROPIC_BASE_URL");
        env.remove("ANTHROPIC_AUTH_TOKEN");
    }

    let content = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&path, content)?;
    Ok(())
}
