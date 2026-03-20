mod ai;
mod chat_history;
mod config;
mod context_rules;
mod reminder;
mod state;
mod system_monitor;

use ai::ContextMessage;
use chrono::Local;
use config::AppConfig;
use context_rules::ReminderConfig;
use state::{new_shared_state, PetState, SharedState};
use std::sync::Mutex;
use tauri::{Emitter, Manager};

// ── Commands ──────────────────────────────────────────────────────────────────

/// 返回宠物当前状态字符串，供前端确认后端连接正常。
#[tauri::command]
async fn get_pet_status(state: tauri::State<'_, SharedState>) -> Result<String, String> {
    let s = state.read().await;
    let status = format!(
        "Moocha v{} | mood: {} | ready: {}",
        env!("CARGO_PKG_VERSION"),
        s.pet_state,
        s.is_ready,
    );
    tracing::debug!("get_pet_status -> {}", status);
    Ok(status)
}

/// 返回当前应用配置。
/// 若 API Key 已设置，返回值中以 `"****"` 占位，避免明文暴露给前端渲染层。
#[tauri::command]
async fn get_config(state: tauri::State<'_, SharedState>) -> Result<AppConfig, String> {
    let s = state.read().await;
    let config = s
        .config
        .lock()
        .map_err(|e| format!("配置锁异常: {}", e))?;

    let mut exposed = config.clone();
    if !exposed.api_key.is_empty() {
        exposed.api_key = "****".to_string();
    }
    Ok(exposed)
}

/// 接收前端提交的新配置，持久化后同步更新运行时状态。
///
/// 若前端回传的 `api_key` 为空字符串或占位符 `"****"`，
/// 则保留当前内存中的密钥，避免误清空。
#[tauri::command]
async fn save_config(
    state: tauri::State<'_, SharedState>,
    mut config: AppConfig,
) -> Result<(), String> {
    let s = state.read().await;

    {
        let current = s
            .config
            .lock()
            .map_err(|e| format!("配置锁异常: {}", e))?;

        // 占位符或空值 → 保留已有密钥
        if config.api_key.is_empty() || config.api_key == "****" {
            config.api_key = current.api_key.clone();
        }
    } // MutexGuard 在此处释放，后续 I/O 不持有锁

    // 写盘（api_key 在 save() 内部被置空，不落地）
    config
        .save()
        .map_err(|e| format!("保存配置失败: {}", e))?;

    // 更新内存状态
    let mut current = s
        .config
        .lock()
        .map_err(|e| format!("配置锁异常: {}", e))?;
    *current = config;

    tracing::info!("运行时配置已更新");
    Ok(())
}

/// 测试与 AI 服务的网络连通性（使用已保存的配置）。
#[tauri::command]
async fn test_connection(state: tauri::State<'_, SharedState>) -> Result<bool, String> {
    let (base_url, api_key) = {
        let s = state.read().await;
        let config = s
            .config
            .lock()
            .map_err(|e| format!("配置锁异常: {}", e))?;
        (config.base_url.clone(), config.api_key.clone())
    };
    probe_connection(&base_url, &api_key).await
}

/// 用前端传入的参数临时测试连通性，**不读写任何持久化状态**。
///
/// 供"填完表单但尚未保存时"使用，使测试与保存完全解耦。
/// `api_key` 为空时不携带 Authorization 头（适用于无鉴权的本地服务）。
#[tauri::command]
async fn test_connection_with(base_url: String, api_key: String) -> Result<bool, String> {
    probe_connection(&base_url, &api_key).await
}

/// 非流式对话：发送一条消息，等待 AI 返回完整回复。
///
/// `context` 为可选历史上下文（`[{role, content}, ...]` 形式的 JSON 数组字符串，
/// 为空时传空数组）。
#[tauri::command]
async fn chat(
    state: tauri::State<'_, SharedState>,
    message: String,
    context: Vec<serde_json::Value>,
) -> Result<String, String> {
    let config = {
        let s = state.read().await;
        let c = s.config.lock().map_err(|e| format!("配置锁异常: {}", e))?;
        c.clone()
    };

    // 将前端传入的 JSON 数组转换为 ContextMessage
    let ctx: Vec<ContextMessage> = context
        .into_iter()
        .filter_map(|v| {
            let role = v["role"].as_str()?.to_string();
            let content = v["content"].as_str()?.to_string();
            Some(ContextMessage { role, content })
        })
        .collect();

    let provider = ai::create_provider(&config);

    tracing::info!("chat: provider={}, message_len={}", provider.name(), message.len());

    provider
        .chat(&message, &ctx)
        .await
        .map_err(|e| e.to_string())
}

