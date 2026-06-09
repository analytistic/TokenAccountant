use super::types::CreateProviderRequest;

pub fn default_presets() -> Vec<CreateProviderRequest> {
    vec![
        CreateProviderRequest {
            name: "OpenAI Official".into(),
            provider_type: "official".into(),
            api_base_url: "https://api.openai.com".into(),
            api_key: "".into(),
            supported_models: vec!["gpt-4".into(), "gpt-4o".into(), "gpt-3.5-turbo".into()],
        },
        CreateProviderRequest {
            name: "Anthropic Official".into(),
            provider_type: "official".into(),
            api_base_url: "https://api.anthropic.com".into(),
            api_key: "".into(),
            supported_models: vec!["claude-3-5-sonnet".into(), "claude-3-opus".into()],
        },
    ]
}
