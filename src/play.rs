use crate::appkey::get_appkey;
use crate::colors::*;
use crate::types::*;
use crate::utils::change_shell;
use crate::utils::display_target_info;
use std::io;
use std::io::Write;
use reqwest::blocking::Client;
use serde::Serialize;

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

pub fn play_machine(machine_name: &str) {
    let appkey = get_appkey();

    let mut machine_info = PlayingMachine::get_machine(machine_name, &appkey);
    let mut user_info = PlayingUser::get_user(&appkey);

    display_target_info(&machine_info, &user_info);
    //change_shell(&mut machine_info, &mut user_info);
}

pub fn submit_flag() {
    let appkey = get_appkey();
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

    let machine_info = PlayingMachine::get_machine(&machine_name, &appkey);

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

    let flag_data = FlagData {
        flag: flag.trim().to_string(),
        id: machine_info.id,
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
  
    match response {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                // Print the response text
                let response_text = resp.text().expect("Failed to read response text");
                println!("{}", response_text);
            } else {
                // Status 400: Incorrect flag
                let response_text = resp.text().expect("Failed to read response text");
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

    if machine_info.user_pwn != "null" && machine_info.root_pwn != "null" && machine_info.review == true {
        println!("{}Wonderful! You PWNED {}! Would you like to submit feedback?{}", BYELLOW, machine_info.name, RESET);

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
                    println!("Stars (1 to 5):");
    
                    let mut review_stars = String::new();
                    io::stdin().read_line(&mut review_stars).expect("Failed to read input");
                    let review_stars = review_stars.trim().parse::<u32>().unwrap_or(0);
    
                    let review_data = ReviewData {
                        id: machine_info.id,
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
                        .header("Authorization", format!("Bearer {}", appkey))
                        .send()
                        .expect("Failed to send request");
    
                    if response.status().is_success() {
                        let message: serde_json::Value = response.json().expect("Failed to parse JSON");
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
