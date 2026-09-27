import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { useCoordinates } from "./CoordinateContext";

export default function GetHeightMap() {
    const { coords } = useCoordinates();
    const [status, setStatus] = useState<"loading" | "success" | "error">("loading");
    const [message, setMessage] = useState<string>("");
    const navigate = useNavigate();

    useEffect(() => {
        if (!coords) {
            setStatus("error");
            setMessage("No coordinates found");
            return;
        }

        invoke<string>("get_height_map", {
            lat: coords.lat,
            lon: coords.lng,
            lng: coords.lng,
        })
            .then((res) => {
                setStatus("success");
                setMessage(res);
                setTimeout(() => {
                    navigate("/execute-blender-shit");
                }, 1000);
            })
            .catch((err) => {
                setStatus("error");
                setMessage(String(err));
            });
    }, [coords, navigate]);

    return (
        <div style={{ padding: "20px", maxWidth: "400px", margin: "40px auto" }}>
            <h2>Height Map Fetcher</h2>
            <p>
                {status === "loading" && "Fetching height map..."}
                {status === "success" && `Success: ${message || "Height map acquired. Redirecting..."}`}
                {status === "error" && `Error: ${message}`}
            </p>
        </div>
    );
}
