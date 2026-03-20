import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ChatMessage } from "../types/chat";

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
    throw new Error(msg);
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
      rejectDone(new Error(event.payload));
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
    throw new Error(msg);
  }

  await donePromise;
}
