import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

/** 后端系统监控快照（与各类 `get_*` 命令对齐） */
export interface SystemStatus {
  activeApp: string;
  systemTime: string;
  idleDuration: number;
  /** 全局 CPU 使用率 0–100 */
  cpuUsage: number;
  /** 已用物理内存（字节） */
  memoryUsage: number;
}

const DEFAULT_STATUS: SystemStatus = {
  activeApp: "Unknown",
  systemTime: "night",
  idleDuration: 0,
  cpuUsage: 0,
  memoryUsage: 0,
};

const POLL_INTERVAL_MS = 30_000;

/**
 * 轮询系统状态：前台应用、本地时段、空闲秒数。
 * 每 30 秒刷新；组件卸载时清除定时器。
 */
export function useSystemStatus(): SystemStatus {
  const [state, setState] = useState<SystemStatus>(DEFAULT_STATUS);

  useEffect(() => {
    let cancelled = false;

    async function poll() {
      try {
        const [activeApp, systemTime, idleDuration, cpuUsage, memoryUsage] =
          await Promise.all([
            invoke<string>("get_active_app"),
            invoke<string>("get_system_time"),
            invoke<number>("get_idle_duration"),
            invoke<number>("get_cpu_usage"),
            invoke<number>("get_memory_usage"),
          ]);
        if (!cancelled) {
          setState({
            activeApp,
            systemTime,
            idleDuration: Number(idleDuration),
            cpuUsage: Number(cpuUsage),
            memoryUsage: Number(memoryUsage),
          });
        }
      } catch (e) {
        console.warn("[useSystemStatus] 轮询失败:", e);
        if (!cancelled) {
          setState((prev) => ({
            activeApp: "Unknown",
            systemTime: prev.systemTime,
            idleDuration: 0,
            cpuUsage: 0,
            memoryUsage: 0,
          }));
        }
      }
    }

    void poll();
    const timerId = window.setInterval(() => void poll(), POLL_INTERVAL_MS);

    return () => {
      cancelled = true;
      window.clearInterval(timerId);
    };
  }, []);

  return state;
}
