use reqwest::Client;
use serde_json::json;
use std::error::Error;
use std::fs::File;
use std::io::Write;

// 1. Function to fetch a single specific band
pub async fn fetch_single_band(
    client: &Client,
    token: &str,
    lat: f64,
    lon: f64,
    band: &str,
) -> Result<(), Box<dyn Error>> {
    // Calculate the 1280x1280m bounding box
    let half_width_m = 640.0;
    let lat_offset = half_width_m / 111_320.0;
    let lon_offset = half_width_m / (111_320.0 * lat.to_radians().cos());

    let min_lon = lon - lon_offset;
    let min_lat = lat - lat_offset;
    let max_lon = lon + lon_offset;
    let max_lat = lat + lat_offset;

    // 2. Dynamically inject the requested band into the evalscript
    // We use {{ and }} to escape the brackets in Rust's format! macro
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
                    // Pick a recent clear month (e.g., April 2024 for Udaipur)
                    "timeRange": { "from": "2024-04-01T00:00:00Z", "to": "2024-04-30T23:59:59Z" },
                    "maxCloudCoverage": 5
                }
            }]
        },
        "output": {
            "width": 128,
            "height": 128,
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

        // Save the file dynamically based on the band name (e.g., "B02.tiff")
        let filename = format!("{}.tiff", band);
        let mut file = File::create(&filename)?;
        file.write_all(&bytes)?;

        println!("Saved -> {}", filename);
        Ok(())
    } else {
        let error_msg = res.text().await?;
        Err(format!("Failed to fetch {}: {}", band, error_msg).into())
    }
}

// 3. The main function that loops through the required bands
pub async fn download_all_bands(token: &str, lat: f64, lon: f64) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let bands_needed = vec!["B02", "B03", "B04", "B08"];

    println!("Starting data pipeline for coords: [{}, {}]", lat, lon);

    let mut errors = Vec::new();
    for band in bands_needed {
        if let Err(e) = fetch_single_band(&client, token, lat, lon, band).await {
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

// 4. Tauri command exposed to frontend
#[tauri::command]
pub async fn fetch_sentinel_patch(
    token: Option<String>,
    lat: f64,
    lon: Option<f64>,
    lng: Option<f64>,
) -> Result<String, String> {
    let auth_token = match token {
        Some(t) if !t.trim().is_empty() => t,
        _ => crate::bearer_token_ops::get_bearer_token()?,
    };

    let target_lon = lon.or(lng).ok_or_else(|| "Longitude (lon or lng) is required".to_string())?;

    download_all_bands(&auth_token, lat, target_lon)
        .await
        .map_err(|e| e.to_string())?;

    Ok("All bands downloaded successfully (B02.tiff, B03.tiff, B04.tiff, B08.tiff)".to_string())
}
