use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

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
    /// 非流式对话：发送消息，附带可选历史上下文，阻塞直到返回完整回复。
    async fn chat(
        &self,
        message: &str,
        context: &[ContextMessage],
    ) -> Result<String>;

    /// 流式对话：逐块通过 `tx` 推送文本片段，发送完毕后 `tx` 自动 drop。
    ///
    /// 默认实现：直接调用 `chat()` 并将完整回复作为单块推送，
    /// 供不支持流式的 Provider 继承。
    async fn chat_stream(
        &self,
        message: &str,
        context: &[ContextMessage],
        tx: UnboundedSender<String>,
    ) -> Result<()> {
        let response = self.chat(message, context).await?;
        let _ = tx.send(response);
        Ok(())
    }

    /// 返回当前 Provider 的可读名称，用于日志和 UI 展示。
    fn name(&self) -> &str;
}
