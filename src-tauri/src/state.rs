use std::sync::Arc;
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
/// 通过 `Arc<RwLock<_>>` 确保多线程安全访问。
#[derive(Debug)]
pub struct AppState {
    pub config: AppConfig,
    pub pet_mood: PetMood,
    pub is_ready: bool,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            pet_mood: PetMood::Idle,
            is_ready: false,
        }
    }
}

/// Tauri 托管的状态类型别名，方便在 Command 函数签名中使用。
pub type SharedState = Arc<RwLock<AppState>>;

/// 构造一个新的共享状态实例。
pub fn new_shared_state(config: AppConfig) -> SharedState {
    Arc::new(RwLock::new(AppState::new(config)))
}
