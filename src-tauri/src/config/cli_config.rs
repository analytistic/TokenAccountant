use anyhow::Result;
use serde_json::{Value, json};
use std::path::PathBuf;

pub fn claude_settings_path() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".claude").join("settings.json"))
        .unwrap_or_else(|| PathBuf::from(".claude/settings.json"))
}

/// Write Claude Code settings.json to point to the given base URL + API key.
/// Preserves all existing env fields except ANTHROPIC_BASE_URL and ANTHROPIC_AUTH_TOKEN.
/// - Provider selected: write_claude_settings(provider_url, provider_api_key)
/// - Proxy started:      write_claude_settings("http://localhost:{port}", api_key)
/// - Proxy stopped:      write_claude_settings(provider_url, provider_api_key)
pub fn write_claude_settings(base_url: &str, api_key: &str) -> Result<()> {
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

    env.insert("ANTHROPIC_BASE_URL".into(), json!(base_url));
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
