import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

/**
 * 启用后端宠物巡逻（底部随机横移 + 休息）。
 * 卸载时停止；开发环境 StrictMode 会短暂 stop→start，无妨。
 */
export function usePetPatrol(): void {
  useEffect(() => {
    void invoke("start_pet_patrol");
    return () => {
      void invoke("stop_pet_patrol");
    };
  }, []);
}
