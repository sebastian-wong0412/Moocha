/** 单条对话消息 */
export interface ChatMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: number;
}

/** 对话面板完整状态（可用于后续接入全局状态） */
export interface ChatState {
  messages: ChatMessage[];
  isLoading: boolean;
  isError: boolean;
  error?: string;
}
