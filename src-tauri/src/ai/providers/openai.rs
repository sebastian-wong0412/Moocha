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
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

// ── OpenAI Provider ──────────────────────────────────────────────────────────

/// 兼容 OpenAI Chat Completions API 的 Provider。
/// 支持任何 OpenAI 兼容端点（OpenAI、Azure、各类国内代理等）。
pub struct OpenAIProvider {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("reqwest client build failed");
        Self { client, base_url, api_key, model }
    }

    fn endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
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
impl AIProvider for OpenAIProvider {
    async fn chat(&self, message: &str, context: &[ContextMessage]) -> Result<String> {
        let body = ChatRequest {
            model: &self.model,
            messages: self.build_messages(message, context),
            stream: false,
        };

        let mut req = self.client.post(self.endpoint()).json(&body);
        if !self.api_key.is_empty() {
            req = req.bearer_auth(&self.api_key);
        }

        let resp = req.send().await.context("发送请求失败")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API 错误 (HTTP {}): {}", status, text);
        }

        let data: ChatResponse = resp.json().await.context("解析响应失败")?;
        data.choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| anyhow::anyhow!("响应中没有 choices"))
    }

    /// 流式调用：逐块解析 Server-Sent Events，通过 `tx` 推送文本片段。
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

        let mut req = self.client.post(self.endpoint()).json(&body);
        if !self.api_key.is_empty() {
            req = req.bearer_auth(&self.api_key);
        }

        let resp = req.send().await.context("发送流式请求失败")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI 流式请求错误 (HTTP {}): {}", status, text);
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.context("读取流数据失败")?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            // 按行处理 SSE 数据
            while let Some(nl) = buffer.find('\n') {
                let line = buffer[..nl].trim().to_string();
                buffer = buffer[nl + 1..].to_string();

                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        return Ok(());
                    }
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                            if !content.is_empty() && tx.send(content.to_string()).is_err() {
                                // 接收端已关闭（前端断开），提前退出
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "openai"
    }
}
