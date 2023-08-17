use reqwest::blocking::Client;
use core::time::Duration;
use serde_json;

pub fn fetch_api(api_url: &str, appkey: &str) -> Result<serde_json::Value, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .unwrap();

    let response = client.get(api_url)
        .header("Authorization", format!("Bearer {}", appkey))
        .header("content-type", "application/json" )
        .send()
        .unwrap()
        .text()
        .unwrap();

    let json_data = serde_json::from_str(&response).expect("JSON Parsing Error");

    Ok(json_data)
}