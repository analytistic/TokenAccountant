use rusqlite::{Connection, params};
use anyhow::Result;
use std::path::Path;
use super::migration;
use crate::api::types::{
    DashboardData, TodaySummary, ModelBreakdown, TrendPoint,
    DailyBreakdown, ProviderRank, CurrentAudit, AuditSummary, ProviderWithStats,
};

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

    // ---- Dashboard Aggregation ----

    pub fn get_dashboard_data(&self) -> Result<DashboardData> {
        // Today summary
        let today_summary = self.conn.query_row(
            "SELECT
                COUNT(*),
                COALESCE(SUM(CASE WHEN is_suspicious = 1 THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(claimed_input_tokens), 0),
                COALESCE(SUM(real_input_tokens), 0),
                COALESCE(SUM(claimed_cached_tokens), 0),
                COALESCE(SUM(detected_cached_tokens), 0),
                COALESCE(SUM(claimed_output_tokens), 0),
                COALESCE(SUM(real_output_tokens), 0)
            FROM audit_log
            WHERE date(timestamp) = date('now')",
            [],
            |row| {
                let total: i64 = row.get(0)?;
                let suspicious: i64 = row.get(1)?;
                let claimed_input: i64 = row.get(2)?;
                let real_input: i64 = row.get(3)?;
                let claimed_cache: i64 = row.get(4)?;
                let real_cache: i64 = row.get(5)?;
                let claimed_output: i64 = row.get(6)?;
                let real_output: i64 = row.get(7)?;

                let input_diff_rate = if claimed_input > 0 { (claimed_input - real_input) as f64 / claimed_input as f64 * 100.0 } else { 0.0 };
                let cache_diff_rate = if claimed_cache > 0 { (claimed_cache - real_cache) as f64 / claimed_cache as f64 * 100.0 } else { 0.0 };
                let output_diff_rate = if claimed_output > 0 { (claimed_output - real_output) as f64 / claimed_output as f64 * 100.0 } else { 0.0 };

                Ok(TodaySummary {
                    total_requests: total,
                    suspicious_requests: suspicious,
                    input_diff_rate,
                    cache_diff_rate,
                    output_diff_rate,
                    input_diff_tokens: claimed_input - real_input,
                    cache_diff_tokens: claimed_cache - real_cache,
                    output_diff_tokens: claimed_output - real_output,
                    input_match_rate: (100.0 - input_diff_rate).max(0.0).min(100.0),
                    cache_match_rate: (100.0 - cache_diff_rate).max(0.0).min(100.0),
                    output_match_rate: (100.0 - output_diff_rate).max(0.0).min(100.0),
                })
            },
        )?;

        // Model breakdown (today, grouped by model)
        let model_breakdown = {
            let mut stmt = self.conn.prepare(
                "SELECT model,
                    COALESCE(SUM(real_input_tokens), 0),
                    COALESCE(SUM(detected_cached_tokens), 0),
                    COALESCE(SUM(real_output_tokens), 0),
                    COALESCE(SUM(claimed_input_tokens), 0),
                    COALESCE(SUM(claimed_cached_tokens), 0),
                    COALESCE(SUM(claimed_output_tokens), 0)
                FROM audit_log
                WHERE date(timestamp) = date('now')
                GROUP BY model
                ORDER BY SUM(real_input_tokens) + SUM(real_output_tokens) DESC"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(ModelBreakdown {
                    model: row.get(0)?,
                    input_total: row.get(1)?,
                    cache_total: row.get(2)?,
                    output_total: row.get(3)?,
                    input_claimed_total: row.get(4)?,
                    cache_claimed_total: row.get(5)?,
                    output_claimed_total: row.get(6)?,
                })
            })?;
            let mut models = Vec::new();
            for r in rows {
                models.push(r?);
            }
            models
        };

        // Latest trend point
        let latest_trend_point = {
            let mut stmt = self.conn.prepare(
                "SELECT id, claimed_input_tokens, real_input_tokens,
                    claimed_cached_tokens, detected_cached_tokens,
                    claimed_output_tokens, real_output_tokens
                FROM audit_log ORDER BY id DESC LIMIT 1"
            )?;
            let mut rows = stmt.query_map([], |row| {
                Ok(TrendPoint {
                    idx: row.get(0)?,
                    input_claimed: row.get(1)?,
                    input_detected: row.get(2)?,
                    cache_claimed: row.get(3)?,
                    cache_detected: row.get(4)?,
                    output_claimed: row.get(5)?,
                    output_detected: row.get(6)?,
                })
            })?;
            rows.next().transpose()?
        };

        // Daily breakdown (last 7 days)
        let daily_breakdown = {
            let mut stmt = self.conn.prepare(
                "SELECT date(timestamp) as d,
                    COALESCE(SUM(claimed_input_tokens), 0),
                    COALESCE(SUM(real_input_tokens), 0),
                    COALESCE(SUM(claimed_cached_tokens), 0),
                    COALESCE(SUM(detected_cached_tokens), 0),
                    COALESCE(SUM(claimed_output_tokens), 0),
                    COALESCE(SUM(real_output_tokens), 0)
                FROM audit_log
                WHERE timestamp >= datetime('now', '-7 days')
                GROUP BY d
                ORDER BY d DESC
                LIMIT 7"
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(DailyBreakdown {
                    date: row.get(0)?,
                    input_claimed: row.get(1)?,
                    input_detected: row.get(2)?,
                    cache_claimed: row.get(3)?,
                    cache_detected: row.get(4)?,
                    output_claimed: row.get(5)?,
                    output_detected: row.get(6)?,
                })
            })?;
            let mut days = Vec::new();
            for r in rows {
                days.push(r?);
            }
            // Reverse so oldest first (for chart left-to-right)
            days.reverse();
            days
        };

        // Provider ranking (reuse list_providers_with_stats)
        let provider_ranking = {
            let providers = self.list_providers_with_stats()?;
            providers
                .into_iter()
                .filter(|p| p.credibility.is_some())
                .map(|p| ProviderRank {
                    id: p.id,
                    name: p.name,
                    url: p.api_base_url,
                    credibility: p.credibility.unwrap_or(100.0),
                    input_diff_rate: self.compute_diff_rate(
                        p.audit_summary.as_ref().map(|s| s.input_claimed).unwrap_or(0),
                        p.audit_summary.as_ref().map(|s| s.input_detected).unwrap_or(0),
                    ),
                    cache_diff_rate: self.compute_diff_rate(
                        p.audit_summary.as_ref().map(|s| s.cache_claimed).unwrap_or(0),
                        p.audit_summary.as_ref().map(|s| s.cache_detected).unwrap_or(0),
                    ),
                    output_diff_rate: self.compute_diff_rate(
                        p.audit_summary.as_ref().map(|s| s.output_claimed).unwrap_or(0),
                        p.audit_summary.as_ref().map(|s| s.output_detected).unwrap_or(0),
                    ),
                })
                .collect()
        };

        // Current audit (latest one)
        let current_audit = {
            let mut stmt = self.conn.prepare(
                "SELECT is_suspicious,
                    real_input_tokens, claimed_input_tokens, input_diff,
                    detected_cached_tokens, claimed_cached_tokens, cache_diff,
                    real_output_tokens, claimed_output_tokens, output_diff
                FROM audit_log ORDER BY id DESC LIMIT 1"
            )?;
            let mut rows = stmt.query_map([], |row| {
                Ok(CurrentAudit {
                    audit_passed: row.get::<_, i32>(0)? == 0,
                    input_audit: row.get(1)?,
                    input_claimed: row.get(2)?,
                    input_diff: row.get(3)?,
                    cache_audit: row.get(4)?,
                    cache_claimed: row.get(5)?,
                    cache_diff: row.get(6)?,
                    output_audit: row.get(7)?,
                    output_claimed: row.get(8)?,
                    output_diff: row.get(9)?,
                })
            })?;
            rows.next().transpose()?
        };

        Ok(DashboardData {
            today_summary,
            model_breakdown,
            latest_trend_point,
            daily_breakdown,
            provider_ranking,
            current_audit,
        })
    }

    /// Compute diff rate percentage: (claimed - detected) / claimed * 100
    fn compute_diff_rate(&self, claimed: i64, detected: i64) -> f64 {
        if claimed > 0 {
            (claimed - detected) as f64 / claimed as f64 * 100.0
        } else {
            0.0
        }
    }

    /// List providers with aggregated audit statistics
    pub fn list_providers_with_stats(&self) -> Result<Vec<ProviderWithStats>> {
        let providers = self.list_providers()?;
        let mut result = Vec::new();

        for p in providers {
            let total: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM audit_log WHERE provider_id=?1",
                params![p.id],
                |r| r.get(0),
            ).unwrap_or(0);

            let suspicious: i64 = self.conn.query_row(
                "SELECT COUNT(*) FROM audit_log WHERE provider_id=?1 AND is_suspicious=1",
                params![p.id],
                |r| r.get(0),
            ).unwrap_or(0);

            let audit_summary = if total > 0 {
                self.conn.query_row(
                    "SELECT
                        COALESCE(SUM(claimed_input_tokens), 0),
                        COALESCE(SUM(real_input_tokens), 0),
                        COALESCE(SUM(claimed_cached_tokens), 0),
                        COALESCE(SUM(detected_cached_tokens), 0),
                        COALESCE(SUM(claimed_output_tokens), 0),
                        COALESCE(SUM(real_output_tokens), 0)
                    FROM audit_log WHERE provider_id=?1",
                    params![p.id],
                    |row| {
                        Ok(AuditSummary {
                            input_claimed: row.get(0)?,
                            input_detected: row.get(1)?,
                            cache_claimed: row.get(2)?,
                            cache_detected: row.get(3)?,
                            output_claimed: row.get(4)?,
                            output_detected: row.get(5)?,
                        })
                    },
                ).ok()
            } else {
                None
            };

            let credibility = audit_summary.as_ref().map(|s| {
                let input_over = (s.input_claimed - s.input_detected).max(0) as f64;
                let cache_over = (s.cache_claimed - s.cache_detected).max(0) as f64;
                let output_over = (s.output_claimed - s.output_detected).max(0) as f64;

                let input_over_rate = if s.input_claimed > 0 { input_over / s.input_claimed as f64 * 100.0 } else { 0.0 };
                let cache_over_rate = if s.cache_claimed > 0 { cache_over / s.cache_claimed as f64 * 100.0 } else { 0.0 };
                let output_over_rate = if s.output_claimed > 0 { output_over / s.output_claimed as f64 * 100.0 } else { 0.0 };

                let composite = (input_over_rate * 10.0 + cache_over_rate * 2.0 + output_over_rate * 5.0) / 17.0;
                (100.0 - composite).max(0.0).min(100.0)
            });

            result.push(ProviderWithStats {
                id: p.id,
                name: p.name,
                provider_type: p.provider_type,
                api_base_url: p.api_base_url,
                api_key: p.api_key,
                supported_models: p.supported_models,
                is_active: p.is_active,
                created_at: p.created_at,
                updated_at: p.updated_at,
                credibility,
                total_requests: if total > 0 { Some(total) } else { None },
                suspicious_requests: if suspicious > 0 { Some(suspicious) } else { None },
                audit_summary,
            });
        }

        Ok(result)
    }

    /// Get audit summary for a specific provider + model combination
    pub fn get_provider_detail(&self, provider_id: &str, model: &str) -> Result<AuditSummary> {
        Ok(self.conn.query_row(
            "SELECT
                COALESCE(SUM(claimed_input_tokens), 0),
                COALESCE(SUM(real_input_tokens), 0),
                COALESCE(SUM(claimed_cached_tokens), 0),
                COALESCE(SUM(detected_cached_tokens), 0),
                COALESCE(SUM(claimed_output_tokens), 0),
                COALESCE(SUM(real_output_tokens), 0)
            FROM audit_log
            WHERE provider_id=?1 AND model=?2",
            params![provider_id, model],
            |row| {
                Ok(AuditSummary {
                    input_claimed: row.get(0)?,
                    input_detected: row.get(1)?,
                    cache_claimed: row.get(2)?,
                    cache_detected: row.get(3)?,
                    output_claimed: row.get(4)?,
                    output_detected: row.get(5)?,
                })
            },
        )?)
    }
}
