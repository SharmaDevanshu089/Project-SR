import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface HeatmapResult {
    heatmap_path: string;
    overlay_path: string;
    consensus_path: string;
    heatmap_b64: string;
    overlay_b64: string;
    consensus_b64: string;
    message: string;
}

export default function CreateHeatmap() {
    const [step, setStep] = useState<number>(0);
    const [statusMessage, setStatusMessage] = useState<string>("Initializing hallucination analysis...");
    const [result, setResult] = useState<HeatmapResult | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [selectedTab, setSelectedTab] = useState<"overlay" | "heatmap" | "consensus">("overlay");

    const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

    useEffect(() => {
        async function runPipeline() {
            try {
                setStep(1);
                setStatusMessage("Step 1/3: Injecting micro-pixel noise into 5 input variants...");
                await sleep(1000);

                const variants = await invoke<string[]>("generate_noise_variants", {
                    inputFilename: "input_RGBN.tiff",
                    count: 5,
                });

                setStep(2);
                const srImages: string[] = [];
                for (let i = 0; i < variants.length; i++) {
                    const inputVar = variants[i];
                    const outputVar = `sr_var_${i}.tiff`;
                    setStatusMessage(`Step 2/3: Super-resolving variant ${i + 1} of ${variants.length} on RTX 4050 GPU...`);
                    await invoke<string>("run_super_res_single", {
                        inputPath: inputVar,
                        outputPath: outputVar,
                    });
                    srImages.push(outputVar);
                    await sleep(1000);
                }

                setStep(3);
                setStatusMessage("Step 3/3: Comparing variants to extract pixel variance and generate hallucination heatmap...");
                await sleep(1000);

                const res = await invoke<HeatmapResult>("compute_hallucination_analysis", {
                    srImages,
                });

                setResult(res);
                setStep(4);
                setStatusMessage("Analysis complete! Review the results below.");
            } catch (err) {
                setError(String(err));
            }
        }

        runPipeline();
    }, []);

    return (
        <div style={{ padding: "30px", fontFamily: "sans-serif", maxWidth: "1000px", margin: "0 auto" }}>
            <h2>AI Hallucination & Uncertainty Heatmap</h2>
            
            <div style={{ padding: "15px", background: "#f5f5f5", borderRadius: "8px", marginBottom: "20px" }}>
                <p style={{ margin: "0 0 8px 0", fontWeight: "bold" }}>
                    {step < 4 ? "Processing Pipeline..." : "Completed"}
                </p>
                <p style={{ margin: 0, color: error ? "red" : "#333" }}>
                    {error ? `Error: ${error}` : statusMessage}
                </p>
            </div>

            {result && (
                <div>
                    <div style={{ display: "flex", gap: "10px", marginBottom: "15px" }}>
                        <button
                            type="button"
                            onClick={() => setSelectedTab("overlay")}
                            style={{
                                padding: "8px 16px",
                                background: selectedTab === "overlay" ? "#0078d7" : "#e0e0e0",
                                color: selectedTab === "overlay" ? "white" : "black",
                                border: "none",
                                borderRadius: "4px",
                                cursor: "pointer",
                            }}
                        >
                            Heatmap Overlay
                        </button>
                        <button
                            type="button"
                            onClick={() => setSelectedTab("heatmap")}
                            style={{
                                padding: "8px 16px",
                                background: selectedTab === "heatmap" ? "#0078d7" : "#e0e0e0",
                                color: selectedTab === "heatmap" ? "white" : "black",
                                border: "none",
                                borderRadius: "4px",
                                cursor: "pointer",
                            }}
                        >
                            Raw Uncertainty Heatmap
                        </button>
                        <button
                            type="button"
                            onClick={() => setSelectedTab("consensus")}
                            style={{
                                padding: "8px 16px",
                                background: selectedTab === "consensus" ? "#0078d7" : "#e0e0e0",
                                color: selectedTab === "consensus" ? "white" : "black",
                                border: "none",
                                borderRadius: "4px",
                                cursor: "pointer",
                            }}
                        >
                            Clean Consensus Super-Res
                        </button>
                    </div>

                    <div style={{ border: "1px solid #ddd", borderRadius: "8px", padding: "15px", textAlign: "center" }}>
                        {selectedTab === "overlay" && (
                            <div>
                                <h4>Hallucination Heatmap Overlay (Red/Yellow = AI Uncertainty)</h4>
                                <img
                                    src={result.overlay_b64}
                                    alt="Hallucination Overlay"
                                    style={{ maxWidth: "100%", maxHeight: "550px", borderRadius: "4px" }}
                                />
                            </div>
                        )}
                        {selectedTab === "heatmap" && (
                            <div>
                                <h4>Scientific Uncertainty Heatmap (Turbo Colormap)</h4>
                                <img
                                    src={result.heatmap_b64}
                                    alt="Raw Heatmap"
                                    style={{ maxWidth: "100%", maxHeight: "550px", borderRadius: "4px" }}
                                />
                            </div>
                        )}
                        {selectedTab === "consensus" && (
                            <div>
                                <h4>Ensemble Consensus (Averaged Noise-Free Super-Resolution)</h4>
                                <img
                                    src={result.consensus_b64}
                                    alt="Consensus Super-Res"
                                    style={{ maxWidth: "100%", maxHeight: "550px", borderRadius: "4px" }}
                                />
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}
