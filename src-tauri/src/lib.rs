mod ai;
mod config;
mod state;

use config::AppConfig;
use state::{new_shared_state, SharedState};
use tauri::Manager;

/// 返回宠物当前状态字符串，供前端轮询确认连接。
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

    // 加载配置（数据目录在 Tauri 初始化后才可知，此处先加载默认值）
    let config = AppConfig::default();
    // 注入 API Key（如果已通过环境变量设置）
    let config = AppConfig {
        api_key: std::env::var("MOOCHA_API_KEY").unwrap_or_default(),
        ..config
    };

    let shared_state = new_shared_state(config);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(shared_state)
        .invoke_handler(tauri::generate_handler![get_pet_status])
        .setup(|app| {
            // 应用初始化完成后，将 is_ready 设为 true
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
