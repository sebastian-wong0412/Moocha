//! 主窗口层级：默认不置顶；提醒警报时临时置顶；支持托盘手动固定置顶。

use std::sync::Mutex;

use tauri::{AppHandle, Manager};

use crate::tray::MAIN_WINDOW_LABEL;

/// 与前端/托盘共享的层级逻辑状态（线程安全由外层 `Mutex` 保证）。
#[derive(Debug, Default, Clone, Copy)]
pub struct WindowLayerState {
    /// 是否有需要用户处理的提醒气泡（应置顶以便看见）
    pub is_alert_mode: bool,
    /// 用户通过托盘勾选「置顶宠物」
    pub is_manually_pinned: bool,
}

impl WindowLayerState {
    /// 实际是否应 `always_on_top`
    #[inline]
    pub fn effective_always_on_top(self) -> bool {
        self.is_alert_mode || self.is_manually_pinned
    }

    /// 将当前逻辑状态应用到主窗口
    pub fn apply_to_main_window(self, app: &AppHandle) -> Result<(), String> {
        let w = app
            .get_webview_window(MAIN_WINDOW_LABEL)
            .ok_or_else(|| "未找到主窗口".to_string())?;
        w.set_always_on_top(self.effective_always_on_top())
            .map_err(|e| e.to_string())
    }

    /// 设置警报模式（提醒展示中）并刷新窗口层级
    pub fn set_alert_mode(&mut self, app: &AppHandle, enabled: bool) -> Result<(), String> {
        self.is_alert_mode = enabled;
        self.apply_to_main_window(app)
    }

    /// 用户手动置顶开关
    pub fn set_pinned(&mut self, app: &AppHandle, pinned: bool) -> Result<(), String> {
        self.is_manually_pinned = pinned;
        self.apply_to_main_window(app)
    }

    /// 恢复为默认：非警报、非手动置顶（预留：托盘「恢复默认层级」等）
    #[allow(dead_code)]
    pub fn reset_layer(&mut self, app: &AppHandle) -> Result<(), String> {
        *self = WindowLayerState::default();
        self.apply_to_main_window(app)
    }
}

pub fn set_alert_mode(layer: &Mutex<WindowLayerState>, app: &AppHandle, enabled: bool) -> Result<(), String> {
    let mut g = layer
        .lock()
        .map_err(|e| format!("层级状态锁异常: {}", e))?;
    g.set_alert_mode(app, enabled)
}

pub fn set_manually_pinned(
    layer: &Mutex<WindowLayerState>,
    app: &AppHandle,
    pinned: bool,
) -> Result<(), String> {
    let mut g = layer
        .lock()
        .map_err(|e| format!("层级状态锁异常: {}", e))?;
    g.set_pinned(app, pinned)
}

/// 仅结束警报置顶（不影响手动置顶）
pub fn clear_alert_mode(layer: &Mutex<WindowLayerState>, app: &AppHandle) -> Result<(), String> {
    let mut g = layer
        .lock()
        .map_err(|e| format!("层级状态锁异常: {}", e))?;
    g.set_alert_mode(app, false)
}

#[allow(dead_code)]
pub fn reset_layer(layer: &Mutex<WindowLayerState>, app: &AppHandle) -> Result<(), String> {
    let mut g = layer
        .lock()
        .map_err(|e| format!("层级状态锁异常: {}", e))?;
    g.reset_layer(app)
}
