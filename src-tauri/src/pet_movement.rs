//! 宠物窗口底部巡逻：工作区内随机横移、平滑过渡、随机休息。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use rand::Rng;
use serde::{Deserialize, Serialize};
use tauri::webview::WebviewWindow;
use tauri::{AppHandle, Manager, PhysicalPosition};

use crate::tray::MAIN_WINDOW_LABEL;

const BOTTOM_MARGIN: i32 = 10;
const MIN_MOVE_DELTA: i32 = 48;

/// 与前端 `invoke` 对齐（camelCase）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenInfo {
    pub width: u32,
    pub height: u32,
    pub work_area_x: i32,
    pub work_area_y: i32,
    pub work_area_width: u32,
    pub work_area_height: u32,
    /// 粗略：显示器高度 − 工作区高度（用于兼容旧接口）
    pub taskbar_height: i32,
}

#[derive(Debug, Clone)]
pub struct MovementConfig {
    pub move_wait_min_ms: u64,
    pub move_wait_max_ms: u64,
    pub rest_min_ms: u64,
    pub rest_max_ms: u64,
    pub movement_speed_ms: u64,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            move_wait_min_ms: 3000,
            move_wait_max_ms: 8000,
            rest_min_ms: 10_000,
            rest_max_ms: 30_000,
            movement_speed_ms: 1000,
        }
    }
}

/// 巡逻开关（由 `start_pet_patrol` / `stop_pet_patrol` 控制）
#[derive(Clone)]
pub struct PetPatrolState {
    pub enabled: Arc<AtomicBool>,
}

impl Default for PetPatrolState {
    fn default() -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PetMovementState {
    pub current_x: i32,
    pub target_x: i32,
    pub is_moving: bool,
    pub is_resting: bool,
    pub last_move_time: u64,
}

/// 右下角（工作区内），留出边距
pub fn get_initial_position(
    work_x: i32,
    work_y: i32,
    work_w: u32,
    work_h: u32,
    window_w: i32,
    window_h: i32,
) -> (i32, i32) {
    let x = work_x + work_w as i32 - window_w - BOTTOM_MARGIN;
    let y = work_y + work_h as i32 - window_h - BOTTOM_MARGIN;
    let x = x.max(work_x);
    (x, y)
}

/// 在工作区底部范围内随机下一个 X
pub fn get_next_x(current_x: i32, work_x: i32, work_w: u32, window_w: i32) -> i32 {
    let min_x = work_x;
    let max_x = work_x + work_w as i32 - window_w;
    if max_x <= min_x {
        return min_x;
    }
    let mut rng = rand::thread_rng();
    for _ in 0..12 {
        let x = rng.gen_range(min_x..=max_x);
        if (x - current_x).abs() >= MIN_MOVE_DELTA {
            return x;
        }
    }
    rng.gen_range(min_x..=max_x)
}

pub fn screen_info(win: &WebviewWindow) -> Result<ScreenInfo, String> {
    let m = win
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "无当前显示器".to_string())?;
    let sz = m.size();
    let wa = m.work_area();
    let taskbar_h = (sz.height as i32).saturating_sub(wa.size.height as i32);
    Ok(ScreenInfo {
        width: sz.width,
        height: sz.height,
        work_area_x: wa.position.x,
        work_area_y: wa.position.y,
        work_area_width: wa.size.width,
        work_area_height: wa.size.height,
        taskbar_height: taskbar_h.max(0),
    })
}

/// 在主线程执行与主窗口相关的逻辑，并通过 channel 取回结果（避免在非主线程直接访问窗口）。
fn with_main_window<T, F>(app: &AppHandle, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(WebviewWindow) -> Result<T, String> + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let h = app.clone();
    let scheduler = h.clone();
    scheduler
        .run_on_main_thread(move || {
            let res = match h.get_webview_window(MAIN_WINDOW_LABEL) {
                Some(w) => f(w),
                None => Err("无主窗口".into()),
            };
            let _ = tx.send(res);
        })
        .map_err(|e| e.to_string())?;
    rx.recv()
        .map_err(|_| "主线程任务未返回".to_string())?
}

