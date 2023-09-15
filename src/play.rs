use crate::appkey::get_appkey;
use crate::colors::*;
use crate::manage::*;
use crate::types::*;
use crate::utils::*;
use crate::vpn::*;
use std::env;
use std::io::{self,Write};
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use tokio::spawn;

#[derive(Serialize)]
struct ReviewData {
    id: u64,
    stars: u32,
    headline: String,
    review: String,
}

#[derive(Serialize)]
struct FlagData {
    flag: String,
    id: u64,
    difficulty: String,
}

pub async fn play_machine(machine_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let appkey = get_appkey();
    let appkey_clone = appkey.clone(); // Clone the necessary data to avoid borrowed value error
    let htb_path = format!("{}/.htb.conf", env::var("HOME").unwrap());
    let htbconfig = HTBConfig::get_current_config(&htb_path);

    let mut machine_info = PlayingMachine::get_machine(machine_name, &appkey).await;

    println!("Stopping any active machine...");
    stop_machine().await;
    
    check_vpn(machine_info.sp_flag).await;    

    let blocking_task = spawn(async move {
        //For SP Machines and VIP VPN (not Free VPN)
        let client = Client::new();
        let response_result = client
            .post("https://www.hackthebox.com/api/v4/vm/spawn")
            .json(&json!({
                "machine_id": machine_info.machine.id
            }))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", appkey))
            .send()
            .await;
        
        match response_result {
            Ok(response) => {
                let response_status = response.status();
                if response_status.is_success() {
                    let message = response.text().await.expect("Failed to get response text");
                    println!("{}{}{}", BGREEN, message, RESET);
                } else if response_status.as_u16() == 400 {
                    // For Free VPN
                    let post_req = format!("https://www.hackthebox.com/api/v4/machine/play/{}", machine_info.machine.id);
                    let response_play = client
                        .post(post_req)
                        .header("Authorization", format!("Bearer {}", appkey))
                        .send()
                        .await;
                    
                    match response_play {
                        Ok(subresponse) => {
                            let subresponse_status = subresponse.status();
                            if subresponse_status.is_success() {
                                let subresponse_text = subresponse.text().await.expect("Failed to get response text");
                                println!("Response: {}", subresponse_text);
                            } else {
                                let subresponse_text = subresponse.text().await.expect("Failed to get response text");
                                println!("Response: {}", subresponse_text);
                                eprintln!("Request failed with status: {}", subresponse_status);
                                std::process::exit(1);
                            }
                        }
                        Err(err) => {
                            eprintln!("Error on POST request: {:?}", err);
                        }
                    }
                } else {
                    let response_text = response.text().await.expect("Failed to get response text");
                    println!("Response: {}", response_text);
                    eprintln!("Request failed with status: {}", response_status);
                    std::process::exit(1);
                }
            }
            Err(err) => {
                eprintln!("Error on POST request: {:?}", err);
            }
        }
    });

    // Await the result of the blocking task
    blocking_task.await.expect("Blocking task failed");

    if machine_info.ip.is_empty() { //Starting Point case because SP IP address is assigned only after spawn of the machine
        machine_info.ip = get_ip(&appkey_clone).await;
    }

    let mut user_info = PlayingUser::get_playinguser(&appkey_clone).await; // Before this it is needed to run HTB VPN to take the Attacker IP address

    let _ = print_banner();

    if machine_info.machine.user_pwn != "null" {
        println!("{}Hey! You have already found the User Flag! Nice one!{}", BGREEN, RESET);
    }

    if machine_info.machine.root_pwn != "null" {
        println!("{}Hey! You have already found the Root Flag! Keep it up!{}", BGREEN, RESET);
    }

    if htbconfig.promptchange { //If the prompt is set to change during the playing...
        change_shell(&mut machine_info, &mut user_info);
    }

    // Writing /etc/hosts
    let _ = add_hosts(&machine_info);

    display_target_info(&machine_info, &user_info);

    Ok(())
}

