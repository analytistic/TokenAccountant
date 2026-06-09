pub const MIGRATIONS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS schema_version (
        version INTEGER PRIMARY KEY
    )",
    "CREATE TABLE IF NOT EXISTS providers (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        provider_type TEXT NOT NULL DEFAULT 'relay',
        api_base_url TEXT NOT NULL,
        api_key TEXT NOT NULL,
        supported_models TEXT NOT NULL DEFAULT '[]',
        is_active INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    )",
    "CREATE TABLE IF NOT EXISTS audit_log (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL DEFAULT (datetime('now')),
        provider_id TEXT NOT NULL,
        model TEXT NOT NULL,
        api_format TEXT NOT NULL,
        claimed_input_tokens INTEGER NOT NULL DEFAULT 0,
        claimed_output_tokens INTEGER NOT NULL DEFAULT 0,
        claimed_cached_tokens INTEGER NOT NULL DEFAULT 0,
        real_input_tokens INTEGER NOT NULL DEFAULT 0,
        real_output_tokens INTEGER NOT NULL DEFAULT 0,
        detected_cached_tokens INTEGER NOT NULL DEFAULT 0,
        input_diff INTEGER NOT NULL DEFAULT 0,
        output_diff INTEGER NOT NULL DEFAULT 0,
        cache_diff INTEGER NOT NULL DEFAULT 0,
        is_suspicious INTEGER NOT NULL DEFAULT 0,
        suspicion_reason TEXT,
        request_preview TEXT,
        response_preview TEXT,
        FOREIGN KEY (provider_id) REFERENCES providers(id)
    )",
    "CREATE TABLE IF NOT EXISTS cache_state (
        block_hash TEXT PRIMARY KEY,
        block_data BLOB NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        access_count INTEGER NOT NULL DEFAULT 1
    )",
    "CREATE INDEX IF NOT EXISTS idx_audit_provider ON audit_log(provider_id)",
    "CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp)",
    "CREATE INDEX IF NOT EXISTS idx_audit_suspicious ON audit_log(is_suspicious)",
];

pub fn get_schema_version() -> i32 {
    MIGRATIONS.len() as i32
}
