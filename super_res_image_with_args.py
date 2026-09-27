import os
import sys
import time
import argparse
import numpy as np
import rasterio
from rasterio.transform import Affine
import torch
import torch.nn.functional as F
import mlstac
import sen2sr

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", "-i", required=True)
    parser.add_argument("--output", "-o", required=True)
    parser.add_argument("--dtype", choices=["uint16", "float32"], default="uint16")
    args = parser.parse_args()

    t_start = time.time()
    print(f"[super_res] Starting super-resolution: input={args.input}, output={args.output}")

    if torch.cuda.is_available():
        device = torch.device("cuda:0")
        torch.backends.cudnn.benchmark = True
        print(f"[super_res] Using GPU: {torch.cuda.get_device_name(0)}")
    else:
        device = torch.device("cpu")
        print("[super_res] CUDA not available, using CPU")

    model_dir = "model/SEN2SRLite_RGBN"
    if not os.path.exists(model_dir) and os.path.exists(os.path.join("..", model_dir)):
        model_dir = os.path.join("..", model_dir)

    mlm_json = os.path.join(model_dir, "mlm.json")
    if not os.path.exists(mlm_json):
        print(f"[super_res] Local model not found, downloading to {model_dir}...")
        model_url = "https://huggingface.co/tacofoundation/sen2sr/resolve/main/SEN2SRLite/NonReference_RGBN_x4/mlm.json"
        mlstac.download(file=model_url, output_dir=model_dir)
    else:
        print(f"[super_res] Loading cached model offline from {model_dir}")

    t_model = time.time()
    model = mlstac.load(model_dir).compiled_model(device=device)
    print(f"[super_res] Model loaded in {time.time() - t_model:.2f}s")

    input_path = args.input
    if not os.path.exists(input_path) and os.path.exists(os.path.join("..", input_path)):
        input_path = os.path.join("..", input_path)

    if not os.path.exists(input_path):
        raise FileNotFoundError(f"Input file not found: {args.input}")

    print(f"[super_res] Reading input: {input_path}")
    with rasterio.open(input_path) as src:
        img = src.read()
        meta = src.meta.copy()
        transform = src.transform

    channels, height, width = img.shape
    print(f"[super_res] Dimensions: {width}x{height}, Bands: {channels}, Dtype: {img.dtype}")

    if img.max() > 1.0:
        norm_img = (img / 10000.0).astype(np.float32)
    else:
        norm_img = img.astype(np.float32)

    X = torch.from_numpy(norm_img).float().to(device)
    X = torch.nan_to_num(X, nan=0.0, posinf=0.0, neginf=0.0)

    scale_factor = 4
    print("[super_res] Running inference on GPU...")
    t_infer = time.time()

    with torch.no_grad():
        if height <= 128 and width <= 128:
            if height < 128 or width < 128:
                X_pad = F.pad(X, (0, 128 - width, 0, 128 - height), mode="reflect")
                superX = model(X_pad.unsqueeze(0)).squeeze(0)
                superX = superX[:, :height * scale_factor, :width * scale_factor]
            else:
                superX = model(X.unsqueeze(0)).squeeze(0)
        else:
            superX = sen2sr.predict_large(model=model, X=X, overlap=32)

    sr_img = superX.cpu().numpy()
    _, new_height, new_width = sr_img.shape
    print(f"[super_res] Inference completed in {time.time() - t_infer:.2f}s, output: {new_width}x{new_height}")

    new_transform = Affine(
        transform.a / scale_factor, transform.b, transform.c,
        transform.d, transform.e / scale_factor, transform.f
    )

    if args.dtype == "uint16":
        out_data = np.clip(sr_img * 10000.0, 0, 65535).astype(np.uint16)
        out_dtype = "uint16"
    else:
        out_data = np.clip(sr_img, 0.0, 1.0).astype(np.float32)
        out_dtype = "float32"

    meta.pop("compress", None)
    meta.pop("interleave", None)
    meta.update({
        "driver": "GTiff",
        "height": new_height,
        "width": new_width,
        "transform": new_transform,
        "dtype": out_dtype,
        "compress": "deflate",
        "predictor": 2,
        "tiled": True,
        "blockxsize": 256,
        "blockysize": 256,
        "interleave": "band",
    })

    print(f"[super_res] Saving GeoTIFF to {args.output}")
    with rasterio.open(args.output, "w", **meta) as dst:
        dst.write(out_data)

    print(f"[super_res] Completed successfully in {time.time() - t_start:.2f}s")

if __name__ == "__main__":
    main()
