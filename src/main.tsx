import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import CoordinatePicker from "./selectCoords";
import BearerAuthentication from "./bearer_authentication";
import { CoordinateProvider } from "./CoordinateContext";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <CoordinateProvider>
        <Routes>
          <Route path="/" element={<BearerAuthentication />} />
          <Route path="/select-coords" element={<CoordinatePicker />} />
        </Routes>
      </CoordinateProvider>
    </BrowserRouter>
  </React.StrictMode>,
);

