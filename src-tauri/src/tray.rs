//! 系统托盘：菜单、左键显示/隐藏主窗口；与任务栏隐藏配合使用。

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager};
use std::sync::Mutex;

use crate::chat_window;
use crate::settings_window;
use crate::window_layer::{self, WindowLayerState};

/// 与 `tauri.conf.json` 中主窗口 `label` 一致
pub const MAIN_WINDOW_LABEL: &str = "main";

const MENU_SHOW: &str = "tray_show";
const MENU_HIDE: &str = "tray_hide";
const MENU_PIN: &str = "tray_pin";
const MENU_CHAT: &str = "tray_chat";
const MENU_SETTINGS: &str = "tray_settings";
const MENU_QUIT: &str = "tray_quit";

/// 显示宠物主窗口并尝试聚焦（供托盘 / 命令调用）。
pub fn show_pet_window(app: &AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| "未找到主窗口".to_string())?;
    w.show().map_err(|e| e.to_string())?;
    let _ = w.set_focus();
    Ok(())
}

/// 隐藏宠物主窗口。
pub fn hide_pet_window(app: &AppHandle) -> Result<(), String> {
    let w = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .ok_or_else(|| "未找到主窗口".to_string())?;
    w.hide().map_err(|e| e.to_string())?;
    Ok(())
}

/// 左键：当前可见则隐藏，否则显示。
fn toggle_pet_window(app: &AppHandle) {
    let Some(w) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        tracing::warn!("托盘切换：无主窗口");
        return;
    };
    let visible = w.is_visible().unwrap_or(false);
    let res = if visible {
        w.hide()
    } else {
        w.show().and_then(|_| w.set_focus())
    };
    if let Err(e) = res {
        tracing::warn!("托盘切换窗口可见性失败: {}", e);
    }
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    let r = match id {
        MENU_SHOW => show_pet_window(app),
        MENU_HIDE => hide_pet_window(app),
        MENU_CHAT => chat_window::show_chat_window(app),
        MENU_SETTINGS => settings_window::show_settings_window(app),
        MENU_QUIT => {
            app.exit(0);
            return;
        }
        _ => Ok(()),
    };
    if let Err(e) = r {
        tracing::warn!("托盘菜单操作失败: {}", e);
    }
}

/// 创建托盘图标与菜单；在 `setup` 中调用。
pub fn create_tray(app: &App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, MENU_SHOW, "显示宠物", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, MENU_HIDE, "隐藏宠物", true, None::<&str>)?;

    let initial_pinned = app
        .state::<Mutex<WindowLayerState>>()
        .lock()
        .map(|g| g.is_manually_pinned)
        .unwrap_or(false);
    let pin = CheckMenuItem::with_id(
        app,
        MENU_PIN,
        "置顶宠物",
        true,
        initial_pinned,
        None::<&str>,
    )?;
    let pin_for_event = pin.clone();

    let chat = MenuItem::with_id(app, MENU_CHAT, "Moocha 对话", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, MENU_SETTINGS, "Moocha 设置", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "退出", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show,
            &hide,
            &PredefinedMenuItem::separator(app)?,
            &pin,
            &PredefinedMenuItem::separator(app)?,
            &chat,
            &settings,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    let icon = Image::from_bytes(include_bytes!("../icons/32x32.png"))?;

    let app_handle = app.handle().clone();

    TrayIconBuilder::with_id("moocha_tray")
        .icon(icon)
        .menu(&menu)
        .tooltip("Moocha")
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            let id = event.id.as_ref();
            if id == MENU_PIN {
                let layer = app.state::<Mutex<WindowLayerState>>();
                let new_pinned = match layer.lock() {
                    Ok(g) => !g.is_manually_pinned,
                    Err(_) => false,
                };
                if let Err(e) = window_layer::set_manually_pinned(&layer, app, new_pinned) {
                    tracing::warn!("置顶切换失败: {}", e);
                } else if let Err(e) = pin_for_event.set_checked(new_pinned) {
                    tracing::warn!("同步托盘勾选状态失败: {}", e);
                }
                return;
            }
            handle_menu_event(app, id);
        })
        .on_tray_icon_event(move |_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_pet_window(&app_handle);
            }
        })
        .build(app)?;

    Ok(())
}
