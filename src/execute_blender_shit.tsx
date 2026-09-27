import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";

export default function ExecuteBlenderShit() {
    const [status, setStatus] = useState<"loading" | "success" | "error">("loading");
    const [message, setMessage] = useState<string>("");
    const [isNavigating, setIsNavigating] = useState<boolean>(false);
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

    const handleProceed = () => {
        setIsNavigating(true);
        console.log("[execute_blender] Navigating to /create-heatmap...");
        setTimeout(() => {
            navigate("/create-heatmap");
        }, 300);
    };

    return (
        <div style={{ padding: "20px", maxWidth: "400px", margin: "40px auto" }}>
            <h2>Blender 3D Object Generator</h2>
            <p>
                {status === "loading" && "Generating 3D terrain in Blender..."}
                {status === "success" && (message ? `Success: ${message}` : "Success: 3D terrain generated.")}
                {status === "error" && `Error: ${message}`}
            </p>
            {status === "success" && (
                isNavigating ? (
                    <p>Loading...</p>
                ) : (
                    <button
                        type="button"
                        onClick={handleProceed}
                        style={{ padding: "10px", cursor: "pointer", width: "100%" }}
                    >
                        Initiate Hallucination Heatmap Generation
                    </button>
                )
            )}
        </div>
    );
}
