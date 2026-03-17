import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [status, setStatus] = useState<string>("Moocha Loading...");

  useEffect(() => {
    invoke<string>("get_pet_status")
      .then((s) => setStatus(s))
      .catch(() => setStatus("Moocha Loading..."));
  }, []);

  return (
    <div className="pet-container">
      <p className="status-text">{status}</p>
    </div>
  );
}

export default App;
