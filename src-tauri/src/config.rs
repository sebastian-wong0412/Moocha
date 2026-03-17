use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 支持的 AI 服务商类型。
/// 添加新服务商时只需扩展此枚举，无需改动核心逻辑。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    /// OpenAI 兼容接口（包括本地 Ollama）
    OpenAiCompatible,
    /// Anthropic Claude
    Claude,
    /// 自定义 HTTP 接口
    Custom,
}

impl Default for ProviderType {
    fn default() -> Self {
        Self::OpenAiCompatible
    }
}

/// 应用全局配置，存储于用户数据目录下的 `config.json`。
/// API Key 不在此持久化——请通过环境变量或 `.env` 文件提供。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// AI 服务的 Base URL，留空则使用各 Provider 默认值
    #[serde(default = "default_base_url")]
    pub base_url: String,

    /// 使用的模型名称
    #[serde(default = "default_model_name")]
    pub model_name: String,

    /// 服务商类型
    #[serde(default)]
    pub provider_type: ProviderType,

    /// API Key（运行时从环境变量注入，不写入磁盘）
    #[serde(skip)]
    pub api_key: String,
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_model_name() -> String {
    "gpt-4o-mini".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            model_name: default_model_name(),
            provider_type: ProviderType::default(),
            api_key: String::new(),
        }
    }
}

impl AppConfig {
    /// 从指定路径加载配置文件；文件不存在时返回默认配置。
    pub fn load(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            tracing::info!("配置文件不存在，使用默认配置: {}", path.display());
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("读取配置文件失败: {}", path.display()))?;

        let mut config: Self = serde_json::from_str(&content)
            .with_context(|| format!("解析配置文件失败: {}", path.display()))?;

        // API Key 始终从环境变量读取，避免意外写入磁盘
        config.api_key = std::env::var("MOOCHA_API_KEY").unwrap_or_default();

        tracing::info!("配置加载成功，Provider: {:?}", config.provider_type);
        Ok(config)
    }

    /// 将当前配置（不含 api_key）持久化到磁盘。
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("创建配置目录失败: {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(self)
            .context("序列化配置失败")?;

        std::fs::write(path, content)
            .with_context(|| format!("写入配置文件失败: {}", path.display()))?;

        tracing::info!("配置已保存: {}", path.display());
        Ok(())
    }
}
