import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function ExecuteBlenderShit() {
    const [status, setStatus] = useState<"loading" | "success" | "error">("loading");
    const [message, setMessage] = useState<string>("");

    useEffect(() => {
        invoke<string>("execute_blender_shit")
            .then((res) => {
                setStatus("success");
                setMessage(res);
            })
            .catch((err) => {
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
