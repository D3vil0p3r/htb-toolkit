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
        .send()?;

    let response_text = response.text()?;
    //println!("Response Body: {:?}", response_text); //DEBUG

    let json_data: serde_json::Value = serde_json::from_str(&response_text).expect("JSON Parsing Error");

    // Accessing a specific field from the JSON data //DEBUG
    /*if let Some(field_value) = json_data.get("info") {
        println!("Field Value: {:?}", field_value);
    }
    std::process::exit(1);*/

    Ok(json_data)
}