/// 流式对话：发送一条消息，通过 Tauri 事件逐块推送回复。
///
/// 事件：
///   - `chat-chunk`  每个文本块（`String`）
///   - `chat-done`   流结束（`""`）
///   - `chat-error`  错误信息（`String`）
///
/// 本命令立即返回，流在后台任务中运行。
#[tauri::command]
async fn chat_stream(
    state: tauri::State<'_, SharedState>,
    app: tauri::AppHandle,
    message: String,
    context: Vec<serde_json::Value>,
) -> Result<(), String> {
    let config = {
        let s = state.read().await;
        let c = s.config.lock().map_err(|e| format!("配置锁异常: {}", e))?;
        c.clone()
    };

    let ctx: Vec<ContextMessage> = context
        .into_iter()
        .filter_map(|v| {
            let role = v["role"].as_str()?.to_string();
            let content = v["content"].as_str()?.to_string();
            Some(ContextMessage { role, content })
        })
        .collect();

    let provider = ai::create_provider(&config);

    tracing::info!("chat_stream: provider={}", provider.name());

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // 后台任务：Provider 产生数据块 → channel → 转发为 Tauri 事件
    let app_bg = app.clone();
    tauri::async_runtime::spawn(async move {
        // 在独立任务中运行 Provider 流，使其与 relay 解耦
        let stream_handle = tauri::async_runtime::spawn(async move {
            provider.chat_stream(&message, &ctx, tx).await
        });

        // 将 channel 中的块逐一 emit 给前端
        while let Some(chunk) = rx.recv().await {
            if app_bg.emit("chat-chunk", &chunk).is_err() {
                tracing::warn!("chat-chunk emit 失败，前端可能已关闭");
                break;
            }
        }

        // Provider 任务结束后发送完成/错误事件
        match stream_handle.await {
            Ok(Ok(())) => {
                let _ = app_bg.emit("chat-done", "");
            }
            Ok(Err(e)) => {
                tracing::error!("chat_stream provider 错误: {}", e);
                let _ = app_bg.emit("chat-error", e.to_string());
            }
            Err(e) => {
                tracing::error!("chat_stream 任务 panic: {}", e);
                let _ = app_bg.emit("chat-error", format!("内部错误: {}", e));
            }
        }
    });

    Ok(())
}

// ── 对话历史持久化 ────────────────────────────────────────────────────────────

/// 返回已保存的全部消息（文件不存在则为空数组）。
#[tauri::command]
async fn get_chat_history(app: tauri::AppHandle) -> Result<Vec<chat_history::ChatMessage>, String> {
    chat_history::load_history(&app)
        .map(|h| h.messages)
        .map_err(|e| e.to_string())
}

/// 追加一条消息并写盘。
#[tauri::command]
async fn save_chat_message(
    app: tauri::AppHandle,
    message: chat_history::ChatMessage,
) -> Result<(), String> {
    let mut history = chat_history::load_history(&app).map_err(|e| e.to_string())?;
    history.messages.push(message);
    history.save(&app).map_err(|e| e.to_string())
}

/// 清空历史文件。
#[tauri::command]
async fn clear_chat_history(app: tauri::AppHandle) -> Result<(), String> {
    chat_history::clear_history(&app).map_err(|e| e.to_string())
}

// ── 系统监控（v0.5）──────────────────────────────────────────────────────────

/// 当前前台窗口标题（无法识别时为 `"Unknown"`）。
#[tauri::command]
async fn get_active_window() -> Result<String, String> {
    tokio::task::spawn_blocking(system_monitor::get_active_window)
        .await
        .map_err(|e| format!("任务异常: {}", e))?
}

