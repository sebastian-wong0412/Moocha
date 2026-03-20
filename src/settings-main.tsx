import React from "react";
import ReactDOM from "react-dom/client";
import Settings from "./components/Settings";
/* 勿引入 index.css：其中 html/body overflow:hidden 会导致设置页无法滚轮滚动 */
import "./settings-window.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <div className="settings-scroll-host">
      <Settings standalone />
    </div>
  </React.StrictMode>,
);
