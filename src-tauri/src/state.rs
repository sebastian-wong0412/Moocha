use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use crate::config::AppConfig;

/// 宠物当前的情感/行为状态。
/// 前端通过 `get_pet_status` 或后续专用命令获取该值。
#[derive(Debug, Clone, PartialEq)]
pub enum PetState {
    Idle,     // 闲置
    Happy,    // 开心
    Sleepy,   // 困倦
    Excited,  // 兴奋
    Curious,  // 好奇
}

impl std::fmt::Display for PetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PetState::Idle    => write!(f, "idle"),
            PetState::Happy   => write!(f, "happy"),
            PetState::Sleepy  => write!(f, "sleepy"),
            PetState::Excited => write!(f, "excited"),
            PetState::Curious => write!(f, "curious"),
        }
    }
}

/// 应用运行时的可变共享状态。
///
/// `config` 单独用 `Arc<Mutex<>>` 包裹，这样读写配置时
/// 不需要锁定整个 `AppState`，避免影响宠物动画等高频状态更新。
#[derive(Debug)]
pub struct AppState {
    /// AI 配置，独立加锁以支持并发读写
    pub config: Arc<Mutex<AppConfig>>,
    pub pet_state: PetState,
    pub is_ready: bool,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            pet_state: PetState::Idle,
            is_ready: false,
        }
    }
}

/// Tauri 托管的顶层状态类型别名。
pub type SharedState = Arc<RwLock<AppState>>;

/// 构造新的共享状态实例。
pub fn new_shared_state(config: AppConfig) -> SharedState {
    Arc::new(RwLock::new(AppState::new(config)))
}
