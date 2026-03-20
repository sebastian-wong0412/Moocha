import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import Pet from "./components/Pet";
import type { PetMood } from "./types/pet";
import { usePetPatrol } from "./hooks/usePetPatrol";
import { showChatWindow } from "./utils/tauri";
import "./App.css";

const PET_MOODS: readonly PetMood[] = [
  "idle",
  "happy",
  "sleepy",
  "excited",
  "curious",
];

function isPetMood(v: string): v is PetMood {
  return (PET_MOODS as readonly string[]).includes(v);
}

function App() {
  usePetPatrol();
  const [petMood, setPetMood] = useState<PetMood>("idle");

  useEffect(() => {
    invoke<string>("get_pet_status")
      .then((status) => {
        const match = status.match(/mood: (\w+)/);
        const mood = (match?.[1] ?? "idle") as PetMood;
        setPetMood(mood);
      })
      .catch(() => setPetMood("idle"));
  }, []);

  // 独立对话窗口通过 emit 同步心情
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<string>("pet-mood-sync", (event) => {
      const m = event.payload;
      if (isPetMood(m)) setPetMood(m);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  return (
    <div className="app-root">
      <Pet mood={petMood} onMoodChange={setPetMood} />

      <div className="app-top-actions">
        <button
          type="button"
          className="chat-btn"
          onClick={() => void showChatWindow()}
          title="对话"
        >
          💬
        </button>
      </div>
    </div>
  );
}

export default App;
