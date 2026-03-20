// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Moocha 桌面宠物入口。
//!
//! 关闭宠物窗口时**不会退出进程**，仅隐藏窗口；从托盘菜单选择「退出」才会结束应用。
//! 实现位置：`lib.rs` 中 `Builder::on_window_event`（`CloseRequested` → `prevent_close` + `hide`）。

fn main() {
    moocha::run()
}
