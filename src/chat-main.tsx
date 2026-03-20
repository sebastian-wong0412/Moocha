import React from "react";
import ReactDOM from "react-dom/client";
import Chat from "./components/Chat";
import "./index.css";
import "./chat-window.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <div className="chat-page-root">
      <Chat standalone />
    </div>
  </React.StrictMode>,
);
