import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./windows/settings/App";
import { lockDownWebview } from "./lib/webview";
import "./styles/global.css";

lockDownWebview();

const root = document.getElementById("root");
if (!root) throw new Error("#root missing from index.html");

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
