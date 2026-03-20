//! 独立「设置」Webview 窗口（`label = settings`），与宠物主窗口分离。

use tauri::{AppHandle, Manager};

pub const SETTINGS_WINDOW_LABEL: &str = "settings";

/// 显示设置窗口并居中、聚焦。
pub fn show_settings_window(app: &AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window(SETTINGS_WINDOW_LABEL)
        .ok_or_else(|| "未找到设置窗口".to_string())?;
    w.center().map_err(|e| e.to_string())?;
    w.show().map_err(|e| e.to_string())?;
    let _ = w.set_focus();
    Ok(())
}

/// 隐藏设置窗口（不退出应用）。
pub fn hide_settings_window(app: &AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window(SETTINGS_WINDOW_LABEL)
        .ok_or_else(|| "未找到设置窗口".to_string())?;
    w.hide().map_err(|e| e.to_string())
}
