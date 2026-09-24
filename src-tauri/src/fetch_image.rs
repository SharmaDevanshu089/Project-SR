// TODO: ye hardcoded resolution ko runtime pe sync karna
use reqwest::Client;
use serde_json::json;
use std::error::Error;
use std::fs::File;
use std::io::Write;

pub async fn fetch_sentinel_patch(token: &str, lat: f64, lon: f64) -> Result<(), Box<dyn Error>> {
    let half_width_m = 640.0;
    let lat_offset = half_width_m / 111_320.0;

    let lon_offset = half_width_m / (111_320.0 * lat.to_radians().cos());

    let min_lon = lon - lon_offset;
    let min_lat = lat - lat_offset;
    let max_lon = lon + lon_offset;
    let max_lat = lat + lat_offset;

    let evalscript = r#"
//VERSION=3
function setup() {
    return {
        input: ["B02", "B03", "B04", "B08"],
        output: { bands: 4, sampleType: "FLOAT32" }
    };
}
function evaluatePixel(sample) {
    return [sample.B02, sample.B03, sample.B04, sample.B08];
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

    println!("Requesting 128x128 image patch...");

    let client = Client::new();
    let res = client
        .post(url)
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await?;

    if res.status().is_success() {
        let bytes = res.bytes().await?;
        let filename = "raw_s2_patch.tiff";

        let mut file = File::create(filename)?;
        file.write_all(&bytes)?;

        println!("Success! Saved Sentinel-2 patch to {}", filename);
        Ok(())
    } else {
        let error_msg = res.text().await?;
        Err(format!("Process API failed: {}", error_msg).into())
    }
}
