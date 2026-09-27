use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HeatmapResult {
    pub heatmap_path: String,
    pub overlay_path: String,
    pub consensus_path: String,
    pub heatmap_b64: String,
    pub overlay_b64: String,
    pub consensus_b64: String,
    pub message: String,
}

fn find_python() -> PathBuf {
    if Path::new(r".\.venv\Scripts\python.exe").exists() {
        PathBuf::from(r".\.venv\Scripts\python.exe")
    } else if Path::new(r"..\.venv\Scripts\python.exe").exists() {
        PathBuf::from(r"..\.venv\Scripts\python.exe")
    } else {
        PathBuf::from("python")
    }
}

#[tauri::command]
pub async fn generate_noise_variants(
    input_filename: Option<String>,
    count: Option<u32>,
) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let base_name = input_filename.unwrap_or_else(|| "input_RGBN.tiff".to_string());
        let num_variants = count.unwrap_or(5);
        let py = find_python();

        let script = format!(
            r#"
import os, numpy as np, rasterio

src_file = "{base}" if os.path.exists("{base}") else os.path.join("..", "{base}")
if not os.path.exists(src_file):
    raise FileNotFoundError(f"Input file not found: {{src_file}}")

with rasterio.open(src_file) as src:
    data = src.read()
    profile = src.profile.copy()

is_uint16 = data.max() > 1.0
noise_scale = 120.0 if is_uint16 else 0.012

for i in range({count}):
    np.random.seed(42 + i * 1337)
    noise = np.random.normal(0.0, noise_scale, data.shape).astype(data.dtype)
    if is_uint16:
        noisy = np.clip(data.astype(np.float32) + noise, 0, 65535).astype(np.uint16)
    else:
        noisy = np.clip(data + noise, 0.0, 1.0)

    out_file = f"var_{{i}}.tiff"
    with rasterio.open(out_file, "w", **profile) as dst:
        dst.write(noisy)
    if os.path.exists("../package.json"):
        with rasterio.open(os.path.join("..", out_file), "w", **profile) as dst:
            dst.write(noisy)
"#,
            base = base_name,
            count = num_variants
        );

        let output = Command::new(&py)
            .arg("-c")
            .arg(&script)
            .output()
            .map_err(|e| format!("Failed to generate noise variants: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Noise generation error: {}", err));
        }

        let mut filenames = Vec::new();
        for i in 0..num_variants {
            filenames.push(format!("var_{}.tiff", i));
        }

        Ok(filenames)
    })
    .await
    .map_err(|e| format!("Task execution error: {}", e))?
}

#[tauri::command]
pub async fn run_super_res_single(input_path: String, output_path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let py = find_python();
        let script_file = if Path::new("run_model.py").exists() {
            "run_model.py"
        } else {
            "../run_model.py"
        };

        let resolved_input = if Path::new(&input_path).exists() {
            input_path.clone()
        } else if Path::new("..").join(&input_path).exists() {
            format!("../{}", input_path)
        } else {
            input_path.clone()
        };

        let output = Command::new(&py)
            .arg(script_file)
            .arg("--input")
            .arg(&resolved_input)
            .arg("--output")
            .arg(&output_path)
            .output()
            .map_err(|e| format!("Failed to execute super-resolution on {}: {}", input_path, e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Super-resolution error for {}: {}", input_path, err));
        }

        if Path::new("../package.json").exists() && Path::new(&output_path).exists() {
            let _ = fs::copy(&output_path, Path::new("..").join(&output_path));
        }

        Ok(format!("Super-resolution completed for {}", output_path))
    })
    .await
    .map_err(|e| format!("Task execution error: {}", e))?
}

