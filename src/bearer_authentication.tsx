import React, { useState } from "react";

export default function BearerAuthentication() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");

  const handleGenerateToken = (userEmail: string, userPassword: string) => {
    console.log("Invoking token generation function with:", { userEmail, userPassword });
    // Token generation / API logic can be added here
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
