use serde::{Deserialize, Serialize};

/// Single-command return for all Dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub today_summary: TodaySummary,
    pub model_breakdown: Vec<ModelBreakdown>,
    pub latest_trend_point: Option<TrendPoint>,
    pub daily_breakdown: Vec<DailyBreakdown>,
    pub provider_ranking: Vec<ProviderRank>,
    pub current_audit: Option<CurrentAudit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodaySummary {
    pub total_requests: i64,
    pub suspicious_requests: i64,
    pub input_diff_rate: f64,
    pub cache_diff_rate: f64,
    pub output_diff_rate: f64,
    pub input_diff_tokens: i64,
    pub cache_diff_tokens: i64,
    pub output_diff_tokens: i64,
    pub input_match_rate: f64,
    pub cache_match_rate: f64,
    pub output_match_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBreakdown {
    pub model: String,
    pub input_total: i64,
    pub cache_total: i64,
    pub output_total: i64,
    pub input_claimed_total: i64,
    pub cache_claimed_total: i64,
    pub output_claimed_total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendPoint {
    pub idx: i64,
    pub input_claimed: i64,
    pub input_detected: i64,
    pub cache_claimed: i64,
    pub cache_detected: i64,
    pub output_claimed: i64,
    pub output_detected: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyBreakdown {
    pub date: String,
    pub input_claimed: i64,
    pub input_detected: i64,
    pub cache_claimed: i64,
    pub cache_detected: i64,
    pub output_claimed: i64,
    pub output_detected: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRank {
    pub id: String,
    pub name: String,
    pub url: String,
    pub credibility: f64,
    pub input_diff_rate: f64,
    pub cache_diff_rate: f64,
    pub output_diff_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentAudit {
    pub audit_passed: bool,
    pub input_audit: i64,
    pub input_claimed: i64,
    pub input_diff: i64,
    pub cache_audit: i64,
    pub cache_claimed: i64,
    pub cache_diff: i64,
    pub output_audit: i64,
    pub output_claimed: i64,
    pub output_diff: i64,
}

/// I/C/O claimed vs detected summary for a single provider+model combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub input_claimed: i64,
    pub input_detected: i64,
    pub cache_claimed: i64,
    pub cache_detected: i64,
    pub output_claimed: i64,
    pub output_detected: i64,
}

/// Config payload sent to/from frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfigPayload {
    pub proxy_port: u16,
    pub language: String,
    pub auto_start_proxy: bool,
    pub dev_mode_enabled: bool,
    pub dev_trace_buffer_size: u32,
}

/// Provider with aggregated audit stats (returned by list_providers)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderWithStats {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub api_base_url: String,
    pub api_key: String,
    pub supported_models: Vec<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub credibility: Option<f64>,
    pub total_requests: Option<i64>,
    pub suspicious_requests: Option<i64>,
    pub audit_summary: Option<AuditSummary>,
}
