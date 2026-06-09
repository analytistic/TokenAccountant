use rusqlite::{Connection, params};
use anyhow::Result;
use std::path::Path;
use super::migration;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Database { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let current: i32 = self.conn
            .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |r| r.get(0))
            .unwrap_or(0);

        for (i, sql) in migration::MIGRATIONS.iter().enumerate() {
            let version = (i + 1) as i32;
            if version > current {
                self.conn.execute(sql, [])?;
                self.conn.execute(
                    "INSERT OR REPLACE INTO schema_version (version) VALUES (?1)",
                    params![version],
                )?;
            }
        }
        Ok(())
    }

    // ---- Provider CRUD ----

    pub fn list_providers(&self) -> Result<Vec<crate::provider::types::Provider>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, provider_type, api_base_url, api_key, supported_models, is_active, created_at, updated_at FROM providers ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(crate::provider::types::Provider {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: row.get(2)?,
                api_base_url: row.get(3)?,
                api_key: row.get(4)?,
                supported_models: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                is_active: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        let mut providers = Vec::new();
        for row in rows {
            providers.push(row?);
        }
        Ok(providers)
    }

    pub fn get_active_provider(&self) -> Result<Option<crate::provider::types::Provider>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, provider_type, api_base_url, api_key, supported_models, is_active, created_at, updated_at FROM providers WHERE is_active = 1 LIMIT 1"
        )?;
        let mut rows = stmt.query_map([], |row| {
            Ok(crate::provider::types::Provider {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: row.get(2)?,
                api_base_url: row.get(3)?,
                api_key: row.get(4)?,
                supported_models: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                is_active: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        match rows.next() {
            Some(Ok(p)) => Ok(Some(p)),
            _ => Ok(None),
        }
    }

    pub fn add_provider(&self, p: &crate::provider::types::Provider) -> Result<()> {
        self.conn.execute(
            "INSERT INTO providers (id, name, provider_type, api_base_url, api_key, supported_models, is_active) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                p.id, p.name, p.provider_type, p.api_base_url, p.api_key,
                serde_json::to_string(&p.supported_models).unwrap_or_default(),
                p.is_active as i32,
            ],
        )?;
        Ok(())
    }

    pub fn update_provider(&self, p: &crate::provider::types::Provider) -> Result<()> {
        self.conn.execute(
            "UPDATE providers SET name=?1, provider_type=?2, api_base_url=?3, api_key=?4, supported_models=?5, updated_at=datetime('now') WHERE id=?6",
            params![
                p.name, p.provider_type, p.api_base_url, p.api_key,
                serde_json::to_string(&p.supported_models).unwrap_or_default(),
                p.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_provider(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM providers WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn set_active_provider(&self, id: &str) -> Result<()> {
        self.conn.execute("UPDATE providers SET is_active=0", [])?;
        self.conn.execute("UPDATE providers SET is_active=1, updated_at=datetime('now') WHERE id=?1", params![id])?;
        Ok(())
    }

    // ---- Audit Log CRUD ----

    pub fn insert_audit_log(&self, log: &crate::auditor::diff_comparator::AuditRecord) -> Result<()> {
        self.conn.execute(
            "INSERT INTO audit_log (provider_id, model, api_format, claimed_input_tokens, claimed_output_tokens, claimed_cached_tokens, real_input_tokens, real_output_tokens, detected_cached_tokens, input_diff, output_diff, cache_diff, is_suspicious, suspicion_reason, request_preview, response_preview) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                log.provider_id, log.model, log.api_format,
                log.claimed_input_tokens, log.claimed_output_tokens, log.claimed_cached_tokens,
                log.real_input_tokens, log.real_output_tokens, log.detected_cached_tokens,
                log.input_diff, log.output_diff, log.cache_diff,
                log.is_suspicious as i32, log.suspicion_reason,
                log.request_preview, log.response_preview,
            ],
        )?;
        Ok(())
    }

    pub fn list_audit_logs(&self, limit: i64, offset: i64, suspicious_only: bool) -> Result<Vec<crate::auditor::diff_comparator::AuditRecord>> {
        let sql = if suspicious_only {
            "SELECT * FROM audit_log WHERE is_suspicious = 1 ORDER BY timestamp DESC LIMIT ?1 OFFSET ?2"
        } else {
            "SELECT * FROM audit_log ORDER BY timestamp DESC LIMIT ?1 OFFSET ?2"
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok(crate::auditor::diff_comparator::AuditRecord {
                id: Some(row.get(0)?),
                timestamp: row.get(1)?,
                provider_id: row.get(2)?,
                model: row.get(3)?,
                api_format: row.get(4)?,
                claimed_input_tokens: row.get(5)?,
                claimed_output_tokens: row.get(6)?,
                claimed_cached_tokens: row.get(7)?,
                real_input_tokens: row.get(8)?,
                real_output_tokens: row.get(9)?,
                detected_cached_tokens: row.get(10)?,
                input_diff: row.get(11)?,
                output_diff: row.get(12)?,
                cache_diff: row.get(13)?,
                is_suspicious: row.get::<_, i32>(14)? != 0,
                suspicion_reason: row.get(15)?,
                request_preview: row.get(16)?,
                response_preview: row.get(17)?,
            })
        })?;
        let mut logs = Vec::new();
        for row in rows {
            logs.push(row?);
        }
        Ok(logs)
    }

    pub fn get_audit_stats(&self, provider_id: &str) -> Result<crate::auditor::diff_comparator::AuditStats> {
        let total: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM audit_log WHERE provider_id=?1", params![provider_id], |r| r.get(0)
        ).unwrap_or(0);
        let suspicious: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM audit_log WHERE provider_id=?1 AND is_suspicious=1", params![provider_id], |r| r.get(0)
        ).unwrap_or(0);
        let avg_input_diff: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(input_diff), 0) FROM audit_log WHERE provider_id=?1", params![provider_id], |r| r.get(0)
        ).unwrap_or(0.0);
        let avg_output_diff: f64 = self.conn.query_row(
            "SELECT COALESCE(AVG(output_diff), 0) FROM audit_log WHERE provider_id=?1", params![provider_id], |r| r.get(0)
        ).unwrap_or(0.0);

        Ok(crate::auditor::diff_comparator::AuditStats {
            total_requests: total,
            suspicious_requests: suspicious,
            avg_input_diff,
            avg_output_diff,
        })
    }
}
