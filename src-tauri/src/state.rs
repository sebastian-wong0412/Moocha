use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use crate::config::AppConfig;

/// 宠物当前的情感/行为状态。
#[derive(Debug, Clone, PartialEq)]
pub enum PetMood {
    Idle,
    Happy,
    Thinking,
    Sleeping,
}

impl std::fmt::Display for PetMood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PetMood::Idle => write!(f, "idle"),
            PetMood::Happy => write!(f, "happy"),
            PetMood::Thinking => write!(f, "thinking"),
            PetMood::Sleeping => write!(f, "sleeping"),
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
    pub pet_mood: PetMood,
    pub is_ready: bool,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            pet_mood: PetMood::Idle,
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
