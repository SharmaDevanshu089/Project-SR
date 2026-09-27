# Project SR - Execution & Debugging Log

A comprehensive reference of issues encountered, root-cause analyses, code fixes applied, and how the super-resolution pipeline works.

---

## 1. Initial Problem: `ModuleNotFoundError: No module named 'mamba_ssm'`

### What Happened
Running `python run_model.py` crashed during model initialization:
```text
ModuleNotFoundError: No module named 'mamba_ssm'
ImportError: Please install the mamba_ssm package before using MambaSR model.
RuntimeError: Failed to load Python module from model/SEN2SR_RGBN
```

### Root Cause
1. **CUDA / GPU Requirement:** The default model (`SEN2SR/NonReference_RGBN_x4`) uses **MambaSR** (State-Space Model), which requires `mamba_ssm` compiled CUDA C++ kernels (`selective_scan_cuda`).
2. **Platform & Device Incompatibility:** The execution environment is on Windows running on CPU (`Using device: cpu`). `mamba_ssm` cannot compile easily on Windows without specialized MSVC/CUDA toolchains and **fundamentally does not run on CPU**.

### Fix Applied
Switched the model in `run_model.py` to the official lightweight version provided by the authors (`tacofoundation/sen2sr`):
- **Model Dir:** `model/SEN2SRLite_RGBN`
- **Model MLM URL:** `https://huggingface.co/tacofoundation/sen2sr/resolve/main/SEN2SRLite/NonReference_RGBN_x4/mlm.json`
- **Architecture:** `SEN2SRLite` uses `CNNSR` (convolutional super-resolution) instead of Mamba state space models. It runs purely on standard PyTorch on CPU/GPU across Windows, macOS, and Linux without external compiled C++/CUDA kernels.

---

## 2. Issue: "No Change in Clarity" & "128-bit 200MB Image"

### Issue 2A: Why the image was a 256MB "128-bit image"
1. **128-Bit Pixel Depth:** The output raster has 4 channels (`Red, Green, Blue, NIR`) saved as 32-bit floats. $4 \times 32\text{ bits} = \mathbf{128\text{ bits per pixel}}$.
2. **No GeoTIFF Compression:** An uncompressed $4096 \times 4096 \times 4\text{ bands} \times 4\text{ bytes}$ GeoTIFF is **$268\text{ MB}$** on disk.
3. **Display Incompatibility:** Surface reflectance in satellite imagery ranges between $0.0$ and $0.4$. Standard image viewers (Windows Photos, web browsers) cannot handle 128-bit floating-point rasters and display them as pitch black or washed-out gray.

### Issue 2B: Why there was "literal no change in clarity"
1. **The Copernicus API Fetching Mismatch (`src-tauri/src/fetch_image.rs`):**
   - The bounding box in `fetch_rgbn_geotiff` was hardcoded to `half_width_m = 640.0` ($1280\text{ meters}$ total width).
   - However, the code requested `width: 1024, height: 1024` from Copernicus Sentinel Hub.
   - **Sentinel-2 optical bands (B04, B03, B02, B08) have a physical resolution of $10\text{ meters per pixel}$.**
   - A $1280\text{m}$ area on the ground contains only **$128 \times 128$ true satellite pixels**.
   - Because $1024 \times 1024$ was requested, Copernicus **bilinearly interpolated** each $10\text{m}$ pixel into an $8 \times 8$ block of pixels ($1.25\text{m/pixel}$).
   - **The input `input_RGBN.tiff` was already an $8\times$ digitally upscaled, smoothed image before the AI model ever saw it.**
2. **Model Receptive Field & HardConstraint:**
   - `SEN2SR` was trained on raw $10\text{m}$ satellite pixels. Giving it an already-interpolated image meant the model's $3 \times 3$ CNN filters only saw smooth slopes instead of raw pixel edges.
   - Additionally, `SEN2SR`'s `HardConstraint` low-pass filter enforces that the downscaled output matches the low-resolution input, which locked in the bilinear blur.

---

## 3. Code Modifications Applied

### A. Fixed Resolution Scale in `src-tauri/src/fetch_image.rs`
Changed the hardcoded `half_width_m = 640.0` to calculate bounding box width dynamically based on native $10\text{m/pixel}$:
```rust
// Sentinel-2 L2A optical bands (B04, B03, B02, B08) have a native 10m/pixel spatial resolution.
// Ensure the ground bounding box matches the requested pixel matrix so Copernicus returns
// true 1:1 raw satellite sensor pixels without artificial pre-interpolation blur:
let width_m = (resolution as f64) * 10.0;
let half_width_m = width_m / 2.0;
let lat_offset = half_width_m / 111_320.0;
let lon_offset = half_width_m / (111_320.0 * lat.to_radians().cos());
```
* If `resolution = 128`: width is $1280\text{m}$ ($10\text{m/pixel}$).
* If `resolution = 1024`: width is $10240\text{m}$ ($10.24\text{km}$) ($10\text{m/pixel}$).
Copernicus now returns **genuine 1:1 native sensor data** with zero pre-blurring.

---

### B. Overhauled `run_model.py`
1. **Lossless GeoTIFF Compression (`DEFLATE` + `Predictor=2` + `uint16`):**
   - Output GeoTIFF uses standard Sentinel-2 `uint16` surface reflectance ($0 - 10,000$).
   - GeoTIFF compression shrank the file size from **$256.03\text{ MB} \rightarrow 57.08\text{ MB}$** losslessly.
2. **Radiometric Percentile Contrast Stretch (`stretch_percentile_rgb`):**
   - Implemented standard satellite $2\% - 98\%$ percentile histogram stretching with gamma correction ($0.9$).
   - Converts raw floating-point reflectance into crisp, vibrant 8-bit RGB color.
3. **Automatic Visual PNG Export:**
   - Automatically generates `output_super_res_visual.png` ($4096 \times 4096$, $11.79\text{ MB}$) that opens directly in Windows Photos, macOS Preview, and browsers.
4. **Side-by-Side Quality Comparison Generator:**
   - Automatically creates `comparison_before_after.png` showing:
     1. *Input Sentinel-2 (Raw Pixels, nearest neighbor)*
     2. *Standard Bicubic Upsample (No AI baseline)*
     3. *SEN2SR AI Enhanced ($2.5\text{m}$ Super-Resolution)*

---

## 4. How to Run the Pipeline

### Command
```powershell
python run_model.py
```

### Optional Arguments
```powershell
python run_model.py --input input_RGBN.tiff --output output_super_res.tiff --visual output_super_res_visual.png --comparison comparison_before_after.png --dtype uint16
```

---

## 5. Output Artifacts Summary

| File | Type | Purpose | Size |
|---|---|---|---|
| `output_super_res_visual.png` | 8-bit RGB PNG | Full $4096 \times 4096$ enhanced visual image for viewing | ~11.8 MB |
| `comparison_before_after.png` | 8-bit RGB PNG | Visual showcase: Raw vs. Bicubic vs. SEN2SR AI | ~1.8 MB |
| `output_super_res.tiff` | GeoTIFF (uint16) | Full 4-band GIS-ready super-resolved GeoTIFF | ~57.1 MB |
| `input_RGBN.tiff` | GeoTIFF | Raw input GeoTIFF fetched from Copernicus | ~850 KB |