/// 当前前台应用名称（无法识别时为 `"Unknown"`）。
#[tauri::command]
async fn get_active_app() -> Result<String, String> {
    tokio::task::spawn_blocking(system_monitor::get_active_app)
        .await
        .map_err(|e| format!("任务异常: {}", e))?
}

/// 本地时段：`morning` / `afternoon` / `evening` / `night`
#[tauri::command]
async fn get_system_time() -> Result<String, String> {
    system_monitor::get_system_time()
}

/// 用户空闲时长（秒）；不支持的平台为 `0`。
#[tauri::command]
async fn get_idle_duration() -> Result<u64, String> {
    tokio::task::spawn_blocking(system_monitor::get_idle_duration)
        .await
        .map_err(|e| format!("任务异常: {}", e))?
}

/// 全局 CPU 使用率（0.0–100.0）。
#[tauri::command]
async fn get_cpu_usage() -> Result<f32, String> {
    tokio::task::spawn_blocking(system_monitor::get_cpu_usage)
        .await
        .map_err(|e| format!("任务异常: {}", e))?
}

/// 已用物理内存（字节）。
#[tauri::command]
async fn get_memory_usage() -> Result<u64, String> {
    tokio::task::spawn_blocking(system_monitor::get_memory_usage)
        .await
        .map_err(|e| format!("任务异常: {}", e))?
}

/// 手动拉取当前情境下会触发的行动（**不经过**去重冷却，便于调试）。
#[tauri::command]
async fn check_context() -> Result<Vec<context_rules::ContextAction>, String> {
    tokio::task::spawn_blocking(|| {
        let app = system_monitor::get_active_app().unwrap_or_else(|_| "Unknown".into());
        let time = system_monitor::get_system_time().unwrap_or_else(|_| "night".into());
        let idle = system_monitor::get_idle_duration().unwrap_or(0);
        context_rules::check_rules(&app, &time, idle)
    })
    .await
    .map_err(|e| format!("任务异常: {}", e))
}

/// 读取休息 / 整点 / 久坐提醒配置
#[tauri::command]
fn get_reminder_config(state: tauri::State<'_, Mutex<reminder::ReminderBundle>>) -> Result<ReminderConfig, String> {
    state
        .lock()
        .map(|b| b.config.clone())
        .map_err(|e| format!("锁异常: {}", e))
}

/// 更新提醒配置并写盘
#[tauri::command]
fn update_reminder_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<reminder::ReminderBundle>>,
    config: ReminderConfig,
) -> Result<(), String> {
    let mut g = state.lock().map_err(|e| format!("锁异常: {}", e))?;
    g.config = config;
    g.save_disk(&app)?;
    Ok(())
}

/// 用户在前端确认已读某类提醒（休息/久坐会重置连续工作计时）
#[tauri::command]
fn acknowledge_reminder(
    state: tauri::State<'_, Mutex<reminder::ReminderBundle>>,
    kind: String,
) -> Result<(), String> {
    let mut g = state.lock().map_err(|e| format!("锁异常: {}", e))?;
    g.manager.acknowledge(&kind);
    Ok(())
}

/// 手动触发一次「休息」提醒（测试用）
#[tauri::command]
fn trigger_break_reminder(app: tauri::AppHandle) -> Result<(), String> {
    let batch = vec![reminder::manual_break_payload()];
    app.emit("pet-reminder", &batch)
        .map_err(|e| format!("emit 失败: {}", e))
}

// ── 内部辅助 ──────────────────────────────────────────────────────────────────

