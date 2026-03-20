//! 对话消息持久化（`app_data_dir/chat_history.json`）。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

/// 与前端 `ChatMessage` 对齐的单条消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatHistory {
    pub messages: Vec<ChatMessage>,
}

fn history_path(app: &tauri::AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("无法解析 app_data_dir: {}", e))?;
    Ok(dir.join("chat_history.json"))
}

/// 从磁盘加载；文件不存在时返回空历史。
pub fn load_history(app: &tauri::AppHandle) -> Result<ChatHistory> {
    let path = history_path(app)?;
    if !path.exists() {
        return Ok(ChatHistory::default());
    }
    let text = std::fs::read_to_string(&path).with_context(|| format!("读取 {}", path.display()))?;
    let history: ChatHistory =
        serde_json::from_str(&text).with_context(|| format!("解析 {}", path.display()))?;
    Ok(history)
}

impl ChatHistory {
    /// 写入磁盘（自动创建父目录）。
    pub fn save(&self, app: &tauri::AppHandle) -> Result<()> {
        let path = history_path(app)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| format!("创建目录 {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(self).context("序列化 chat_history")?;
        std::fs::write(&path, json).with_context(|| format!("写入 {}", path.display()))?;
        tracing::debug!("chat_history 已保存: {} 条", self.messages.len());
        Ok(())
    }
}

/// 删除历史文件（若存在）。
pub fn clear_history(app: &tauri::AppHandle) -> Result<()> {
    let path = history_path(app)?;
    if path.exists() {
        std::fs::remove_file(&path).with_context(|| format!("删除 {}", path.display()))?;
        tracing::info!("chat_history 已清空");
    }
    Ok(())
}
