use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 应用全局配置，存储于用户数据目录下的 `config.json`。
///
/// 安全说明：`api_key` 仅存活于内存及前后端通信中，
/// `save()` 会主动将其置空后再写盘，确保密钥不落地。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// API Key（内存使用，不写入磁盘）
    #[serde(default)]
    pub api_key: String,

    /// AI 服务的 Base URL
    #[serde(default = "default_base_url")]
    pub base_url: String,

    /// 使用的模型名称
    #[serde(default = "default_model_name")]
    pub model_name: String,

    /// 服务商类型标识（如 "openai" / "claude" / "ollama"）
    #[serde(default = "default_provider_type")]
    pub provider_type: String,
}

fn default_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_model_name() -> String {
    "gpt-3.5-turbo".to_string()
}

fn default_provider_type() -> String {
    "openai".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: default_base_url(),
            model_name: default_model_name(),
            provider_type: default_provider_type(),
        }
    }
}

impl AppConfig {
    /// 返回配置文件的存储路径。
    ///
    /// - Windows: `%APPDATA%\moocha\config.json`
    /// - macOS:   `~/Library/Application Support/moocha/config.json`
    /// - Linux:   `~/.local/share/moocha/config.json`
    pub fn config_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("moocha")
            .join("config.json")
    }

    /// 从本地文件加载配置；文件不存在或解析失败时返回默认值。
    /// API Key 始终优先从环境变量 `MOOCHA_API_KEY` 注入。
    pub fn load() -> Self {
        let path = Self::config_path();

        let mut config = if path.exists() {
            match Self::load_from_file(&path) {
                Ok(c) => {
                    tracing::info!("配置加载成功: {}", path.display());
                    c
                }
                Err(e) => {
                    tracing::warn!("配置文件解析失败，使用默认配置: {}", e);
                    Self::default()
                }
            }
        } else {
            tracing::info!("配置文件不存在，使用默认配置");
            Self::default()
        };

        // 环境变量中的 API Key 优先级最高
        if let Ok(key) = std::env::var("MOOCHA_API_KEY") {
            if !key.is_empty() {
                config.api_key = key;
            }
        }

        tracing::info!(
            "当前配置 — provider: {}, model: {}, base_url: {}",
            config.provider_type,
            config.model_name,
            config.base_url,
        );
        config
    }

    /// 将当前配置持久化到磁盘（`api_key` 不写入）。
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("创建配置目录失败: {}", parent.display()))?;
        }

        // api_key 永远不落地
        let disk = AppConfig {
            api_key: String::new(),
            ..self.clone()
        };

        let content = serde_json::to_string_pretty(&disk).context("序列化配置失败")?;

        std::fs::write(&path, content)
            .with_context(|| format!("写入配置文件失败: {}", path.display()))?;

        tracing::info!("配置已保存: {}", path.display());
        Ok(())
    }

    // ── 内部辅助 ──────────────────────────────────────────────────────

    fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("读取配置文件失败: {}", path.display()))?;
        let config: Self = serde_json::from_str(&content)
            .with_context(|| format!("解析 JSON 失败: {}", path.display()))?;
        Ok(config)
    }
}
