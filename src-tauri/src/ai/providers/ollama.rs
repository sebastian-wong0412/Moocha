use anyhow::{Context, Result};
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::ai::provider::{AIProvider, ContextMessage};

// ── 请求 / 响应结构体 ────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// 非流式响应
#[derive(Deserialize)]
struct ChatResponse {
    message: Message,
}

/// 流式响应（NDJSON 每行一条）
#[derive(Deserialize)]
struct StreamChunk {
    message: Option<Message>,
    done: bool,
}

// ── Ollama Provider ──────────────────────────────────────────────────────────

/// 本地 Ollama 服务的 Provider。
/// 默认端点：`http://localhost:11434`，使用 `/api/chat` 接口。
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: String, model: String) -> Self {
        let client = Client::builder()
            // Ollama 本地模型推理可能较慢，给更长超时
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("reqwest client build failed");
        Self { client, base_url, model }
    }

    fn endpoint(&self) -> String {
        format!("{}/api/chat", self.base_url.trim_end_matches('/'))
    }

    fn build_messages(&self, message: &str, context: &[ContextMessage]) -> Vec<Message> {
        let mut msgs: Vec<Message> = context
            .iter()
            .map(|c| Message { role: c.role.clone(), content: c.content.clone() })
            .collect();
        msgs.push(Message { role: "user".to_string(), content: message.to_string() });
        msgs
    }
}

#[async_trait::async_trait]
impl AIProvider for OllamaProvider {
    async fn chat(&self, message: &str, context: &[ContextMessage]) -> Result<String> {
        let body = ChatRequest {
            model: &self.model,
            messages: self.build_messages(message, context),
            stream: false,
        };

        let resp = self
            .client
            .post(self.endpoint())
            .json(&body)
            .send()
            .await
            .context("发送请求失败")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ollama API 错误 (HTTP {}): {}", status, text);
        }

        let data: ChatResponse = resp.json().await.context("解析响应失败")?;
        Ok(data.message.content)
    }

    /// 流式调用：逐行解析 NDJSON，通过 `tx` 推送文本片段。
    async fn chat_stream(
        &self,
        message: &str,
        context: &[ContextMessage],
        tx: UnboundedSender<String>,
    ) -> Result<()> {
        let body = ChatRequest {
            model: &self.model,
            messages: self.build_messages(message, context),
            stream: true,
        };

        let resp = self
            .client
            .post(self.endpoint())
            .json(&body)
            .send()
            .await
            .context("发送流式请求失败")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ollama 流式请求错误 (HTTP {}): {}", status, text);
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.context("读取流数据失败")?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            // 按行处理 NDJSON
            while let Some(nl) = buffer.find('\n') {
                let line = buffer[..nl].trim().to_string();
                buffer = buffer[nl + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                if let Ok(item) = serde_json::from_str::<StreamChunk>(&line) {
                    if let Some(msg) = item.message {
                        if !msg.content.is_empty() && tx.send(msg.content).is_err() {
                            return Ok(());
                        }
                    }
                    if item.done {
                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "ollama"
    }
}
