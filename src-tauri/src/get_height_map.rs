use reqwest::Client;
use serde_json::json;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn save_file(filename: &str, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(filename)?;
    file.write_all(bytes)?;

    let parent_path = Path::new("..").join(filename);
    if Path::new("../package.json").exists() {
        if let Ok(mut parent_file) = File::create(&parent_path) {
            let _ = parent_file.write_all(bytes);
        }
    }
    Ok(())
}

pub async fn fetch_dem(
    client: &Client,
    token: &str,
    lat: f64,
    lon: f64,
    resolution: u32,
) -> Result<String, Box<dyn Error>> {
    let width_m = (resolution as f64) * 10.0;
    let half_width_m = width_m / 2.0;
    let lat_offset = half_width_m / 111_320.0;
    let lon_offset = half_width_m / (111_320.0 * lat.to_radians().cos());

    let min_lon = lon - lon_offset;
    let min_lat = lat - lat_offset;
    let max_lon = lon + lon_offset;
    let max_lat = lat + lat_offset;

    let evalscript = r#"//VERSION=3
function setup() {
    return {
        input: ["DEM"],
        output: {
            id: "default",
            bands: 1,
            sampleType: "FLOAT32"
        }
    };
}
function evaluatePixel(sample) {
    return [sample.DEM];
}
"#;

    let payload = json!({
        "input": {
            "bounds": {
                "bbox": [min_lon, min_lat, max_lon, max_lat]
            },
            "data": [{
                "type": "dem",
                "dataFilter": {
                    "demInstance": "COPERNICUS_90"
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

    let res = client
        .post(url)
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await?;

    if res.status().is_success() {
        let bytes = res.bytes().await?;
        let filename = "height_map.tiff";
        save_file(filename, &bytes)?;
        Ok(filename.to_string())
    } else {
        let error_msg = res.text().await?;
        Err(format!("Failed to fetch DEM: {}", error_msg).into())
    }
}

#[tauri::command]
pub async fn get_height_map(
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

    fetch_dem(&client, &auth_token, lat, target_lon, res)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "Height map ({}x{}) downloaded successfully as height_map.tiff",
        res, res
    ))
}
