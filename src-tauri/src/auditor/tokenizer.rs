use std::collections::HashMap;
use std::sync::Arc;
use super::message_converter::{Conversation, NormalizedMessage};

/// Optional parameters for chat template rendering,
/// matching vLLM's `encode_messages` parameter set.
///
/// Tokenizers that don't need these (GPT, Claude) ignore them via the
/// default `apply_chat_template_with` implementation.
#[derive(Debug, Clone, Default)]
pub struct TemplateParams {
    /// "thinking" or "chat" — DeepSeek-V4 thinking mode.
    /// Default: None (tokenizer-specific default, e.g. "thinking" for DeepSeek)
    pub thinking_mode: Option<String>,
    /// Drop reasoning from earlier assistant turns before the last user message.
    /// Default: None (tokenizer-specific default, e.g. true for DeepSeek)
    pub drop_thinking: Option<bool>,
    /// Add BOS token at conversation start.
    /// Default: None (tokenizer-specific default, e.g. true for DeepSeek)
    pub add_default_bos_token: Option<bool>,
    /// Reasoning effort level: "max", "high".
    /// Default: None (means no effort prefix)
    pub reasoning_effort: Option<String>,
}

pub trait Tokenizer: Send + Sync {
    fn encode(&self, text: &str) -> Vec<u32>;
    fn decode(&self, ids: &[u32]) -> String;
    fn count_tokens(&self, text: &str) -> u32 {
        self.encode(text).len() as u32
    }
    /// Apply model-specific chat template to a Conversation.
    /// Output is the formatted prompt string ready for tokenization.
    fn apply_chat_template(&self, conv: &Conversation) -> String;
    /// Apply chat template with optional parameters.
    ///
    /// The default implementation ignores `params` and delegates to
    /// `apply_chat_template`. Tokenizers that need these parameters
    /// (e.g. DeepSeek-V4) override this method.
    fn apply_chat_template_with(
        &self,
        conv: &Conversation,
        _params: &TemplateParams,
    ) -> String {
        self.apply_chat_template(conv)
    }
    /// Render a single assistant output message to model-specific format.
    /// Used for accurate output token counting (not raw SSE text).
    fn render_output(&self, msg: &NormalizedMessage) -> String;
}

/// Extract TemplateParams from an Anthropic Messages API request body.
///
/// This is a standalone helper, not part of message_converter — message
/// conversion is only about messages, not renderer parameters.
pub fn extract_template_params(body_str: &str, model: &str) -> TemplateParams {
    let Ok(body_val) = serde_json::from_str::<serde_json::Value>(body_str) else {
        return TemplateParams::default();
    };

    // thinking_mode: if "thinking" field exists with type "enabled"
    let explicit_mode = body_val.get("thinking")
        .and_then(|t| t.get("type").and_then(|v| v.as_str()))
        .and_then(|kind| match kind {
            "enabled" => Some("thinking".to_string()),
            "disabled" => Some("chat".to_string()),
            _ => None,
        });
    let thinking_mode = explicit_mode.or_else(|| {
        if model == "deepseek-chat" { Some("chat".to_string()) }
        else if model == "deepseek-reasoner" { Some("thinking".to_string()) }
        else { None }
    });

    // reasoning_effort: from output_config.effort
    let reasoning_effort = body_val.get("output_config")
        .and_then(|oc| oc.get("effort"))
        .and_then(|e| e.as_str())
        .map(|s| s.to_string());

    TemplateParams {
        thinking_mode,
        drop_thinking: None,   // DeepSeek default: true
        add_default_bos_token: None,  // DeepSeek default: true
        reasoning_effort,
    }
}

pub struct TokenizerFactory {
    tokenizers: HashMap<String, Arc<dyn Tokenizer>>,
}

impl TokenizerFactory {
    pub fn new() -> Self {
        let mut factory = TokenizerFactory {
            tokenizers: HashMap::new(),
        };
        factory.register("gpt".into(), Arc::new(super::tokenizers::gpt::GptTokenizer::new()));
        factory.register("deepseek".into(), Arc::new(super::tokenizers::deepseek::DeepSeekTokenizer::new()));
        factory
    }

    pub fn register(&mut self, name: String, tokenizer: Arc<dyn Tokenizer>) {
        self.tokenizers.insert(name, tokenizer);
    }

    pub fn for_model(&self, model: &str) -> Option<Arc<dyn Tokenizer>> {
        let normalized = model.to_ascii_lowercase();
        if normalized.starts_with("deepseek-") || normalized.contains("/deepseek-") {
            self.tokenizers.get("deepseek").cloned()
        } else if model.starts_with("gpt-") || model.starts_with("text-")
            || model.starts_with("qwen-") {
            self.tokenizers.get("gpt").cloned()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::extract_template_params;

    #[test]
    fn honors_explicit_thinking_mode() {
        assert_eq!(
            extract_template_params(r#"{"thinking":{"type":"disabled"}}"#, "deepseek-v4-pro").thinking_mode.as_deref(),
            Some("chat")
        );
        assert_eq!(
            extract_template_params(r#"{"thinking":{"type":"enabled"}}"#, "deepseek-v4-pro").thinking_mode.as_deref(),
            Some("thinking")
        );
    }

    #[test]
    fn maps_compatibility_model_modes() {
        assert_eq!(extract_template_params("{}", "deepseek-chat").thinking_mode.as_deref(), Some("chat"));
        assert_eq!(extract_template_params("{}", "deepseek-reasoner").thinking_mode.as_deref(), Some("thinking"));
    }

    #[test]
    fn recognizes_official_vllm_model_id() {
        assert!(super::TokenizerFactory::new()
            .for_model("deepseek-ai/DeepSeek-V4-Pro")
            .is_some());
    }
}
