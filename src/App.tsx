import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Pet from "./components/Pet";
import type { PetMood } from "./types/pet";
import Settings from "./components/Settings";
import Chat from "./components/Chat";
import "./App.css";

function App() {
  const [petMood, setPetMood] = useState<PetMood>("idle");
  const [showSettings, setShowSettings] = useState(false);
  const [showChat, setShowChat] = useState(false);

  // 启动时从后端获取初始状态，解析 mood 字段
  useEffect(() => {
    invoke<string>("get_pet_status")
      .then((status) => {
        const match = status.match(/mood: (\w+)/);
        const mood = (match?.[1] ?? "idle") as PetMood;
        setPetMood(mood);
      })
      .catch(() => setPetMood("idle"));
  }, []);

  return (
    <div className="app-root">
      {/* 宠物主体 */}
      <Pet mood={petMood} onMoodChange={setPetMood} />

      {/* 右上角：对话 + 设置 */}
      <div className="app-top-actions">
        <button
          type="button"
          className="chat-btn"
          onClick={() => setShowChat((v) => !v)}
          title={showChat ? "关闭对话" : "对话"}
        >
          💬
        </button>
        <button
          type="button"
          className="settings-btn"
          onClick={() => setShowSettings(true)}
          title="设置"
        >
          ⚙️
        </button>
      </div>

      {/* 对话面板（模态覆盖） */}
      {showChat && (
        <Chat
          petMood={petMood}
          onMoodChange={setPetMood}
          onClose={() => setShowChat(false)}
        />
      )}

      {/* 设置面板（按需渲染） */}
      {showSettings && (
        <Settings onClose={() => setShowSettings(false)} />
      )}
    </div>
  );
}

export default App;
