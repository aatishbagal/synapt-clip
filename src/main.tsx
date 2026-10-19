import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import "./styles.css";
// Reads experimental_ui once at startup and applies data-experimental-ui.
import "./experimental";

// Set before first paint so the transparent panel window never shows a body
// background; the Settings window (#settings) keeps its opaque one.
if (window.location.hash !== "#settings") {
  document.documentElement.setAttribute("data-window", "panel");
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
