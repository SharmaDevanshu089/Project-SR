import React from "react";
import ReactDOM from "react-dom/client";
import CoordinatePicker from "./selectCoords";
import BearerAuthentication from "./bearer_authentication";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BearerAuthentication />
  </React.StrictMode>,
);
