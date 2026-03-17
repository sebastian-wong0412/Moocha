mod ai;
mod config;
mod state;

use config::AppConfig;
use state::{new_shared_state, SharedState};
use tauri::Manager;

// ── Commands ──────────────────────────────────────────────────────────────────

/// 返回宠物当前状态字符串，供前端确认后端连接正常。
#[tauri::command]
async fn get_pet_status(state: tauri::State<'_, SharedState>) -> Result<String, String> {
    let s = state.read().await;
    let status = format!(
        "Moocha v{} | mood: {} | ready: {}",
        env!("CARGO_PKG_VERSION"),
        s.pet_mood,
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
        ])
        .setup(|app| {
            let state: tauri::State<'_, SharedState> = app.state();
            let state_clone = state.inner().clone();
            tauri::async_runtime::spawn(async move {
                let mut s = state_clone.write().await;
                s.is_ready = true;
                tracing::info!("Moocha 已就绪");
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Moocha 运行时错误");
}
