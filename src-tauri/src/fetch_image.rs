use reqwest::Client;
use serde_json::json;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Saves binary content to the current directory and also mirrors it to the project root
/// if running inside `src-tauri`.
fn save_file(filename: &str, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    file.write_all(bytes)?;

    // Mirror to parent directory if running inside src-tauri (so python scripts in root find it)
    let parent_path = Path::new("..").join(filename);
    if Path::new("../package.json").exists() {
        if let Ok(mut parent_file) = File::create(&parent_path) {
            let _ = parent_file.write_all(bytes);
        }
    }
    Ok(())
}

/// Fetches a unified 4-band GeoTIFF containing [B04 (Red), B03 (Green), B02 (Blue), B08 (NIR)]
/// in a single request from the Copernicus Sentinel Hub Process API.
/// This matches the exact channel order and FLOAT32 reflectance format required by the SEN2SR model.
pub async fn fetch_rgbn_geotiff(
    client: &Client,
    token: &str,
    lat: f64,
    lon: f64,
    resolution: u32,
) -> Result<String, Box<dyn Error>> {
    // Sentinel-2 L2A optical bands (B04, B03, B02, B08) have a native 10m/pixel spatial resolution.
    // Ensure the ground bounding box matches the requested pixel matrix so Copernicus returns
    // true 1:1 raw satellite sensor pixels without artificial pre-interpolation blur.
    let width_m = (resolution as f64) * 10.0;
    let half_width_m = width_m / 2.0;
    let lat_offset = half_width_m / 111_320.0;
    let lon_offset = half_width_m / (111_320.0 * lat.to_radians().cos());

    let min_lon = lon - lon_offset;
    let min_lat = lat - lat_offset;
    let max_lon = lon + lon_offset;
    let max_lat = lat + lat_offset;

    // Returns a 4-band FLOAT32 GeoTIFF: Band 1=B04, Band 2=B03, Band 3=B02, Band 4=B08
    let evalscript = r#"//VERSION=3
function setup() {
    return {
        input: ["B04", "B03", "B02", "B08"],
        output: {
            id: "default",
            bands: 4,
            sampleType: "FLOAT32"
        }
    };
}
function evaluatePixel(sample) {
    return [sample.B04, sample.B03, sample.B02, sample.B08];
}
"#;

    let payload = json!({
        "input": {
            "bounds": {
                "bbox": [min_lon, min_lat, max_lon, max_lat]
            },
            "data": [{
                "type": "sentinel-2-l2a",
                "dataFilter": {
                    "timeRange": { "from": "2024-04-01T00:00:00Z", "to": "2024-04-30T23:59:59Z" },
                    "maxCloudCoverage": 20,
                    "mosaickingOrder": "leastCC"
                }
            }]
        },
        "output": {
            "width": resolution,
            "height": resolution,
            "responses": [{
                "identifier": "default",
                "format": { "type": "image/tiff" }
            }]
        },
        "evalscript": evalscript
    });

    let url = "https://sh.dataspace.copernicus.eu/api/v1/process";

    println!("Fetching 4-band RGBN GeoTIFF ({}x{})...", resolution, resolution);
    let res = client
        .post(url)
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await?;

    if res.status().is_success() {
        let bytes = res.bytes().await?;
        let filename = "input_RGBN.tiff";
        save_file(filename, &bytes)?;

        println!("Saved 4-band GeoTIFF -> {}", filename);
        Ok(filename.to_string())
    } else {
        let error_msg = res.text().await?;
        Err(format!("Failed to fetch 4-band GeoTIFF: {}", error_msg).into())
    }
}

pub async fn fetch_single_band(
    client: &Client,
    token: &str,
    lat: f64,
    lon: f64,
    band: &str,
    resolution: u32,
) -> Result<(), Box<dyn Error>> {
    // Native 10m/pixel ground extent
    let width_m = (resolution as f64) * 10.0;
    let half_width_m = width_m / 2.0;
    let lat_offset = half_width_m / 111_320.0;
    let lon_offset = half_width_m / (111_320.0 * lat.to_radians().cos());

    let min_lon = lon - lon_offset;
    let min_lat = lat - lat_offset;
    let max_lon = lon + lon_offset;
    let max_lat = lat + lat_offset;

    let evalscript = format!(
        r#"
//VERSION=3
function setup() {{
    return {{
        input: ["{band}"],
        output: {{ bands: 1, sampleType: "FLOAT32" }}
    }};
}}
function evaluatePixel(sample) {{
    return [sample.{band}];
}}
"#,
        band = band
    );

    let payload = json!({
        "input": {
            "bounds": {
                "bbox": [min_lon, min_lat, max_lon, max_lat]
            },
            "data": [{
                "type": "sentinel-2-l2a",
                "dataFilter": {
                    "timeRange": { "from": "2024-04-01T00:00:00Z", "to": "2024-04-30T23:59:59Z" },
                    "maxCloudCoverage": 20,
                    "mosaickingOrder": "leastCC"
                }
            }]
        },
        "output": {
            "width": resolution,
            "height": resolution,
            "responses": [{
                "identifier": "default",
                "format": { "type": "image/tiff" }
            }]
        },
        "evalscript": evalscript
    });

    let url = "https://sh.dataspace.copernicus.eu/api/v1/process";

    println!("Fetching {}...", band);
    let res = client
        .post(url)
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await?;

    if res.status().is_success() {
        let bytes = res.bytes().await?;
        let filename = format!("{}.tiff", band);
        save_file(&filename, &bytes)?;

        println!("Saved -> {}", filename);
        Ok(())
    } else {
        let error_msg = res.text().await?;
        Err(format!("Failed to fetch {}: {}", band, error_msg).into())
    }
}

pub async fn download_all_bands(
    token: &str,
    lat: f64,
    lon: f64,
    resolution: u32,
) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let bands_needed = vec!["B04", "B03", "B02", "B08"];

    println!("Starting data pipeline for coords: [{}, {}]", lat, lon);

    let mut errors = Vec::new();
    for band in bands_needed {
        if let Err(e) = fetch_single_band(&client, token, lat, lon, band, resolution).await {
            eprintln!("Error fetching {}: {}", band, e);
            errors.push(format!("{}: {}", band, e));
        }
    }

    if !errors.is_empty() {
        return Err(format!("Error downloading bands: {}", errors.join("; ")).into());
    }

    println!("All bands downloaded successfully!");
    Ok(())
}

#[tauri::command]
pub async fn fetch_sentinel_patch(
    token: Option<String>,
    lat: f64,
    lon: Option<f64>,
    lng: Option<f64>,
    resolution: Option<u32>,
) -> Result<String, String> {
    let auth_token = match token {
        Some(t) if !t.trim().is_empty() => t,
        _ => crate::bearer_token_ops::get_bearer_token()?,
    };

    let target_lon = lon
        .or(lng)
        .ok_or_else(|| "Longitude (lon or lng) is required".to_string())?;

    let res = resolution.unwrap_or(1024);
    let client = Client::new();

    // Primary: fetch complete 4-band RGBN GeoTIFF in a single Copernicus API request
    fetch_rgbn_geotiff(&client, &auth_token, lat, target_lon, res)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "4-band RGBN GeoTIFF ({}x{}) downloaded successfully as input_RGBN.tiff",
        res, res
    ))
}