pub async fn submit_flag() {
    let appkey = get_appkey();
    let appkey_clone = appkey.clone(); // Clone the necessary data to avoid borrowed value error
    let mut flag = String::new();
    let mut machine_name = String::new();
    let mut machine_rating = String::new();

    println!("{}Did you get a flag? Please, submit it and continue your hacking path. Good Luck!{}", BGREEN, RESET);

    print!("{}Submit the flag:{} ", RED, RESET);
    io::stdout().flush().expect("Flush failed!");
    io::stdin()
        .read_line(&mut flag)
        .expect("Failed to read line");

    print!("{}Specify the machine name:{} ", BGREEN, RESET);
    io::stdout().flush().expect("Flush failed!");
    io::stdin()
        .read_line(&mut machine_name)
        .expect("Failed to read line");

    let machine_info = PlayingMachine::get_machine(&machine_name, &appkey).await;

    println!("{}How much is the difficulty of this machine?{}", BCYAN, RESET);
    println!();
    println!("{}[10 --> Piece of Cake!]{}", BGREEN, RESET);
    println!("{}[20 --> Very Easy!]{}", BGREEN, RESET);
    println!("{}[30 --> Easy!]{}", BGREEN, RESET);
    println!("{}[40 --> Not Too Easy!]{}", BYELLOW, RESET);
    println!("{}[50 --> Medium!]{}", BYELLOW, RESET);
    println!("{}[60 --> A Bit Hard!]{}", BYELLOW, RESET);
    println!("{}[70 --> Hard!]{}", BYELLOW, RESET);
    println!("{}[80 --> Too Hard!]{}", RED, RESET);
    println!("{}[90 --> Extremely Hard!]{}", RED, RESET);
    println!("{}[100 --> Brainfuck!]{}", RED, RESET);
    println!();

    
    print!("{}Difficulty Rating:{} ", BYELLOW, RESET);
    io::stdout().flush().expect("Flush failed!");
    io::stdin()
        .read_line(&mut machine_rating)
        .expect("Failed to read line");

    let blocking_task = spawn(async move {
        let flag_data = FlagData {
            flag: flag.trim().to_string(),
            id: machine_info.machine.id,
            difficulty: machine_rating.trim().to_string(),
        };

        // Create a reqwest client
        let client = Client::new();

        // Send the POST request to submit the flag
        let response = client
            .post("https://www.hackthebox.com/api/v4/machine/own")
            .json(&flag_data)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", appkey))
            .send();
  
        match response.await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    // Print the response text
                    let response_text = resp.text().await.expect("Failed to read response text");
                    println!("{}", response_text);
                } else {
                    // Status 400: Incorrect flag
                    let response_text = resp.text().await.expect("Failed to read response text");
                    let parsed_response: Result<serde_json::Value, _> = serde_json::from_str(&response_text);

                    if let Ok(json) = parsed_response {
                        if let Some(message) = json.get("message").and_then(|m| m.as_str()) {
                            println!("{}", message);
                        }
                    } else {
                        eprintln!("Request failed with status: {}", status);
                        std::process::exit(1);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error sending request: {}", err);
            }
        }
    });

    // Await the result of the blocking task
    blocking_task.await.expect("Blocking task failed");

    if machine_info.machine.user_pwn != "null" && machine_info.machine.root_pwn != "null" && machine_info.review {
        println!("{}Wonderful! You PWNED {}! Would you like to submit feedback?{}", BYELLOW, machine_info.machine.name, RESET);

        loop {
            println!("Select an option:");
            println!("1. Yes");
            println!("2. No");
    
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read input");
            let choice = input.trim();
    
            match choice {
                "1" => {
                    println!("{}Please, write a headline for your feedback (max 50 chars):{}", BGREEN, RESET);
                    let mut review_headline = String::new();
                    io::stdin().read_line(&mut review_headline).expect("Failed to read input");
    
                    println!("{}Please, submit your feedback (max 2000 chars):{}", BGREEN, RESET);
                    let mut review_machine = String::new();
                    io::stdin().read_line(&mut review_machine).expect("Failed to read input");
    
                    println!("{}How many stars would you give to this machine?{}", BGREEN, RESET);
                    print!("Stars (1 to 5): ");
    
                    let mut review_stars = String::new();
                    io::stdout().flush().expect("Flush failed!");
                    io::stdin().read_line(&mut review_stars).expect("Failed to read input");
                    let review_stars = review_stars.trim().parse::<u32>().unwrap_or(0);
    
                    let review_data = ReviewData {
                        id: machine_info.machine.id,
                        stars: review_stars,
                        headline: review_headline.trim().to_string(),
                        review: review_machine.trim().to_string(),
                    };
    
                    // Create a reqwest client
                    let client = Client::new();
    
                    // Send the POST request
                    let response = client
                        .post("https://www.hackthebox.com/api/v4/machine/review")
                        .json(&review_data)
                        .header("Content-Type", "application/json")
                        .header("Authorization", format!("Bearer {}", appkey_clone))
                        .send()
                        .await
                        .expect("Failed to send request");
    
                    if response.status().is_success() {
                        let message: serde_json::Value = response.json().await.expect("Failed to parse JSON");
                        println!("{}", message["message"]);
                    } else {
                        println!("Request failed with status: {}", response.status());
                        std::process::exit(1);
                    }
    
                    break;
                }
                "2" => {
                    break;
                }
                _ => {
                    println!("Invalid choice. Please select a valid option.");
                }
            }
        }
    }
}
