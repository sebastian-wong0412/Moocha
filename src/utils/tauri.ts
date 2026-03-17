import { invoke } from "@tauri-apps/api/core";
import { AppConfig } from "../types/config";

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
