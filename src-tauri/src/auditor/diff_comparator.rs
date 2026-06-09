use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: Option<i64>,
    pub timestamp: String,
    pub provider_id: String,
    pub model: String,
    pub api_format: String,
    pub claimed_input_tokens: i32,
    pub claimed_output_tokens: i32,
    pub claimed_cached_tokens: i32,
    pub real_input_tokens: i32,
    pub real_output_tokens: i32,
    pub detected_cached_tokens: i32,
    pub input_diff: i32,
    pub output_diff: i32,
    pub cache_diff: i32,
    pub is_suspicious: bool,
    pub suspicion_reason: String,
    pub request_preview: String,
    pub response_preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_requests: i64,
    pub suspicious_requests: i64,
    pub avg_input_diff: f64,
    pub avg_output_diff: f64,
}

pub struct DiffComparator {
    suspicion_threshold: f64,
}

impl DiffComparator {
    pub fn new(suspicion_threshold: f64) -> Self {
        DiffComparator { suspicion_threshold }
    }

    pub fn compare(
        &self,
        provider_id: &str,
        model: &str,
        api_format: &str,
        request_preview: &str,
        response_preview: &str,
        claimed_input: i32,
        claimed_output: i32,
        claimed_cached: i32,
        real_input: i32,
        real_output: i32,
        detected_cached: i32,
    ) -> AuditRecord {
        let input_diff = claimed_input - real_input;
        let output_diff = claimed_output - real_output;
        let cache_diff = claimed_cached - detected_cached;

        let mut reasons = Vec::new();
        if real_input > 0 && (input_diff as f64 / real_input as f64).abs() > self.suspicion_threshold {
            reasons.push(format!("input diff {:+} ({}%)", input_diff, (input_diff as f64 / real_input as f64 * 100.0) as i32));
        }
        if real_output > 0 && (output_diff as f64 / real_output as f64).abs() > self.suspicion_threshold {
            reasons.push(format!("output diff {:+} ({}%)", output_diff, (output_diff as f64 / real_output as f64 * 100.0) as i32));
        }
        if detected_cached > 0 && (cache_diff as f64 / detected_cached as f64).abs() > self.suspicion_threshold {
            reasons.push(format!("cache diff {:+}", cache_diff));
        }
        if claimed_cached > 0 && detected_cached == 0 {
            reasons.push("claimed cached tokens but none detected".into());
        }

        let is_suspicious = !reasons.is_empty();
        let suspicion_reason = reasons.join("; ");

        AuditRecord {
            id: None,
            timestamp: chrono_now(),
            provider_id: provider_id.to_string(),
            model: model.to_string(),
            api_format: api_format.to_string(),
            claimed_input_tokens: claimed_input,
            claimed_output_tokens: claimed_output,
            claimed_cached_tokens: claimed_cached,
            real_input_tokens: real_input,
            real_output_tokens: real_output,
            detected_cached_tokens: detected_cached,
            input_diff,
            output_diff,
            cache_diff,
            is_suspicious,
            suspicion_reason,
            request_preview: truncate(request_preview, 500),
            response_preview: truncate(response_preview, 2000),
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max])
    } else {
        s.to_string()
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let secs = d.as_secs();
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let mins = (time_secs % 3600) / 60;
    let sec = time_secs % 60;
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if remaining < days_in_year { break; }
        remaining -= days_in_year;
        y += 1;
    }
    let month_days = if is_leap_year(y) { [31,29,31,30,31,30,31,31,30,31,30,31] } else { [31,28,31,30,31,30,31,31,30,31,30,31] };
    let mut m = 0;
    for &md in &month_days {
        if remaining < md { break; }
        remaining -= md;
        m += 1;
    }
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m+1, remaining+1, hours, mins, sec)
}

fn is_leap_year(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
