use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    #[serde(rename = "provider_type")]
    pub provider_type: String,
    pub api_base_url: String,
    pub api_key: String,
    pub supported_models: Vec<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub provider_type: String,
    pub api_base_url: String,
    pub api_key: String,
    pub supported_models: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub provider_type: Option<String>,
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub supported_models: Option<Vec<String>>,
}
