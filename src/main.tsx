import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import CoordinatePicker from "./selectCoords";
import BearerAuthentication from "./bearer_authentication";
import ExecuteFetch from "./execute_fetch";
import GetHeightMap from "./get height map";
import ExecuteBlenderShit from "./execute_blender_shit";
import CreateHeatmap from "./create_heatmap";
import { CoordinateProvider } from "./CoordinateContext";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <CoordinateProvider>
        <Routes>
          <Route path="/" element={<BearerAuthentication />} />
          <Route path="/select-coords" element={<CoordinatePicker />} />
          <Route path="/execute-fetch" element={<ExecuteFetch />} />
          <Route path="/get-height-map" element={<GetHeightMap />} />
          <Route path="/execute-blender-shit" element={<ExecuteBlenderShit />} />
          <Route path="/create-heatmap" element={<CreateHeatmap />} />
        </Routes>
      </CoordinateProvider>
    </BrowserRouter>
  </React.StrictMode>,
);

