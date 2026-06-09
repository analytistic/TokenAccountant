use anyhow::Result;
use crate::storage::database::Database;
use super::types::{Provider, CreateProviderRequest, UpdateProviderRequest};
use uuid::Uuid;

pub struct ProviderManager {
    db: std::sync::Arc<tokio::sync::Mutex<Database>>,
}

impl ProviderManager {
    pub fn new(db: std::sync::Arc<tokio::sync::Mutex<Database>>) -> Self {
        ProviderManager { db }
    }

    pub async fn list(&self) -> Result<Vec<Provider>> {
        let db = self.db.lock().await;
        db.list_providers()
    }

    pub async fn get_active(&self) -> Result<Option<Provider>> {
        let db = self.db.lock().await;
        db.get_active_provider()
    }

    pub async fn create(&self, req: CreateProviderRequest) -> Result<Provider> {
        let now = chrono_now();
        let provider = Provider {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            provider_type: req.provider_type,
            api_base_url: req.api_base_url,
            api_key: req.api_key,
            supported_models: req.supported_models,
            is_active: false,
            created_at: now.clone(),
            updated_at: now,
        };
        let db = self.db.lock().await;
        db.add_provider(&provider)?;
        Ok(provider)
    }

    pub async fn update(&self, id: &str, req: UpdateProviderRequest) -> Result<Provider> {
        let db = self.db.lock().await;
        let providers = db.list_providers()?;
        let mut provider = providers.into_iter().find(|p| p.id == id)
            .ok_or_else(|| anyhow::anyhow!("Provider not found: {}", id))?;

        if let Some(name) = req.name { provider.name = name; }
        if let Some(t) = req.provider_type { provider.provider_type = t; }
        if let Some(url) = req.api_base_url { provider.api_base_url = url; }
        if let Some(key) = req.api_key { provider.api_key = key; }
        if let Some(models) = req.supported_models { provider.supported_models = models; }
        provider.updated_at = chrono_now();

        db.update_provider(&provider)?;
        Ok(provider)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        let db = self.db.lock().await;
        db.delete_provider(id)
    }

    pub async fn switch_active(&self, id: &str) -> Result<Provider> {
        let db = self.db.lock().await;
        db.set_active_provider(id)?;
        let provider = db.get_active_provider()?
            .ok_or_else(|| anyhow::anyhow!("Failed to activate provider"))?;
        Ok(provider)
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
    let month_days = if is_leap_year(y) {
        [31,29,31,30,31,30,31,31,30,31,30,31]
    } else {
        [31,28,31,30,31,30,31,31,30,31,30,31]
    };
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
