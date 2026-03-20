//! 定时提醒：整点、连续工作休息、久坐强提醒；与 `ReminderConfig` 配合。

use crate::context_rules::{is_work_app, ReminderConfig};
use chrono::{DateTime, Duration, Local, NaiveDate, Timelike};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Manager;

/// 发往窗口 `pet-reminder` 的载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetReminderPayload {
    pub kind: String,
    pub message: String,
}

/// 运行时状态：连续工作计时、各提醒去重
#[derive(Debug, Default)]
pub struct ReminderManager {
    /// 当前连续工作段开始时间（仅当上一 tick 仍为工作应用时有效）
    work_start: Option<DateTime<Local>>,
    /// 已播报过的整点槽位 (日期, 小时 0–23)
    last_hourly_slot: Option<(NaiveDate, u32)>,
    last_break_emit: Option<DateTime<Local>>,
    last_long_work_emit: Option<DateTime<Local>>,
}

impl ReminderManager {
    /// 用户确认后：从当前时刻重新累计连续工作时长
    pub fn reset_work_timer(&mut self) {
        self.work_start = Some(Local::now());
        self.last_break_emit = None;
        self.last_long_work_emit = None;
    }

    pub fn acknowledge(&mut self, kind: &str) {
        match kind {
            "break" | "long_work" => self.reset_work_timer(),
            _ => {}
        }
    }

    /// 是否应在「休息」维度再次播报（冷却）
    fn allow_break_emit(&mut self, now: DateTime<Local>) -> bool {
        const COOLDOWN: i64 = 25; // 分钟
        if let Some(t) = self.last_break_emit {
            if now.signed_duration_since(t) < Duration::minutes(COOLDOWN) {
                return false;
            }
        }
        self.last_break_emit = Some(now);
        true
    }

    fn allow_long_emit(&mut self, now: DateTime<Local>) -> bool {
        const COOLDOWN: i64 = 35;
        if let Some(t) = self.last_long_work_emit {
            if now.signed_duration_since(t) < Duration::minutes(COOLDOWN) {
                return false;
            }
        }
        self.last_long_work_emit = Some(now);
        true
    }

    /// 单次轮询内可能产生 0~多条提醒（整点 + 久坐/休息互斥优先级：久坐优先）
    pub fn process_tick(
        &mut self,
        active_app: &str,
        config: &ReminderConfig,
        now: DateTime<Local>,
    ) -> Vec<PetReminderPayload> {
        let mut out = Vec::new();

        // ── 整点：进入新小时后前 5 分钟内首次 tick 播报一次
        if config.enable_hourly {
            let slot = (now.date_naive(), now.hour() as u32);
            if now.minute() < 5 && self.last_hourly_slot != Some(slot) {
                self.last_hourly_slot = Some(slot);
                out.push(PetReminderPayload {
                    kind: "hourly".into(),
                    message: format!("现在是 {} 点啦！", now.hour()),
                });
            }
        }

        let working = is_work_app(active_app);

        if !working {
            self.work_start = None;
            return out;
        }

        let start = *self.work_start.get_or_insert(now);
        let elapsed = now.signed_duration_since(start);
        if elapsed < Duration::zero() {
            self.work_start = Some(now);
            return out;
        }

        let long_mins = config.long_work_minutes.max(1) as i64;
        let break_mins = config.break_interval_minutes.max(1) as i64;

        let mut long_fired_this_tick = false;

        if config.enable_long_work && elapsed >= Duration::minutes(long_mins) {
            if self.allow_long_emit(now) {
                out.push(PetReminderPayload {
                    kind: "long_work".into(),
                    message: format!(
                        "已经工作 {} 分钟了，起来活动活动！",
                        long_mins
                    ),
                });
                long_fired_this_tick = true;
            }
        }

        if !long_fired_this_tick && config.enable_break && elapsed >= Duration::minutes(break_mins) {
            if self.allow_break_emit(now) {
                out.push(PetReminderPayload {
                    kind: "break".into(),
                    message: "休息一下吧~".into(),
                });
            }
        }

        out
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 持久化 + Tauri 托管
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct ReminderBundle {
    pub config: ReminderConfig,
    pub manager: ReminderManager,
}

impl Default for ReminderBundle {
    fn default() -> Self {
        Self {
            config: ReminderConfig::default(),
            manager: ReminderManager::default(),
        }
    }
}

impl ReminderBundle {
    fn path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("app_data_dir: {}", e))?;
        Ok(dir.join("reminder_config.json"))
    }

    pub fn load(app: &tauri::AppHandle) -> Result<Self, String> {
        let path = Self::path(app)?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("读取提醒配置: {}", e))?;
        let config: ReminderConfig =
            serde_json::from_str(&text).map_err(|e| format!("解析提醒配置: {}", e))?;
        Ok(Self {
            config,
            manager: ReminderManager::default(),
        })
    }

    pub fn save_disk(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let path = Self::path(app)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录: {}", e))?;
        }
        let json = serde_json::to_string_pretty(&self.config)
            .map_err(|e| format!("序列化: {}", e))?;
        std::fs::write(&path, json).map_err(|e| format!("写入: {}", e))?;
        Ok(())
    }
}

/// 供 `trigger_break_reminder` 等构造默认文案
pub fn manual_break_payload() -> PetReminderPayload {
    PetReminderPayload {
        kind: "break".into(),
        message: "休息一下吧~".into(),
    }
}
