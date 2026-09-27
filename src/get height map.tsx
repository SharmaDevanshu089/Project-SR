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
        <div style={{ padding: "40px", fontFamily: "sans-serif" }}>
            <h2>Height Map Fetcher</h2>
            {status === "loading" && <p>Fetching height map...</p>}
            {status === "success" && (
                <div>
                    <p style={{ color: "green" }}>Success!</p>
                    <p>{message}</p>
                </div>
            )}
            {status === "error" && (
                <div>
                    <p style={{ color: "red" }}>Error</p>
                    <p>{message}</p>
                </div>
            )}
        </div>
    );
}
