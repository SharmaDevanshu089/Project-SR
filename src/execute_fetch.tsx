import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { useCoordinates } from "./CoordinateContext";

export default function ExecuteFetch() {
    const { coords } = useCoordinates();
    const [status, setStatus] = useState<"loading" | "success" | "error">("loading");
    const navigate = useNavigate();

    useEffect(() => {
        if (!coords) {
            console.error("No coordinates found in context.");
            setStatus("error");
            return;
        }

        console.log("Fetching image for coordinates:", coords);
        invoke<string>("fetch_sentinel_patch", {
            lat: coords.lat,
            lon: coords.lng,
            lng: coords.lng,
        })
            .then((res) => {
                console.log("Success:", res);
                setStatus("success");
                setTimeout(() => {
                    navigate("/get-height-map");
                }, 1000);
            })
            .catch((err) => {
                console.error("Error:", err);
                setStatus("error");
            });
    }, [coords, navigate]);

    return (
        <div style={{ padding: "20px", maxWidth: "400px", margin: "40px auto" }}>
            <h2>Satellite Image Fetcher</h2>
            <p>
                {status === "loading" && "Fetching Sentinel-2 4-band image..."}
                {status === "success" && "Success: Image acquired. Redirecting to height map..."}
                {status === "error" && "Error: Failed to fetch satellite image."}
            </p>
        </div>
    );
}

