import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ChatMessage } from "../types/chat";

/** 与后端 `lib.rs` 一致的未配置提示 */
export const FRIENDLY_NO_API_KEY =
  "暂时未接入 AI 模型哦~ 请在设置中配置 API Key";

/**
 * 将 invoke / 事件中的原始错误转为对用户友好的中文说明。
 * 后端已映射过的文案会原样返回。
 */
export function mapChatFriendlyError(raw: string): string {
  const t = raw.trim();
  if (!t) return FRIENDLY_NO_API_KEY;
  if (
    t.includes("暂时未接入 AI 模型") ||
    t.includes("认证失败，请检查") ||
    t.includes("网络连接失败，请检查") ||
    t.includes("请求的接口不存在")
  ) {
    return t;
  }
  const l = t.toLowerCase();
  if (
    l.includes("401") ||
    l.includes("unauthorized") ||
    l.includes("403") ||
    l.includes("forbidden")
  ) {
    return "认证失败，请检查 API Key 或权限设置是否正确";
  }
  if (
    l.includes("timeout") ||
    l.includes("timed out") ||
    l.includes("failed to resolve") ||
    l.includes("connection") ||
    l.includes("dns") ||
    l.includes("network") ||
    l.includes("fetch")
  ) {
    return "网络连接失败，请检查网络或服务地址是否可达";
  }
  if (l.includes("404")) {
    return "请求的接口不存在，请检查 Base URL 与模型名称";
  }
  return "暂时无法连接 AI 服务，请稍后在设置中检查配置";
}

/** 从磁盘加载对话历史 */
export async function getChatHistory(): Promise<ChatMessage[]> {
  return invoke<ChatMessage[]>("get_chat_history");
}

/** 追加一条消息并持久化 */
export async function saveChatMessage(message: ChatMessage): Promise<void> {
  await invoke("save_chat_message", { message });
}

/** 删除历史文件 */
export async function clearChatHistory(): Promise<void> {
  await invoke("clear_chat_history");
}

/** 后端 `chat` / `chat_stream` 所需的 context 项格式 */
function toInvokeContext(context: ChatMessage[]): { role: string; content: string }[] {
  return context.map((m) => ({ role: m.role, content: m.content }));
}

/**
 * 非流式对话：等待完整回复后返回。
 */
export async function sendMessage(
  message: string,
  context: ChatMessage[],
): Promise<string> {
  try {
    return await invoke<string>("chat", {
      message,
      context: toInvokeContext(context),
    });
  } catch (e) {
    const msg =
      typeof e === "string"
        ? e
        : e instanceof Error
          ? e.message
          : JSON.stringify(e);
    throw new Error(mapChatFriendlyError(msg));
  }
}

/**
 * 流式对话：先注册事件监听，再调用 `chat_stream`；
 * 通过 `onChunk` 推送每个文本块，`chat-done` / `chat-error` 时结束并清理监听。
 *
 * @param signal 传入后可在组件卸载时 `abort()`，会移除监听并中断等待（Promise reject）。
 */
export async function sendMessageStream(
  message: string,
  context: ChatMessage[],
  onChunk: (chunk: string) => void,
  signal?: AbortSignal,
): Promise<void> {
  if (signal?.aborted) {
    throw new DOMException("aborted", "AbortError");
  }

  const unlisteners: UnlistenFn[] = [];

  const cleanup = () => {
    for (const u of unlisteners) {
      try {
        u();
      } catch {
        /* ignore */
      }
    }
    unlisteners.length = 0;
  };

  let resolveDone!: () => void;
  let rejectDone!: (e: Error) => void;
  const donePromise = new Promise<void>((res, rej) => {
    resolveDone = res;
    rejectDone = rej;
  });

  const onAbort = () => {
    cleanup();
    rejectDone(new DOMException("aborted", "AbortError"));
  };
  signal?.addEventListener("abort", onAbort, { once: true });

  try {
    // 必须先挂监听，再 invoke，避免极快返回时丢事件
    const ulChunk = await listen<string>("chat-chunk", (event) => {
      onChunk(event.payload);
    });
    unlisteners.push(ulChunk);

    const ulDone = await listen("chat-done", () => {
      cleanup();
      signal?.removeEventListener("abort", onAbort);
      resolveDone();
    });
    unlisteners.push(ulDone);

    const ulErr = await listen<string>("chat-error", (event) => {
      cleanup();
      signal?.removeEventListener("abort", onAbort);
      rejectDone(new Error(mapChatFriendlyError(event.payload)));
    });
    unlisteners.push(ulErr);

    await invoke("chat_stream", {
      message,
      context: toInvokeContext(context),
    });
  } catch (e) {
    cleanup();
    signal?.removeEventListener("abort", onAbort);
    const msg =
      typeof e === "string"
        ? e
        : e instanceof Error
          ? e.message
          : JSON.stringify(e);
    throw new Error(mapChatFriendlyError(msg));
  }

  await donePromise;
}
