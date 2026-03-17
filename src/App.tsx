import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Settings from "./components/Settings";
import "./App.css";

function App() {
  const [status, setStatus] = useState<string>("Moocha Loading...");
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    invoke<string>("get_pet_status")
      .then((s) => setStatus(s))
      .catch(() => setStatus("Moocha Loading..."));
  }, []);

  return (
    <div className="pet-container">
      {/* 宠物状态文字 */}
      <p className="status-text">{status}</p>

      {/* 右上角设置按钮 */}
      <button
        className="settings-btn"
        onClick={() => setShowSettings(true)}
        title="设置"
      >
        ⚙️
      </button>

      {/* 设置面板（按需渲染） */}
      {showSettings && (
        <Settings onClose={() => setShowSettings(false)} />
      )}
    </div>
  );
}

export default App;
