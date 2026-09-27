import { useState } from "react";
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
    const [statusMessage, setStatusMessage] = useState<string>("Ready to begin hallucination analysis.");
    const [result, setResult] = useState<HeatmapResult | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [selectedTab, setSelectedTab] = useState<"overlay" | "heatmap" | "consensus">("overlay");
    const [isRunning, setIsRunning] = useState<boolean>(false);

    const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

    async function runPipeline() {
        if (isRunning) return;
        setIsRunning(true);
        setError(null);

        try {
            console.log("[create_heatmap] Starting pipeline...");
            setStep(1);
            setStatusMessage("Step 1/3: Injecting micro-pixel noise into 5 input variants...");
            console.log("[create_heatmap] Invoking generate_noise_variants...");
            await sleep(1000);

            const variants = await invoke<string[]>("generate_noise_variants", {
                inputFilename: "input_RGBN.tiff",
                count: 5,
            });
            console.log("[create_heatmap] Generated noise variants:", variants);

            setStep(2);
            const srImages: string[] = [];
            for (let i = 0; i < variants.length; i++) {
                const inputVar = variants[i];
                const outputVar = `sr_var_${i}.tiff`;
                setStatusMessage(`Step 2/3: Super-resolving variant ${i + 1} of ${variants.length} on RTX 4050 GPU...`);
                console.log(`[create_heatmap] Running super-res on variant ${i + 1}/${variants.length}: ${inputVar} -> ${outputVar}`);

                await invoke<string>("run_super_res_single", {
                    inputPath: inputVar,
                    outputPath: outputVar,
                });
                console.log(`[create_heatmap] Variant ${i + 1} completed: ${outputVar}`);
                srImages.push(outputVar);
                await sleep(1000);
            }

            setStep(3);
            setStatusMessage("Step 3/3: Comparing variants to extract pixel variance and generate hallucination heatmap...");
            console.log("[create_heatmap] Invoking compute_hallucination_analysis with:", srImages);
            await sleep(1000);

            const res = await invoke<HeatmapResult>("compute_hallucination_analysis", {
                srImages,
            });
            console.log("[create_heatmap] Analysis completed successfully:", res.message);

            setResult(res);
            setStep(4);
            setStatusMessage("Analysis complete! Review the results below.");
        } catch (err) {
            console.error("[create_heatmap] Error occurred:", err);
            setError(String(err));
        } finally {
            setIsRunning(false);
        }
    }

    return (
        <div style={{ padding: "20px", maxWidth: "800px", margin: "40px auto" }}>
            <h2>AI Hallucination & Uncertainty Heatmap</h2>
            
            <p>{error ? `Error: ${error}` : statusMessage}</p>

            {isRunning && (
                <p>Loading... Processing super-resolution variants on GPU.</p>
            )}

            {step === 0 && !isRunning && (
                <div style={{ marginBottom: "16px" }}>
                    <button
                        type="button"
                        onClick={runPipeline}
                        style={{ padding: "10px", cursor: "pointer" }}
                    >
                        Start Hallucination Heatmap Generation
                    </button>
                </div>
            )}

            {result && (
                <div style={{ marginTop: "20px" }}>
                    <div style={{ marginBottom: "15px" }}>
                        <button
                            type="button"
                            onClick={() => setSelectedTab("overlay")}
                            style={{
                                padding: "6px 12px",
                                marginRight: "8px",
                                cursor: "pointer",
                                fontWeight: selectedTab === "overlay" ? "bold" : "normal",
                            }}
                        >
                            Heatmap Overlay
                        </button>
                        <button
                            type="button"
                            onClick={() => setSelectedTab("heatmap")}
                            style={{
                                padding: "6px 12px",
                                marginRight: "8px",
                                cursor: "pointer",
                                fontWeight: selectedTab === "heatmap" ? "bold" : "normal",
                            }}
                        >
                            Raw Uncertainty Heatmap
                        </button>
                        <button
                            type="button"
                            onClick={() => setSelectedTab("consensus")}
                            style={{
                                padding: "6px 12px",
                                marginRight: "8px",
                                cursor: "pointer",
                                fontWeight: selectedTab === "consensus" ? "bold" : "normal",
                            }}
                        >
                            Clean Consensus Super-Res (PNG)
                        </button>
                    </div>

                    <div>
                        {selectedTab === "overlay" && (
                            <div>
                                <p><strong>Hallucination Heatmap Overlay (Red/Yellow = AI Uncertainty)</strong></p>
                                <img
                                    src={result.overlay_b64}
                                    alt="Hallucination Overlay"
                                    style={{ maxWidth: "100%", maxHeight: "500px", display: "block", marginTop: "10px" }}
                                />
                            </div>
                        )}
                        {selectedTab === "heatmap" && (
                            <div>
                                <p><strong>Scientific Uncertainty Heatmap (Turbo Colormap)</strong></p>
                                <img
                                    src={result.heatmap_b64}
                                    alt="Raw Heatmap"
                                    style={{ maxWidth: "100%", maxHeight: "500px", display: "block", marginTop: "10px" }}
                                />
                            </div>
                        )}
                        {selectedTab === "consensus" && (
                            <div>
                                <p><strong>Ensemble Consensus Super-Resolution (Noise-Free 4x PNG)</strong></p>
                                <img
                                    src={result.consensus_b64}
                                    alt="Consensus Super-Res"
                                    style={{ maxWidth: "100%", maxHeight: "500px", display: "block", marginTop: "10px" }}
                                />
                            </div>
                        )}
                    </div>

                    <div style={{ marginTop: "16px", paddingTop: "12px", borderTop: "1px solid #ccc" }}>
                        <p><strong>Generated PNG Images:</strong></p>
                        <ul style={{ paddingLeft: "20px" }}>
                            <li>
                                <a href={result.consensus_b64} download="ensemble_consensus_sr.png">
                                    ensemble_consensus_sr.png (4× Super-Resolved Satellite PNG)
                                </a>
                            </li>
                            <li>
                                <a href={result.heatmap_b64} download="hallucination_heatmap.png">
                                    hallucination_heatmap.png (Uncertainty Heatmap PNG)
                                </a>
                            </li>
                            <li>
                                <a href={result.overlay_b64} download="hallucination_overlay.png">
                                    hallucination_overlay.png (Heatmap Overlay PNG)
                                </a>
                            </li>
                        </ul>
                    </div>
                </div>
            )}
        </div>
    );
}
