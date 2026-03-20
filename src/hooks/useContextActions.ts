import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import type { ContextActionPayload } from "../types/contextAction";

/**
 * 监听后端 `context-action`（情境规则触发 batch）。
 * 卸载时自动 unlisten；可选 `onBatch` 用 ref 持有，避免 effect 重复订阅。
 */
export function useContextActions(
  onBatch?: (actions: ContextActionPayload[]) => void,
): {
  actions: ContextActionPayload[];
  clearActions: () => void;
} {
  const [actions, setActions] = useState<ContextActionPayload[]>([]);
  const onBatchRef = useRef(onBatch);
  onBatchRef.current = onBatch;

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    void listen<ContextActionPayload[]>("context-action", (event) => {
      const batch = event.payload;
      if (!Array.isArray(batch) || batch.length === 0) return;
      setActions((prev) => [...prev, ...batch]);
      onBatchRef.current?.(batch);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const clearActions = useCallback(() => setActions([]), []);

  return { actions, clearActions };
}