/// 将主窗口一次性贴到当前显示器工作区右下角（供启动 `setup` 与巡逻循环兜底调用）。
pub fn snap_pet_window_bottom_right(win: &WebviewWindow) -> Result<(), String> {
    let sz = win.outer_size().map_err(|e| e.to_string())?;
    let m = win
        .current_monitor()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "无显示器".to_string())?;
    let wa = m.work_area();
    let win_w = sz.width as i32;
    let win_h = sz.height as i32;
    let (x, y) = get_initial_position(
        wa.position.x,
        wa.position.y,
        wa.size.width,
        wa.size.height,
        win_w,
        win_h,
    );
    win
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|e| e.to_string())
}

fn snapshot_move_plan(app: &AppHandle) -> Result<(i32, i32, i32), String> {
    with_main_window(app, |w| {
        let pos = w.outer_position().map_err(|e| e.to_string())?;
        let sz = w.outer_size().map_err(|e| e.to_string())?;
        let m = w
            .current_monitor()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "无显示器".to_string())?;
        let wa = m.work_area();
        let win_w = sz.width as i32;
        let win_h = sz.height as i32;
        let y = wa.position.y + wa.size.height as i32 - win_h - BOTTOM_MARGIN;
        let target_x = get_next_x(pos.x, wa.position.x, wa.size.width, win_w);
        Ok((pos.x, y, target_x))
    })
}

fn set_position_main(app: &AppHandle, x: i32, y: i32) -> Result<(), String> {
    with_main_window(app, move |w| {
        w.set_position(PhysicalPosition::new(x, y))
            .map_err(|e| e.to_string())
    })
}

/// 供 `set_pet_position` 命令调用
pub fn set_pet_window_position(app: &AppHandle, x: i32, y: i32) -> Result<(), String> {
    set_position_main(app, x, y)
}

/// 水平平滑移动（smoothstep 插值）
async fn smooth_move_x(app: &AppHandle, from_x: i32, to_x: i32, y: i32, duration_ms: u64) {
    let steps: u32 = 24;
    let step_ms = (duration_ms / steps as u64).max(1);
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let e = t * t * (3.0 - 2.0 * t);
        let x = from_x + (((to_x - from_x) as f32) * e).round() as i32;
        if let Err(e) = set_position_main(app, x, y) {
            tracing::debug!("巡逻 set_position: {}", e);
            break;
        }
        tokio::time::sleep(Duration::from_millis(step_ms)).await;
    }
}

/// 后台巡逻循环（由 `setup` 里 `spawn`）
pub async fn run_patrol_loop(app: AppHandle, enabled: Arc<AtomicBool>) {
    let cfg = MovementConfig::default();

    // 启动落位仅在 `setup` 中同步完成，此处不再延迟或重复落位，避免首帧瞬移

    loop {
        if !enabled.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(500)).await;
            continue;
        }

        let visible = with_main_window(&app, |w| w.is_visible().map_err(|e| e.to_string()))
            .unwrap_or(false);
        if !visible {
            tokio::time::sleep(Duration::from_millis(900)).await;
            continue;
        }

        let wait_ms = rand::thread_rng().gen_range(cfg.move_wait_min_ms..=cfg.move_wait_max_ms);
        tokio::time::sleep(Duration::from_millis(wait_ms)).await;
        if !enabled.load(Ordering::SeqCst) {
            continue;
        }

        let (from_x, y, target_x) = match snapshot_move_plan(&app) {
            Ok(v) => v,
            Err(e) => {
                tracing::debug!("巡逻计划失败: {}", e);
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        if (target_x - from_x).abs() < 8 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            continue;
        }

        smooth_move_x(&app, from_x, target_x, y, cfg.movement_speed_ms).await;

        let rest_ms = rand::thread_rng().gen_range(cfg.rest_min_ms..=cfg.rest_max_ms);
        tokio::time::sleep(Duration::from_millis(rest_ms)).await;
    }
}