/// 向 `{base_url}/models` 发送 GET 请求（8 秒超时）。
///
/// - HTTP 1xx–4xx → 服务可达，返回 `true`（4xx 通常是 Key 问题而非网络问题）
/// - HTTP 5xx 或连接失败 → 返回 `false`
async fn probe_connection(base_url: &str, api_key: &str) -> Result<bool, String> {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    tracing::info!("测试连接: {}", url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| format!("HTTP 客户端构建失败: {}", e))?;

    let mut req = client.get(&url);
    if !api_key.is_empty() {
        req = req.bearer_auth(api_key);
    }

    match req.send().await {
        Ok(resp) => {
            let reachable = resp.status().as_u16() < 500;
            tracing::info!(
                "连接测试完成: reachable={}, HTTP {}",
                reachable,
                resp.status()
            );
            Ok(reachable)
        }
        Err(e) => {
            tracing::warn!("连接测试失败: {}", e);
            Ok(false)
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化结构化日志，优先读取 RUST_LOG 环境变量
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Moocha 启动中...");

    // 从本地文件加载配置（含环境变量 API Key 注入）
    let config = AppConfig::load();
    let shared_state = new_shared_state(config);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(shared_state)
        .invoke_handler(tauri::generate_handler![
            get_pet_status,
            get_config,
            save_config,
            test_connection,
            test_connection_with,
            chat,
            chat_stream,
            get_chat_history,
            save_chat_message,
            clear_chat_history,
            get_active_window,
            get_active_app,
            get_system_time,
            get_idle_duration,
            get_cpu_usage,
            get_memory_usage,
            check_context,
            get_reminder_config,
            update_reminder_config,
            acknowledge_reminder,
            trigger_break_reminder,
        ])
        .manage(Mutex::new(context_rules::RuleDeduper::default()))
        .setup(|app| {
            let h = app.handle().clone();
            let bundle = reminder::ReminderBundle::load(&h).unwrap_or_default();
            if !app.manage(Mutex::new(bundle)) {
                tracing::error!("注册 ReminderBundle 失败（类型已存在？）");
            }

            let state: tauri::State<'_, SharedState> = app.state();
            let state_clone = state.inner().clone();
            tauri::async_runtime::spawn(async move {
                let mut s = state_clone.write().await;
                s.is_ready = true;
                s.pet_state = PetState::Idle;
                tracing::info!("Moocha 已就绪");
            });

            // 情境规则 + 定时提醒：每 60 秒
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;

                    let (actions_raw, app_name) = match tokio::task::spawn_blocking(|| {
                        let app =
                            system_monitor::get_active_app().unwrap_or_else(|_| "Unknown".into());
                        let time =
                            system_monitor::get_system_time().unwrap_or_else(|_| "night".into());
                        let idle = system_monitor::get_idle_duration().unwrap_or(0);
                        let actions = context_rules::check_rules(&app, &time, idle);
                        (actions, app)
                    })
                    .await
                    {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::warn!("情境检测任务 join 失败: {}", e);
                            continue;
                        }
                    };

                    let mut filtered: Vec<context_rules::ContextAction> = Vec::new();
                    if let Some(deduper_state) =
                        app_handle.try_state::<Mutex<context_rules::RuleDeduper>>()
                    {
                        if let Ok(mut deduper) = deduper_state.lock() {
                            for a in actions_raw {
                                let cd = context_rules::cooldown_for_rule(&a.rule);
                                if deduper.try_fire(&a.rule, cd) {
                                    filtered.push(a);
                                }
                            }
                        }
                    } else {
                        tracing::debug!("RuleDeduper 未注册，跳过情境去重 emit");
                    }

                    if !filtered.is_empty() {
                        if let Err(e) = app_handle.emit("context-action", &filtered) {
                            tracing::warn!("emit context-action 失败: {}", e);
                        } else {
                            tracing::debug!("context-action: {} 条", filtered.len());
                        }
                    }

                    // 整点 / 休息 / 久坐
                    let pet_reminders = match app_handle.try_state::<Mutex<reminder::ReminderBundle>>() {
                        Some(rb) => match rb.lock() {
                            Ok(mut b) => {
                                let cfg = b.config.clone();
                                b.manager.process_tick(&app_name, &cfg, Local::now())
                            }
                            Err(_) => Vec::new(),
                        },
                        None => Vec::new(),
                    };

                    if !pet_reminders.is_empty() {
                        if let Err(e) = app_handle.emit("pet-reminder", &pet_reminders) {
                            tracing::warn!("emit pet-reminder 失败: {}", e);
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Moocha 运行时错误");
}
