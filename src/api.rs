use reqwest::blocking::Client;
use core::time::Duration;
use std::io::{self, Write};
use tokio::task;

// Define an async version of fetch_api
pub async fn fetch_api_async(api_url: &str, appkey: &str) -> Result<serde_json::Value, reqwest::Error> {
    let api_url_owned = api_url.to_owned(); // Clone the URL
    let appkey_owned = appkey.to_owned(); // Clone the appkey

    task::spawn_blocking(move || fetch_api(&api_url_owned, &appkey_owned)).await.unwrap()
}

// fetch_api checks also if the App Token is valid or not
fn fetch_api(api_url: &str, appkey: &str) -> Result<serde_json::Value, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;
    
    let response = client.get(api_url)
        .header("Authorization", format!("Bearer {appkey}"))
        .header("content-type", "application/json" )
        .send()?;

    let status_code = response.status().as_u16();
    let response_text = response.text()?;

    if status_code == 429 {
        eprintln!("HTTP 429: Too many requests. Please wait and try again later.");
        std::process::exit(1);
    } else if (500..600).contains(&status_code) {
        eprintln!("HTTP {status_code}: Server error. Please try again later.");
        std::process::exit(1);
    }

    let json_data = match serde_json::from_str(&response_text) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Check if your API key is incorrect or expired. Renew your API key by running 'htb-toolkit -k reset'. Error details: {err}");
            print!("Press Enter to continue...");
            let mut input = String::new();
            io::stdout().flush().expect("Flush failed!");
            io::stdin().read_line(&mut input).expect("Failed to read line");
            std::process::exit(1);
        }
    };

    Ok(json_data)
}