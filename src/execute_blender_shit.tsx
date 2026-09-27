import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";

export default function ExecuteBlenderShit() {
    const [status, setStatus] = useState<"loading" | "success" | "error">("loading");
    const [message, setMessage] = useState<string>("");
    const navigate = useNavigate();

    useEffect(() => {
        console.log("[execute_blender] Invoking execute_blender_shit...");
        invoke<string>("execute_blender_shit")
            .then((res) => {
                console.log("[execute_blender] Blender generation successful:", res);
                setStatus("success");
                setMessage(res);
            })
            .catch((err) => {
                console.error("[execute_blender] Blender generation failed:", err);
                setStatus("error");
                setMessage(String(err));
            });
    }, []);

    return (
        <div style={{ padding: "40px", fontFamily: "sans-serif" }}>
            <h2>Blender 3D Object Generator</h2>
            {status === "loading" && <p>Generating 3D terrain in Blender...</p>}
            {status === "success" && (
                <div>
                    <p style={{ color: "green" }}>Success!</p>
                    <p>{message}</p>
                    <button
                        onClick={() => {
                            console.log("[execute_blender] Navigating to /create-heatmap...");
                            navigate("/create-heatmap");
                        }}
                        style={{
                            marginTop: "16px",
                            padding: "12px 24px",
                            backgroundColor: "#0078d7",
                            color: "white",
                            border: "none",
                            borderRadius: "6px",
                            cursor: "pointer",
                            fontSize: "14px",
                            fontWeight: "bold",
                        }}
                    >
                        Initiate Hallucination Heatmap Generation
                    </button>
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
