use reqwest::blocking::Client;
use core::time::Duration;


//fetch_api checks also if the App Token is valid or not
pub fn fetch_api(api_url: &str, appkey: &str) -> Result<serde_json::Value, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;

    //let client = Client::new();

    let response = client.get(api_url)
        .header("Authorization", format!("Bearer {}", appkey))
        .header("content-type", "application/json" )
        .send()?;

    let status_code  = response.status().as_u16();
    let response_text = response.text()?;

    /*println!("Call: {}", api_url); // DEBUG
    println!("Response Body: {:?}", response_text); // DEBUG*/
    
    if status_code == 429 {
        // Handle 429 Too Many Requests
        eprintln!("HTTP 429: Too many requests. Please wait and try again later.");
        std::process::exit(1);
    } else if (500..600).contains(&status_code) {
        // Handle 5xx Server Errors
        eprintln!("HTTP {}: Server error. Please try again later.", status_code);
        std::process::exit(1);
    }

    let json_data = match serde_json::from_str(&response_text) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Check if your API key is incorrect or expired. Renew your API key by running 'htb-toolkit -k reset'. Error details: {}", err);
            std::process::exit(1);
        }
    };

    // Accessing a specific field from the JSON data //DEBUG
    /*if let Some(field_value) = json_data.get("info") {
        println!("Field Value: {:?}", field_value);
    }
    std::process::exit(1);*/

    Ok(json_data)
}