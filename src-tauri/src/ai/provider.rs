use anyhow::Result;

/// 对话上下文条目
#[derive(Debug, Clone)]
pub struct ContextMessage {
    pub role: String,
    pub content: String,
}

/// 所有 AI 后端必须实现的统一接口。
/// 无论是本地模型（Ollama）还是云端 API（OpenAI、Claude 等），
/// 都通过此 trait 与应用其余部分解耦。
#[async_trait::async_trait]
pub trait AIProvider: Send + Sync {
    /// 发送一条消息，附带可选的历史上下文，返回模型回复。
    async fn chat(
        &self,
        message: &str,
        context: &[ContextMessage],
    ) -> Result<String>;

    /// 返回当前 Provider 的可读名称，用于日志和 UI 展示。
    fn name(&self) -> &str;
}