#[tauri::command]
pub async fn compute_hallucination_analysis(
    sr_images: Vec<String>,
) -> Result<HeatmapResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let py = find_python();
        let img_list_str = sr_images
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(",");

        let script = format!(
            r#"
import os, io, json, base64, numpy as np, rasterio
from PIL import Image
import matplotlib.pyplot as plt

image_files = [{files}]
stack = []
ref_profile = None

for f in image_files:
    path = f if os.path.exists(f) else os.path.join("..", f)
    with rasterio.open(path) as src:
        d = src.read().astype(np.float32)
        if d.max() > 1.0:
            d = d / 10000.0
        stack.append(d)
        if ref_profile is None:
            ref_profile = src.profile.copy()

stack = np.stack(stack, axis=0)
consensus = np.mean(stack, axis=0)

rgb_stack = stack[:, :3, :, :]
std_per_band = np.std(rgb_stack, axis=0)
uncertainty = np.mean(std_per_band, axis=0)

p_low, p_high = np.percentile(uncertainty, (2, 98))
if p_high > p_low:
    norm_uncertainty = np.clip((uncertainty - p_low) / (p_high - p_low), 0.0, 1.0)
else:
    norm_uncertainty = np.zeros_like(uncertainty)

cmap = plt.get_cmap("turbo")
heatmap_rgba = cmap(norm_uncertainty)
heatmap_u8 = (heatmap_rgba[:, :, :3] * 255).astype(np.uint8)
Image.fromarray(heatmap_u8).save("hallucination_heatmap.png")

rgb_consensus = consensus[:3, :, :].transpose(1, 2, 0)
stretched_rgb = np.zeros_like(rgb_consensus)
for c in range(3):
    ch = rgb_consensus[:, :, c]
    pl, ph = np.percentile(ch, (2, 98))
    if ph > pl:
        stretched_rgb[:, :, c] = np.clip((ch - pl) / (ph - pl), 0.0, 1.0)
    else:
        stretched_rgb[:, :, c] = np.clip(ch, 0.0, 1.0)

stretched_rgb = np.power(stretched_rgb, 0.9)
consensus_u8 = (stretched_rgb * 255).astype(np.uint8)
Image.fromarray(consensus_u8).save("ensemble_consensus_sr.png")

overlay_u8 = (0.65 * consensus_u8.astype(np.float32) + 0.35 * heatmap_u8.astype(np.float32)).astype(np.uint8)
Image.fromarray(overlay_u8).save("hallucination_overlay.png")

if os.path.exists("../package.json"):
    Image.fromarray(heatmap_u8).save("../hallucination_heatmap.png")
    Image.fromarray(consensus_u8).save("../ensemble_consensus_sr.png")
    Image.fromarray(overlay_u8).save("../hallucination_overlay.png")

if ref_profile:
    ref_profile.update(count=1, dtype="float32", compress="deflate")
    with rasterio.open("hallucination_metric.tiff", "w", **ref_profile) as dst:
        dst.write(uncertainty.astype(np.float32), 1)
    if os.path.exists("../package.json"):
        with rasterio.open("../hallucination_metric.tiff", "w", **ref_profile) as dst:
            dst.write(uncertainty.astype(np.float32), 1)

def to_b64(arr):
    img = Image.fromarray(arr).resize((512, 512), Image.BILINEAR)
    buf = io.BytesIO()
    img.save(buf, format="PNG", optimize=True)
    return "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode("utf-8")

result = {{
    "heatmap_path": "hallucination_heatmap.png",
    "overlay_path": "hallucination_overlay.png",
    "consensus_path": "ensemble_consensus_sr.png",
    "heatmap_b64": to_b64(heatmap_u8),
    "overlay_b64": to_b64(overlay_u8),
    "consensus_b64": to_b64(consensus_u8),
    "message": "Hallucination heatmap and consensus images generated successfully"
}}

print(json.dumps(result))
"#,
            files = img_list_str
        );

        let output = Command::new(&py)
            .arg("-c")
            .arg(&script)
            .output()
            .map_err(|e| format!("Failed to compute hallucination heatmap: {}", e))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Hallucination calculation error: {}", err));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        let res: HeatmapResult = serde_json::from_str(trimmed)
            .map_err(|e| format!("Failed to parse heatmap JSON response: {}. Output: {}", e, trimmed))?;

        Ok(res)
    })
    .await
    .map_err(|e| format!("Task execution error: {}", e))?
}
