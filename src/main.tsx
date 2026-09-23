import React from "react";
import ReactDOM from "react-dom/client";
import CoordinatePicker from "./selectCoords";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <CoordinatePicker />
  </React.StrictMode>,
);
