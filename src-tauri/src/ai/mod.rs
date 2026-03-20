pub mod provider;
pub mod providers;

pub use provider::{AIProvider, ContextMessage};

use crate::config::AppConfig;
use providers::{ollama::OllamaProvider, openai::OpenAIProvider};

/// 根据配置创建对应的 AI Provider。
///
/// - `provider_type == "ollama"` → 使用本地 Ollama 服务
/// - 其他（默认）                 → 使用 OpenAI 兼容端点
pub fn create_provider(config: &AppConfig) -> Box<dyn AIProvider> {
    match config.provider_type.as_str() {
        "ollama" => Box::new(OllamaProvider::new(
            config.base_url.clone(),
            config.model_name.clone(),
        )),
        _ => Box::new(OpenAIProvider::new(
            config.base_url.clone(),
            config.api_key.clone(),
            config.model_name.clone(),
        )),
    }
}
