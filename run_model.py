import os
import argparse
import numpy as np
import rasterio
from rasterio.transform import Affine
import torch
import torch.nn.functional as F
from PIL import Image, ImageDraw, ImageFont
import mlstac
import sen2sr


def stretch_percentile_rgb(rgb_array: np.ndarray, lower_pct: float = 2.0, upper_pct: float = 98.0) -> np.ndarray:
    """
    Applies standard satellite radiometric contrast stretching (lower to upper percentile)
    to convert raw surface reflectance [0.0 - 0.4] into an 8-bit [0 - 255] RGB visual image.
    rgb_array shape: (H, W, 3)
    """
    stretched = np.zeros_like(rgb_array, dtype=np.float32)
    for c in range(3):
        channel = rgb_array[:, :, c]
        p_low, p_high = np.percentile(channel, (lower_pct, upper_pct))
        if p_high > p_low:
            stretched[:, :, c] = np.clip((channel - p_low) / (p_high - p_low), 0.0, 1.0)
        else:
            stretched[:, :, c] = np.clip(channel, 0.0, 1.0)
            
    # Apply a gentle gamma curve (gamma=0.9) to reveal shadow details
    stretched = np.power(stretched, 0.9)
    return (stretched * 255.0).astype(np.uint8)


def main():
    parser = argparse.ArgumentParser(description="Sentinel-2 4x Super Resolution via SEN2SR")
    parser.add_argument("--input", default="input_RGBN.tiff", help="Path to input 4-band GeoTIFF")
    parser.add_argument("--output", default="output_super_res.tiff", help="Path to output super-resolved GeoTIFF")
    parser.add_argument("--visual", default="output_super_res_visual.png", help="Path to 8-bit visual RGB PNG")
    parser.add_argument("--comparison", default="comparison_before_after.png", help="Path to comparison showcase PNG")
    parser.add_argument("--dtype", choices=["uint16", "float32"], default="uint16",
                        help="Data type for output GeoTIFF (uint16 saves 4x space and is GIS-standard)")
    args = parser.parse_args()

    # 1. Setup Device & Load Model
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"Using device: {device}")

    model_dir = "model/SEN2SRLite_RGBN"
    model_url = "https://huggingface.co/tacofoundation/sen2sr/resolve/main/SEN2SRLite/NonReference_RGBN_x4/mlm.json"
    
    print("Loading SEN2SR model...")
    mlstac.download(file=model_url, output_dir=model_dir)
    model = mlstac.load(model_dir).compiled_model(device=device)

    # 2. Read the Local GeoTIFF
    input_file = args.input
    output_file = args.output
    
    if not os.path.exists(input_file):
        raise FileNotFoundError(f"Input file not found: {input_file}")

    print(f"Reading input image: {input_file}")
    with rasterio.open(input_file) as src:
        img = src.read()  # Shape: [Bands, H, W]
        meta = src.meta.copy()
        transform = src.transform
        crs = src.crs

    channels, height, width = img.shape
    print(f"Input image dimensions: {width}x{height}, Channels: {channels}, Dtype: {img.dtype}")

    # Inspect ground resolution in meters
    lat_center = transform.f
    m_per_deg_lon = 111_320.0 * np.cos(np.radians(lat_center))
    m_per_deg_lat = 111_320.0
    pixel_size_x = abs(transform.a) * m_per_deg_lon
    pixel_size_y = abs(transform.e) * m_per_deg_lat
    print(f"Ground pixel resolution: {pixel_size_x:.2f}m x {pixel_size_y:.2f}m")

    # 3. Normalize Reflectance to [0.0, 1.0]
    if img.max() > 1.0:
        norm_img = (img / 10_000.0).astype("float32")
    else:
        norm_img = img.astype("float32")
        
    X = torch.from_numpy(norm_img).float().to(device)
    X = torch.nan_to_num(X, nan=0.0, posinf=0.0, neginf=0.0)

    # 4. Super-Resolution Inference
    scale_factor = 4
    print("Applying 4x super-resolution inference...")

    with torch.no_grad():
        if height <= 128 and width <= 128:
            # Single patch direct inference
            if height < 128 or width < 128:
                X_pad = F.pad(X, (0, 128 - width, 0, 128 - height), mode="reflect")
                superX = model(X_pad.unsqueeze(0)).squeeze(0)
                superX = superX[:, :height * scale_factor, :width * scale_factor]
            else:
                superX = model(X.unsqueeze(0)).squeeze(0)
        else:
            # Multi-tile overlapping chunking inference
            superX = sen2sr.predict_large(
                model=model,
                X=X,
                overlap=32  # 32px overlap ensures clean boundary transitions
            )
    
    sr_img = superX.cpu().numpy()
    _, new_height, new_width = sr_img.shape
    print(f"Super-resolved dimensions: {new_width}x{new_height}")

    # 5. Format & Compress Output GeoTIFF (Fixing 128-bit / 250MB uncompressed issue)
    new_transform = Affine(
        transform.a / scale_factor, transform.b, transform.c,
        transform.d, transform.e / scale_factor, transform.f
    )

    if args.dtype == "uint16":
        # Standard Sentinel-2 L2A surface reflectance integer scale (0 - 10,000)
        out_data = np.clip(sr_img * 10_000.0, 0, 65535).astype(np.uint16)
        out_dtype = "uint16"
    else:
        out_data = np.clip(sr_img, 0.0, 1.0).astype(np.float32)
        out_dtype = "float32"

    meta.update({
        "driver": "GTiff",
        "height": new_height,
        "width": new_width,
        "transform": new_transform,
        "dtype": out_dtype,
        "compress": "deflate",     # Lossless DEFLATE compression shrinks file size by 5x-10x
        "predictor": 2,            # Horizontal differencing predictor optimizes raster compression
        "tiled": True,             # Tiling for high-performance GIS viewport loading
        "blockxsize": 256,
        "blockysize": 256,
    })

    print(f"Writing compressed {out_dtype} GeoTIFF to: {output_file}")
    with rasterio.open(output_file, "w", **meta) as dst:
        dst.write(out_data)

    file_size_mb = os.path.getsize(output_file) / (1024 * 1024)
    print(f"Successfully saved {output_file} ({file_size_mb:.2f} MB)")

    # 6. Generate 8-Bit True-Color RGB Visual Product (.png)
    # Band mapping: Band 0 = B04 (Red), Band 1 = B03 (Green), Band 2 = B02 (Blue)
    print("Generating 8-bit true-color visual RGB preview...")
    sr_rgb = sr_img[:3].transpose(1, 2, 0)
    sr_rgb_stretched = stretch_percentile_rgb(sr_rgb)
    
    visual_img = Image.fromarray(sr_rgb_stretched)
    visual_img.save(args.visual, optimize=True)
    visual_size_mb = os.path.getsize(args.visual) / (1024 * 1024)
    print(f"Visual RGB preview saved to: {args.visual} ({visual_size_mb:.2f} MB)")

    # 7. Generate Side-by-Side Comparison Showcase
    print("Generating side-by-side comparison showcase...")
    # Extract center 128x128 crop from input and corresponding 512x512 from output
    crop_h = min(128, height)
    crop_w = min(128, width)
    start_y = (height - crop_h) // 2
    start_x = (width - crop_w) // 2
    
    in_crop = norm_img[:3, start_y:start_y + crop_h, start_x:start_x + crop_w]
    sr_crop = sr_img[:3, start_y * scale_factor:(start_y + crop_h) * scale_factor,
                         start_x * scale_factor:(start_x + crop_w) * scale_factor]

    # Naive Bicubic interpolation for fair baseline comparison
    in_crop_tensor = torch.from_numpy(in_crop).unsqueeze(0)
    bicubic_crop = F.interpolate(
        in_crop_tensor,
        size=(crop_h * scale_factor, crop_w * scale_factor),
        mode="bicubic",
        antialias=True
    ).squeeze(0).numpy()

    # Visual stretch
    comp_size = 512
    vis_nearest = Image.fromarray(
        stretch_percentile_rgb(in_crop.transpose(1, 2, 0))
    ).resize((comp_size, comp_size), Image.NEAREST)

    vis_bicubic = Image.fromarray(
        stretch_percentile_rgb(bicubic_crop.transpose(1, 2, 0))
    ).resize((comp_size, comp_size), Image.BILINEAR)

    vis_sr = Image.fromarray(
        stretch_percentile_rgb(sr_crop.transpose(1, 2, 0))
    ).resize((comp_size, comp_size), Image.BILINEAR)

    comparison_canvas = Image.new("RGB", (comp_size * 3, comp_size))
    comparison_canvas.paste(vis_nearest, (0, 0))
    comparison_canvas.paste(vis_bicubic, (comp_size, 0))
    comparison_canvas.paste(vis_sr, (comp_size * 2, 0))

    draw = ImageDraw.Draw(comparison_canvas)
    draw.text((15, 15), "1. Input Sentinel-2 (Raw Pixels)", fill=(255, 255, 0))
    draw.text((comp_size + 15, 15), "2. Standard Bicubic Upsample (No AI)", fill=(255, 255, 0))
    draw.text((comp_size * 2 + 15, 15), "3. SEN2SR AI Enhanced (2.5m Super-Res)", fill=(255, 255, 0))

    comparison_canvas.save(args.comparison, optimize=True)
    print(f"Comparison showcase saved to: {args.comparison}")
    print("\nProcessing complete! You can open:")
    print(f" - {args.visual} (High-res 8-bit RGB preview)")
    print(f" - {args.comparison} (Side-by-side quality comparison)")
    print(f" - {args.output} (Full GIS-ready compressed GeoTIFF)")


if __name__ == "__main__":
    main()