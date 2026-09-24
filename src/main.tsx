import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import CoordinatePicker from "./selectCoords";
import BearerAuthentication from "./bearer_authentication";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<BearerAuthentication />} />
        <Route path="/select-coords" element={<CoordinatePicker />} />
      </Routes>
    </BrowserRouter>
  </React.StrictMode>,
);

