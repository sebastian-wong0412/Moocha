import { invoke } from "@tauri-apps/api/core";
import { AppConfig } from "../types/config";
import type { ReminderConfig } from "../types/reminderConfig";

/**
 * 从 Rust 后端获取当前应用配置。
 * 注意：返回的 api_key 已脱敏为 "****"。
 */
export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>("get_config");
}

/**
 * 将配置保存到后端（持久化到磁盘）。
 * 若 config.api_key 为空字符串，后端会保留已有密钥，不会清空。
 */
export async function saveConfig(config: AppConfig): Promise<void> {
  return invoke("save_config", { config });
}

/**
 * 测试已保存配置下与 AI 服务的网络连通性。
 * 返回 true 表示服务可达（HTTP < 500），false 表示不可达。
 */
export async function testConnection(): Promise<boolean> {
  return invoke<boolean>("test_connection");
}

/**
 * 用指定参数临时测试连通性，不读写任何已保存的配置。
 * 用于"填完表单但尚未保存"的场景，测试与保存完全解耦。
 */
export async function testConnectionWith(
  baseUrl: string,
  apiKey: string
): Promise<boolean> {
  return invoke<boolean>("test_connection_with", {
    baseUrl,
    apiKey,
  });
}

/** 提醒配置（整点 / 休息间隔 / 久坐） */
export async function getReminderConfig(): Promise<ReminderConfig> {
  return invoke<ReminderConfig>("get_reminder_config");
}

export async function updateReminderConfig(
  config: ReminderConfig,
): Promise<void> {
  await invoke("update_reminder_config", { config });
}

/** 用户已读提醒；休息/久坐会重置连续工作计时 */
export async function acknowledgeReminder(kind: string): Promise<void> {
  await invoke("acknowledge_reminder", { kind });
}

/** 手动触发休息提醒（测试） */
export async function triggerBreakReminder(): Promise<void> {
  await invoke("trigger_break_reminder");
}

/** 显示独立对话窗口 */
export async function showChatWindow(): Promise<void> {
  await invoke("show_chat_window");
}

export async function hideChatWindow(): Promise<void> {
  await invoke("hide_chat_window");
}

/** 手动固定 / 取消主窗口置顶（托盘「置顶宠物」） */
export async function setPetAlwaysOnTop(enabled: boolean): Promise<void> {
  await invoke("set_pet_always_on_top", { enabled });
}

/** 当前主窗口是否置顶（警报或手动固定） */
export async function isPetAlwaysOnTop(): Promise<boolean> {
  return invoke<boolean>("is_pet_always_on_top");
}

/** 提醒队列清空后调用：结束警报置顶，不影响手动置顶 */
export async function clearPetAlertTop(): Promise<void> {
  await invoke("clear_pet_alert_top");
}
