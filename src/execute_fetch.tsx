import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useCoordinates } from "./CoordinateContext";

export default function ExecuteFetch() {
    const { coords } = useCoordinates();
    const [status, setStatus] = useState<"loading" | "success" | "error">("loading");

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
            })
            .catch((err) => {
                console.error("Error:", err);
                setStatus("error");
            });
    }, [coords]);

    return (
        <div style={{ padding: "40px", fontFamily: "sans-serif" }}>
            {status === "loading" && <p>loading image</p>}
            {status === "success" && <p>success</p>}
            {status === "error" && <p>error</p>}
        </div>
    );
}
