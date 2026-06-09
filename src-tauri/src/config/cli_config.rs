use anyhow::Result;
use serde_json::{Value, json};
use std::path::PathBuf;

pub fn claude_settings_path() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".claude").join("settings.json"))
        .unwrap_or_else(|| PathBuf::from(".claude/settings.json"))
}

/// Write Claude Code settings.json to point to our proxy
pub fn write_claude_settings(proxy_port: u16, api_key: &str) -> Result<()> {
    let path = claude_settings_path();
    let settings = if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str::<Value>(&content).unwrap_or(json!({}))
    } else {
        json!({})
    };

    let mut settings = settings;
    settings["env"] = json!({
        "ANTHROPIC_BASE_URL": format!("http://localhost:{}", proxy_port),
        "ANTHROPIC_AUTH_TOKEN": api_key,
    });

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

/// Restore original settings (remove proxy env vars)
pub fn restore_claude_settings() -> Result<()> {
    let path = claude_settings_path();
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(&path)?;
    let mut settings: Value = serde_json::from_str(&content)?;
    if let Some(obj) = settings.as_object_mut() {
        obj.remove("env");
    }
    let content = serde_json::to_string_pretty(&settings)?;
    std::fs::write(&path, content)?;
    Ok(())
}
