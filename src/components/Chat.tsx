import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import type { PetMood } from "../types/pet";
import type { ChatMessage } from "../types/chat";
import {
  clearChatHistory,
  getChatHistory,
  saveChatMessage,
  sendMessageStream,
} from "../utils/chat";
import "./Chat.css";

interface Props {
  onClose: () => void;
  /** 当前宠物心情（展示用） */
  petMood?: PetMood;
  /** 对话情绪联动：发送 / 首包 / 完成 / 错误 时更新宠物 */
  onMoodChange?: (mood: PetMood) => void;
}

function newId(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}

const DEFAULT_WELCOME: ChatMessage = {
  id: "welcome-local",
  role: "assistant",
  content: "你好！我是 Moocha 🐱 有什么想聊的吗？",
  timestamp: Date.now(),
};

/** 后端返回的 role 收窄为前端联合类型 */
function normalizeHistoryRow(m: ChatMessage): ChatMessage {
  const role: ChatMessage["role"] =
    m.role === "user" || m.role === "assistant" ? m.role : "user";
  return { ...m, role };
}

/**
 * 对话面板：流式 `chat_stream` + 历史持久化 + 宠物情绪联动。
 */
export default function Chat({ onClose, petMood, onMoodChange }: Props) {
  const [messages, setMessages] = useState<ChatMessage[]>([DEFAULT_WELCOME]);
  const [historyLoaded, setHistoryLoaded] = useState(false);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [streamingMessageId, setStreamingMessageId] = useState<string | null>(null);
  const [isError, setIsError] = useState(false);
  const [error, setError] = useState<string | undefined>();

  const listRef = useRef<HTMLDivElement>(null);
  const assistantStreamIdRef = useRef<string | null>(null);
  /** 当前轮助手流式累积（用于成功后写盘） */
  const streamAccumRef = useRef({ id: "", content: "", ts: 0 });
  const abortRef = useRef<AbortController | null>(null);

  // 挂载时加载历史；无记录则保留本地欢迎语
  useEffect(() => {
    let cancelled = false;
    getChatHistory()
      .then((rows) => {
        if (cancelled) return;
        if (rows.length > 0) {
          setMessages(rows.map(normalizeHistoryRow));
        } else {
          setMessages([{ ...DEFAULT_WELCOME, id: newId(), timestamp: Date.now() }]);
        }
      })
      .catch((e) => {
        if (!cancelled) {
          setIsError(true);
          setError(`加载历史失败: ${String(e)}`);
        }
      })
      .finally(() => {
        if (!cancelled) setHistoryLoaded(true);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const el = listRef.current;
    if (!el) return;
    el.scrollTo({ top: el.scrollHeight, behavior: "smooth" });
  }, [messages, isLoading, streamingMessageId]);

  useEffect(() => {
    return () => {
      abortRef.current?.abort();
      abortRef.current = null;
    };
  }, []);

  async function handleClearHistory() {
    if (isLoading) return;
    try {
      await clearChatHistory();
      setMessages([{ ...DEFAULT_WELCOME, id: newId(), timestamp: Date.now() }]);
      setIsError(false);
      setError(undefined);
    } catch (e) {
      setIsError(true);
      setError(`清除历史失败: ${String(e)}`);
    }
  }

  async function handleSend() {
    const text = input.trim();
    if (!text || isLoading || !historyLoaded) return;

    setIsError(false);
    setError(undefined);
    onMoodChange?.("excited");

    const userMsg: ChatMessage = {
      id: newId(),
      role: "user",
      content: text,
      timestamp: Date.now(),
    };

    const historyForApi: ChatMessage[] = [...messages, userMsg];

    setMessages((prev) => [...prev, userMsg]);
    setInput("");
    setIsLoading(true);
    assistantStreamIdRef.current = null;
    streamAccumRef.current = { id: "", content: "", ts: 0 };
    setStreamingMessageId(null);

    try {
      await saveChatMessage(userMsg);
    } catch (e) {
      setIsError(true);
      setError(`保存消息失败: ${String(e)}`);
    }

    abortRef.current?.abort();
    const ac = new AbortController();
    abortRef.current = ac;

    try {
      await sendMessageStream(
        text,
        historyForApi,
        (chunk) => {
          if (assistantStreamIdRef.current === null) {
            const id = newId();
            const ts = Date.now();
            assistantStreamIdRef.current = id;
            streamAccumRef.current = { id, content: chunk, ts };
            onMoodChange?.("curious");
            setIsLoading(false);
            setStreamingMessageId(id);
            setMessages((prev) => [
              ...prev,
              {
                id,
                role: "assistant",
                content: chunk,
                timestamp: ts,
              },
            ]);
            return;
          }
          streamAccumRef.current.content += chunk;
          const aid = assistantStreamIdRef.current;
          setMessages((prev) =>
            prev.map((m) =>
              m.id === aid ? { ...m, content: m.content + chunk } : m,
            ),
          );
        },
        ac.signal,
      );
      onMoodChange?.("happy");

      const { id, content, ts } = streamAccumRef.current;
      if (id && content.length > 0) {
        try {
          await saveChatMessage({
            id,
            role: "assistant",
            content,
            timestamp: ts,
          });
        } catch (saveErr) {
          setIsError(true);
          setError(`保存回复失败: ${String(saveErr)}`);
        }
      }
    } catch (e) {
      if (e instanceof DOMException && e.name === "AbortError") {
        return;
      }
      onMoodChange?.("sleepy");
      const msg =
        e instanceof Error ? e.message : typeof e === "string" ? e : String(e);
      setIsError(true);
      setError(msg);
    } finally {
      assistantStreamIdRef.current = null;
      streamAccumRef.current = { id: "", content: "", ts: 0 };
      setStreamingMessageId(null);
      setIsLoading(false);
      if (abortRef.current === ac) {
        abortRef.current = null;
      }
    }
  }

  function handleKeyDown(e: KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void handleSend();
    }
  }

  const moodLabel = petMood ?? "idle";

  return (
    <div className="chat-overlay" role="dialog" aria-labelledby="chat-title">
      <div className="chat-panel">
        <header className="chat-header">
          <div className="chat-header-text">
            <span id="chat-title" className="chat-title">
              💬 对话
            </span>
            <span className="chat-mood" title="当前宠物心情">
              心情 · {moodLabel}
            </span>
          </div>
          <div className="chat-header-actions">
            <button
              type="button"
              className="chat-clear"
              onClick={() => void handleClearHistory()}
              disabled={isLoading}
              title="清除本地对话历史"
            >
              清空
            </button>
            <button
              type="button"
              className="chat-close"
              onClick={onClose}
              title="关闭"
              aria-label="关闭对话"
            >
              ×
            </button>
          </div>
        </header>

        {isError && error && (
          <div className="chat-error-banner" role="alert">
            {error}
          </div>
        )}

        <div ref={listRef} className="chat-messages">
          {messages.map((m) => (
            <div
              key={m.id}
              className={`chat-row chat-row--${m.role}`}
            >
              <div>
                <div
                  className={`chat-bubble chat-bubble--${m.role}${
                    m.id === streamingMessageId ? " chat-bubble--streaming" : ""
                  }`}
                >
                  {m.content}
                  {m.id === streamingMessageId && (
                    <span className="chat-stream-cursor" aria-hidden />
                  )}
                </div>
                <div className="chat-meta">
                  {m.role === "user" ? "你" : "Moocha"} ·{" "}
                  {new Date(m.timestamp).toLocaleTimeString(undefined, {
                    hour: "2-digit",
                    minute: "2-digit",
                  })}
                </div>
              </div>
            </div>
          ))}

          {isLoading && (
            <div className="chat-row chat-row--assistant">
              <div>
                <div className="chat-bubble chat-bubble--assistant">
                  <span className="chat-thinking" aria-live="polite">
                    思考中
                    <span className="chat-dot" aria-hidden />
                    <span className="chat-dot" aria-hidden />
                    <span className="chat-dot" aria-hidden />
                  </span>
                </div>
              </div>
            </div>
          )}
        </div>

        <footer className="chat-footer">
          <textarea
            className="chat-input"
            rows={1}
            placeholder="输入消息…"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={isLoading || !historyLoaded}
            aria-label="输入消息"
          />
          <button
            type="button"
            className="chat-send"
            onClick={() => void handleSend()}
            disabled={isLoading || !input.trim() || !historyLoaded}
          >
            发送
          </button>
        </footer>
      </div>
    </div>
  );
}
