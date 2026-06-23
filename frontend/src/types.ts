// ---- Dashboard (matches Rust DashboardData) ----

export interface DashboardData {
  today_summary: TodaySummary;
  model_breakdown: ModelBreakdown[];
  latest_trend_point: TrendPoint | null;
  daily_breakdown: DailyBreakdown[];
  provider_ranking: ProviderRank[];
  current_audit: CurrentAudit | null;
}

export interface TodaySummary {
  total_requests: number;
  suspicious_requests: number;
  input_diff_rate: number;
  cache_diff_rate: number;
  output_diff_rate: number;
  input_diff_tokens: number;
  cache_diff_tokens: number;
  output_diff_tokens: number;
  input_match_rate: number;
  cache_match_rate: number;
  output_match_rate: number;
}

export interface ModelBreakdown {
  model: string;
  input_total: number;
  cache_total: number;
  output_total: number;
  input_claimed_total: number;
  cache_claimed_total: number;
  output_claimed_total: number;
}

export interface TrendPoint {
  idx: number;
  input_claimed: number;
  input_detected: number;
  cache_claimed: number;
  cache_detected: number;
  output_claimed: number;
  output_detected: number;
}

export interface DailyBreakdown {
  date: string;
  input_claimed: number;
  input_detected: number;
  cache_claimed: number;
  cache_detected: number;
  output_claimed: number;
  output_detected: number;
}

export interface ProviderRank {
  id: string;
  name: string;
  url: string;
  credibility: number;
  input_diff_rate: number;
  cache_diff_rate: number;
  output_diff_rate: number;
}

export interface CurrentAudit {
  audit_passed: boolean;
  input_audit: number;
  input_claimed: number;
  input_diff: number;
  cache_audit: number;
  cache_claimed: number;
  cache_diff: number;
  output_audit: number;
  output_claimed: number;
  output_diff: number;
}

// ---- Provider (matches Rust AuditSummary + ProviderWithStats) ----

export interface AuditSummary {
  input_claimed: number;
  input_detected: number;
  cache_claimed: number;
  cache_detected: number;
  output_claimed: number;
  output_detected: number;
}

export interface ProviderWithStats {
  id: string;
  name: string;
  provider_type: string;
  api_base_url: string;
  api_key: string;
  supported_models: string[];
  is_active: boolean;
  created_at: string;
  updated_at: string;
  credibility: number | null;
  total_requests: number | null;
  suspicious_requests: number | null;
  audit_summary: AuditSummary | null;
}

// ---- Config ----

export interface AppConfigPayload {
  proxy_port: number;
  language: string;
  auto_start_proxy: boolean;
  dev_mode_enabled: boolean;
  dev_trace_buffer_size: number;
}

// ---- Constants ----

export const ZERO_TREND_POINT: TrendPoint = {
  idx: 0,
  input_claimed: 0,
  input_detected: 0,
  cache_claimed: 0,
  cache_detected: 0,
  output_claimed: 0,
  output_detected: 0,
};
