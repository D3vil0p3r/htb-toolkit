use crate::appkey::get_appkey;
use crate::colors::*;
use crate::manage::*;
use crate::types::*;
use crate::utils::*;
use crate::vpn::*;
use std::env;
use std::fs;
use std::io::{self,Write};
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::json;

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

pub fn play_machine(machine_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let appkey = get_appkey();
    let htb_path = format!("{}/.htb.conf", env::var("HOME").unwrap());
    let htbconfig = HTBConfig::get_current_config(&htb_path);

    let mut machine_info = PlayingMachine::get_machine(machine_name, &appkey);

    println!("Stopping any active machine...");
    stop_machine();

    check_vpn();    

    //For SP Machines and VIP VPN (not Free VPN)
    let client = Client::new();
    let response = client
        .post("https://www.hackthebox.com/api/v4/vm/spawn")
        .json(&json!({
            "machine_id": machine_info.machine.id
        }))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", appkey))
        .send()?;

    // Check if the response was successful
    let status_code = response.status();
    if status_code.is_success() {
        /*let response_text = response.text()?;
        println!("Response: {}", response_text);*/
    } else if status_code.as_u16() == 400 {
        //println!("Request failed with status: {}", status_code);
        // For Free VPN
        let client = Client::new();
        let post_req = format!("https://www.hackthebox.com/api/v4/machine/play/{}", machine_info.machine.id);
        let response_play = client
            .post(post_req)
            .header("Authorization", format!("Bearer {}", appkey))
            .send()?;
        
        // Check if the response was successful
        let status_code_play = response_play.status();
        if status_code_play.is_success() {
            let response_text = response_play.text()?;
            println!("Response: {}", response_text);
        } else {
            let response_text = response_play.text()?;
            
            println!("Response: {}", response_text);
            println!("Request failed with status: {}", status_code_play);
        }
    } else {
        let response_text = response.text()?;
        println!("Response: {}", response_text);
        println!("Request failed with status: {}", status_code);
    }

    if machine_info.ip == "" { //Starting Point case because SP IP address is assigned only after spawn of the machine
        machine_info.ip = get_ip(&appkey);
    }
    
    let mut user_info = PlayingUser::get_playinguser(&appkey); // Before this it is needed to run HTB VPN to take the Attacker IP address

    if machine_info.machine.user_pwn != "null" {
        println!("{}Hey! You have already found the User Flag! Nice one!{}", BGREEN, RESET);
    }

    if machine_info.machine.root_pwn != "null" {
        println!("{}Hey! You have already found the Root Flag! Keep it up!{}", BGREEN, RESET);
    }

    if htbconfig.promptchange == true { //If the prompt is set to change during the playing...
        change_shell(&mut machine_info, &mut user_info);
    }

    // Writing /etc/hosts
    loop {
        let mut yn = String::new();
        print!("\n{}Would you like to assign a domain name to the target machine IP address and store it in /etc/hosts? (y/n){}", BGREEN, RESET);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut yn).expect("Failed to read input");

        match yn.trim() {
            "y" | "Y" => {
                let hosts_path = std::path::Path::new("/etc/hosts");
                let domain_name = format!("{}.htb", machine_info.machine.name.split_whitespace().next().unwrap_or_default().to_string().to_lowercase()); // Using this set of func to remove the os icon after the machine name
                print!("{}Type the domain name to assign {}[{}]{}: {}", BGREEN, RED, domain_name, BGREEN, RESET);
                io::stdout().flush().unwrap();

                let mut ans = String::new();
                io::stdin().read_line(&mut ans).expect("Failed to read input");
                ans = ans.trim().to_string();

                if ans.is_empty() {
                    ans = domain_name;
                }

                if is_inside_container() {
                    let mut hosts_content = format!("{}  {}\n", machine_info.ip, ans);
                    if let Ok(existing_content) = std::fs::read_to_string(hosts_path) {
                        if !existing_content.contains(&hosts_content) {
                            hosts_content = existing_content + &hosts_content;
                        }
                    }
                    std::fs::write("/tmp/hosts.new", hosts_content).expect("Failed to write to hosts.new");
                    std::process::Command::new("sudo")
                        .args(&["cp", "-f", "/tmp/hosts.new", "/etc/hosts"])
                        .status()
                        .expect("Failed to copy hosts file");
                    std::fs::remove_file("/tmp/hosts.new").expect("Failed to remove hosts.new");
                } else {
                    // Read the current contents of the hosts file
                    let current_content = fs::read_to_string(hosts_path)?;
                    let new_entry = format!("{} {}", machine_info.ip, ans);
                    
                    // Check if the new entry already exists in the hosts file
                    if !current_content.contains(&new_entry) {
                        let sed_pattern = format!("2i{}", new_entry);
                        std::process::Command::new("sudo")
                            .args(&["sed", "-i", &sed_pattern, "/etc/hosts"])
                            .status()
                            .expect("Failed to copy hosts file");
                    }
                    else {
                        println!("Hosts file already contains the new entry.");
                    }
                }
                break;
            }
            "n" | "N" => break,
            _ => println!("Invalid answer."),
        }
    }

    display_target_info(&machine_info, &user_info);

    Ok(())
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

    if machine_info.machine.user_pwn != "null" && machine_info.machine.root_pwn != "null" && machine_info.review == true {
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
                    println!("Stars (1 to 5):");
    
                    let mut review_stars = String::new();
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
