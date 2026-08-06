import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Overlay } from "./windows/overlay/Overlay";
import { lockDownWebview } from "./lib/webview";
import "./styles/global.css";

lockDownWebview();

const root = document.getElementById("root");
if (!root) throw new Error("#root missing from overlay.html");

createRoot(root).render(
  <StrictMode>
    <Overlay />
  </StrictMode>,
);
