import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function BearerAuthentication() {
    const [email, setEmail] = useState("");
    const [password, setPassword] = useState("");

    const handleGenerateToken = (userEmail: string, userPassword: string) => {
        console.log("Invoking token generation function with:", { userEmail, userPassword });
        invoke("get_copernicus_token", { username: userEmail, password: userPassword })
            .then((token) => {
                console.log("Token generated successfully:", token);
                invoke("new_bearer_token", { bearerToken: token })
                    .then(() => {
                        console.log("Token saved successfully");
                    })
                    .catch((error) => {
                        console.error("Error saving token:", error);
                    });
            })
            .catch((error) => {
                console.error("Error generating token:", error);
            });
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        console.log("Submit clicked:", { email, password });
        handleGenerateToken(email, password);
    };

    return (
        <div style={{ padding: "20px", maxWidth: "400px", margin: "40px auto" }}>
            <h2>Bearer Authentication</h2>
            <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
                <div>
                    <label style={{ display: "block", marginBottom: "4px" }}>Email:</label>
                    <input
                        type="email"
                        placeholder="Enter your email"
                        value={email}
                        onChange={(e) => setEmail(e.target.value)}
                        required
                        style={{ width: "100%", padding: "8px", boxSizing: "border-box" }}
                    />
                </div>

                <div>
                    <label style={{ display: "block", marginBottom: "4px" }}>Password:</label>
                    <input
                        type="password"
                        placeholder="Enter your password"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        required
                        style={{ width: "100%", padding: "8px", boxSizing: "border-box" }}
                    />
                </div>

                <button type="submit" style={{ padding: "10px", cursor: "pointer" }}>
                    Submit
                </button>
            </form>
        </div>
    );
}
