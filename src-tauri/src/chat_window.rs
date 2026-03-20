//! 独立「对话」Webview 窗口（`label = chat`）。

use tauri::{AppHandle, Manager};

pub const CHAT_WINDOW_LABEL: &str = "chat";

pub fn show_chat_window(app: &AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window(CHAT_WINDOW_LABEL)
        .ok_or_else(|| "未找到对话窗口".to_string())?;
    w.center().map_err(|e| e.to_string())?;
    w.show().map_err(|e| e.to_string())?;
    let _ = w.set_focus();
    Ok(())
}

pub fn hide_chat_window(app: &AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window(CHAT_WINDOW_LABEL)
        .ok_or_else(|| "未找到对话窗口".to_string())?;
    w.hide().map_err(|e| e.to_string())
}
