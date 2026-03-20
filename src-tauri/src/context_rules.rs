//! 情境规则引擎：根据前台应用、时段、空闲时间生成宠物互动行动。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ═══════════════════════════════════════════════════════════════════════════
// 数据结构（与前端约定）
// ═══════════════════════════════════════════════════════════════════════════

/// 休息 / 整点 / 久坐提醒开关与间隔（持久化到 `reminder_config.json`）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderConfig {
    /// 整点报时
    pub enable_hourly: bool,
    /// 连续工作达到间隔后的休息提醒
    pub enable_break: bool,
    /// 连续工作达到「久坐」阈值后的强提醒
    pub enable_long_work: bool,
    /// 休息提醒间隔（分钟），默认 60
    pub break_interval_minutes: u64,
    /// 久坐阈值（分钟），默认 120
    pub long_work_minutes: u64,
}

impl Default for ReminderConfig {
    fn default() -> Self {
        Self {
            enable_hourly: true,
            enable_break: true,
            enable_long_work: true,
            break_interval_minutes: 60,
            long_work_minutes: 120,
        }
    }
}

/// 单条触发的互动行动（发往前端 `context-action`）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAction {
    pub rule: String,
    pub message: String,
    pub mood: String,
}

/// 规则描述（供扩展 / `add_rule` 占位）
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ContextRule {
    pub name: String,
    pub condition: String,
    pub action: String,
}

/// 规则引擎（当前以内置规则为主，`custom` 预留）
pub struct RuleEngine {
    pub custom: Vec<ContextRule>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self { custom: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn add_rule(&mut self, rule: ContextRule) {
        self.custom.push(rule);
    }

    /// 与全局 `check_rules` 一致：内置匹配 + 将来可合并 `self.custom`
    pub fn check_rules(&self, app: &str, time: &str, idle: u64) -> Vec<ContextAction> {
        let out = evaluate_builtin(app, time, idle);
        // v0.6：custom 规则仅占位，不做字符串 DSL 解析
        let _ = &self.custom;
        out
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// 无状态检查：返回所有**当前满足条件**的内置规则（不做去重）
pub fn check_rules(app: &str, time: &str, idle: u64) -> Vec<ContextAction> {
    RuleEngine::default().check_rules(app, time, idle)
}

/// 某条规则再次触发的最短间隔（秒）
pub fn cooldown_for_rule(rule: &str) -> Duration {
    Duration::from_secs(match rule {
        "morning_greet" | "night_rest" => 6 * 3600,
        "idle_stretch" => 10 * 60,
        _ => 10 * 60, // 应用类
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// 去重（同一 rule 在冷却内不重复 emit）
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Default)]
pub struct RuleDeduper {
    last_fired: HashMap<String, Instant>,
}

impl RuleDeduper {
    /// 若距上次触发已超过冷却，则记一笔并返回 true；否则 false。
    pub fn try_fire(&mut self, rule: &str, cooldown: Duration) -> bool {
        let now = Instant::now();
        if let Some(prev) = self.last_fired.get(rule) {
            if now.duration_since(*prev) < cooldown {
                return false;
            }
        }
        self.last_fired.insert(rule.to_string(), now);
        true
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 内置规则
// ═══════════════════════════════════════════════════════════════════════════

fn app_lower(app: &str) -> String {
    app.to_lowercase()
}

/// 是否视为「工作类」应用（IDE 等），供久坐 / 休息计时使用
pub fn is_work_app(app: &str) -> bool {
    matches_work(&app_lower(app))
}

fn matches_work(a: &str) -> bool {
    [
        "code", "idea", "webstorm", "pycharm", "intellij", "devenv", "rider", "goland",
        "clion", "rustrover", "studio64", "androidstudio",
    ]
    .iter()
    .any(|k| a.contains(k))
}

fn matches_browser(a: &str) -> bool {
    [
        "chrome", "firefox", "msedge", "brave", "opera", "vivaldi", "safari", "edge",
    ]
    .iter()
    .any(|k| a.contains(k))
}

fn matches_social(a: &str) -> bool {
    ["wechat", "weixin", "qq", "discord", "slack", "teams", "telegram"].iter().any(|k| a.contains(k))
}

fn evaluate_builtin(app: &str, time: &str, idle: u64) -> Vec<ContextAction> {
    let a = app_lower(app);
    let mut out = Vec::new();

    // 工作类优先于浏览器（避免 Electron 误判）
    if matches_work(&a) {
        out.push(ContextAction {
            rule: "work_mode".into(),
            message: "加油写代码！".into(),
            mood: "excited".into(),
        });
    } else if matches_browser(&a) {
        out.push(ContextAction {
            rule: "browse_mode".into(),
            message: "在查什么资料？".into(),
            mood: "curious".into(),
        });
    }

    if matches_social(&a) {
        out.push(ContextAction {
            rule: "social_mode".into(),
            message: "在聊天呀~".into(),
            mood: "happy".into(),
        });
    }

    if idle > 300 {
        out.push(ContextAction {
            rule: "idle_stretch".into(),
            message: "该活动一下了".into(),
            mood: "sleepy".into(),
        });
    }

    if time == "morning" {
        out.push(ContextAction {
            rule: "morning_greet".into(),
            message: "早上好！".into(),
            mood: "happy".into(),
        });
    }

    if time == "night" {
        out.push(ContextAction {
            rule: "night_rest".into(),
            message: "该休息了".into(),
            mood: "sleepy".into(),
        });
    }

    out
}
