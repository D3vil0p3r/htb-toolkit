use crate::api::fetch_api;
use crate::colors::*;
use crate::utils::get_interface_ip;
use core::time::Duration;
use std::fs;
use std::process;
use std::thread::sleep;

pub fn get_machine_ip(json_data: &serde_json::Value, appkey: &str) -> String {
    let active_id = serde_json::to_string_pretty(&json_data["info"]["id"]).unwrap();
    let mut get_req = String::new();

    // Retrieve the IP address of the Starting Point Machine or Generic Machine
    if let Some(info) = json_data.get("info") {
        if let Some(type_value) = info.get("type").and_then(|t| t.as_str()) {
            if type_value.contains("Starting Point") {
                get_req = format!(
                    "https://www.hackthebox.com/api/v4/sp/profile/{}",
                    active_id
                );
            }
            else {
                get_req = format!(
                    "https://www.hackthebox.com/api/v4/machine/profile/{}",
                    active_id
                );
            }

            loop {
                let sub_result = fetch_api(get_req.as_str(), &appkey);
                match sub_result {
                    Ok(sub_json_data) => {
                        let machine_ip = (&sub_json_data["info"]["ip"]).to_string();
                        if !machine_ip.is_empty() && machine_ip != "null" {
                            return machine_ip;
                        }
                        println!("Retrieving machine IP address... Wait 30 seconds...");
                        sleep(Duration::from_secs(30));
                    }
                    Err(err) => {
                        if err.is_timeout() {
                            eprintln!("Encountered timeout");
                        } else {
                            eprintln!("\x1B[31mError. Maybe your API key is incorrect or expired. Renew your API key by running htb-toolkit -k reset.\x1B[0m");
                        }
                        process::exit(1); // Exit with a non-zero status code
                    }
                }
            }
        }
    }
    
    String::new() // Return an empty string by default if no IP is found
}

pub struct ActiveMachine {
    pub id: u64,
    pub name: String,
    pub ip: String,
    pub mtype: String,
}

impl ActiveMachine {
    pub fn get_active(appkey: &str) -> Self {

        let call_api: &str = "https://www.hackthebox.com/api/v4/machine/active";

        let result = fetch_api(&call_api, &appkey);
        
        //println!("Result: {:?}", result); // DEBUG: Print the result before the match

        match result {
            Ok(json_data) => {
                //println!("Fetched JSON Data: {:?}", json_data); // Print the fetched JSON data
                if let Some(info) = json_data.get("info") {
                    if info.is_null() {
                        eprintln!("\x1B[31mNo active machine detected.\x1B[0m");
                        process::exit(1); // Exit with a non-zero status code
                    }
                }
                
                let machine_ip = get_machine_ip(&json_data, &appkey);

                let entry = &json_data["info"];

                ActiveMachine {
                    id: entry["id"].as_u64().unwrap(),
                    name: entry["name"].as_str().unwrap_or("Name not available").to_string(),
                    ip: machine_ip,
                    mtype: entry["type"].as_str().unwrap_or("null").to_string(),
                }         
            }
            Err(err) => {
                if err.is_timeout() {
                    eprintln!("Encountered timeout");
                } else {
                    eprintln!("\x1B[31mError. Maybe your API key is incorrect or expired. Renew your API key by running htb-toolkit -k reset.\x1B[0m");
                }
                process::exit(1); // Exit with a non-zero status code
            }
        }
    }

    pub fn print_active(active_machine: &ActiveMachine) {
        println!("ID: {}", active_machine.id);
        println!("Name: {}", active_machine.name);
        println!("IP Address: {}", active_machine.ip);
        println!("Type: {}", active_machine.mtype);
    }
}

pub trait CommonTrait {
    fn get_name(&self) -> &str;
    fn get_avatar(&self) -> &str;
}

#[derive(Clone)]
pub struct SPMachine {
    pub id: u64,
    pub name: String,
    pub difficulty_str: String,
    pub tier: u64,
    pub avatar: String,
}

impl CommonTrait for SPMachine {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_avatar(&self) -> &str {
        &self.avatar
    }
}

#[derive(Clone)]
pub struct Machine {
    pub id: u64,
    pub name: String,
    pub points: u64,
    pub difficulty_str: String,
    pub user_pwn: String,
    pub root_pwn: String,
    pub free: bool,
    pub avatar: String,
}

impl CommonTrait for Machine {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_avatar(&self) -> &str {
        &self.avatar
    }
}

pub struct PlayingMachine {
    pub machine: Machine,
    pub os: String,
    pub ip: String,
    pub review: bool,
}

impl PlayingMachine {

    const MACHINE: Machine = Machine {
            id: 0,
            name: String::new(),
            points: 0,
            difficulty_str: String::new(),
            user_pwn: String::new(),
            root_pwn: String::new(),
            free: false,
            avatar: String::new(),
        };

    pub fn new() -> Self {
        PlayingMachine {
            machine: Self::MACHINE.clone(),
            os: String::new(),
            ip: String::new(),
            review: false,
        }
    }

    pub fn get_os_icon(name: String, os: &String, pos: &str) -> String {
        let mut icon_str = String::new();
        
        if pos == "right" {
            if os == "Linux" {
                icon_str = format!("{} ", name);
            } else if os == "Windows" {
                icon_str = format!("{} ", name);
            } else {
                icon_str = name;
            }
        } else if pos == "left" {
            if os == "Linux" {
                icon_str = format!(" {}", name);
            } else if os == "Windows" {
                icon_str = format!("󰖳 {}", name);
            } else {
                icon_str = name;
            }
        }
        else {
            icon_str = name;
        }
        icon_str
    }

    pub fn get_machine(machine_name: &str, appkey: &str) -> Self {

        let base_api: &str = "https://www.hackthebox.com/api/v4/machine/profile/";
        let call_api = format!("{}{}", base_api, machine_name);

        let result = fetch_api(&call_api, &appkey);

        match result {
            Ok(json_data) => {
                if let Some(message) = json_data.get("message").and_then(|m| m.as_str()) {
                    if message.contains("Machine not found") {
                        eprintln!("\x1B[31m{}.\x1B[0m", message);
                        process::exit(1); // Exit with a non-zero status code
                    }
                    else if message.contains("Starting Point Machine") {
                        let tier_id = &json_data["tierId"].as_u64().unwrap();
                        let get_req = format!(
                            "https://www.hackthebox.com/api/v4/sp/tier/{}",
                            tier_id
                        );
                    
                        let sub_result = fetch_api(get_req.as_str(), &appkey);
                        if let Ok(sub_json_data) = sub_result {
                            let machine_ip = get_machine_ip(&sub_json_data, &appkey);
                            let sub_entry = &sub_json_data["info"];
                            let name = sub_entry["name"]
                                        .as_str()
                                        .unwrap_or("Name not available")
                                        .to_string();
                            let os = sub_entry["os"]
                                        .as_str()
                                        .unwrap_or("null")
                                        .to_string();
                            let machine_name_os_icon = Self::get_os_icon(name, &os, "right");
                        
                            return PlayingMachine {
                                machine: Machine {
                                    id: sub_entry["id"].as_u64().unwrap(),
                                    name: machine_name_os_icon,
                                    points: 0,
                                    difficulty_str: sub_entry["difficultyText"]
                                        .as_str()
                                        .unwrap_or("Difficulty not available")
                                        .to_string(),
                                    user_pwn: sub_entry["userOwn"]
                                        .as_str()
                                        .unwrap_or("null")
                                        .to_string(),
                                    root_pwn: sub_entry["rootOwn"]
                                        .as_str()
                                        .unwrap_or("null")
                                        .to_string(),
                                    free: true,
                                    avatar: sub_entry["avatar"]
                                        .as_str()
                                        .unwrap_or("Avatar not available")
                                        .to_string(),
                                },
                                os: os,
                                ip: machine_ip,
                                review: false,
                            };
                        } else {
                            eprintln!("\x1B[31mError fetching Starting Point data.\x1B[0m");
                            process::exit(1); // Exit with a non-zero status code
                        }
                    }
                }

                let entry = &json_data["info"];
                let name = entry["name"]
                            .as_str()
                            .unwrap_or("Name not available")
                            .to_string();
                let os = entry["os"]
                            .as_str()
                            .unwrap_or("null")
                            .to_string();
                let machine_name_os_icon = Self::get_os_icon(name, &os, "right");

                PlayingMachine {
                    machine: Machine {
                        id: entry["id"].as_u64().unwrap(),
                        name: machine_name_os_icon,
                        points: entry["points"].as_u64().unwrap_or(0),
                        difficulty_str: entry["difficultyText"].as_str().unwrap_or("Difficulty not available").to_string(),
                        user_pwn: entry["authUserInUserOwns"].as_str().unwrap_or("null").to_string(),
                        root_pwn: entry["authUserInRootOwns"].as_str().unwrap_or("null").to_string(),
                        free: entry["free"].as_bool().unwrap_or(false),
                        avatar: entry["avatar"]
                            .as_str()
                            .unwrap_or("Avatar not available")
                            .to_string(),
                    },
                    os: os,
                    ip: entry["ip"].as_str().unwrap_or("null").to_string(),
                    review: entry["authUserHasReviewed"].as_bool().unwrap_or(false),
                }         
            }
            Err(err) => {
                if err.is_timeout() {
                    eprintln!("Encountered timeout");
                } else {
                    eprintln!("\x1B[31mError. Maybe your API key is incorrect or expired. Renew your API key by running htb-toolkit -k reset.\x1B[0m");
                }
                process::exit(1); // Exit with a non-zero status code
            }
        }
    }

    pub fn print_machine(m: PlayingMachine) {
        println!("Name: {}", m.machine.name);
        println!("IP Address: {}", m.ip);
        println!("Points: {}", m.machine.points);
        println!("Difficulty: {}", m.machine.difficulty_str);
        println!("User Pwned: {}", m.machine.user_pwn);
        println!("Root Pwned: {}", m.machine.root_pwn);
    }
}

pub struct PlayingUser {
    pub id: u64,
    pub name: String,
    pub ip: String,
    pub vpnname: String,
}

impl PlayingUser {
    
    pub fn new() -> Self {
        PlayingUser {
            id: 0,
            name: String::new(),
            ip: String::new(),
            vpnname: String::new(),
        }
    }
    
    pub fn get_user(appkey: &str) -> Self {
        let id: u64;
        let username: String;
        let mut userip: String = String::new();
        let mut vpnname: String = String::new();

        // Retrieve User username
        let result = fetch_api("https://www.hackthebox.com/api/v4/user/info", &appkey);
    
        match result {
            Ok(json_user) => {
                id = json_user["info"]["id"].as_u64().unwrap();
                username = json_user["info"]["name"].as_str().unwrap().to_string();

                let details = fetch_api(&format!("https://www.hackthebox.com/api/v4/user/profile/basic/{}", id.to_string()), &appkey);
    
                match details {
                    Ok(json_details) => {
                        vpnname = json_details["profile"]["server"].as_str().unwrap().to_string();
                    }
                    Err(err) => {
                        if err.is_timeout() {
                            eprintln!("Encountered timeout");
                        } else {
                            eprintln!("\x1B[31mError. Maybe your API key is incorrect or expired. Renew your API key by running htb-toolkit -k reset.\x1B[0m");
                        }
                        process::exit(1); // Exit with a non-zero status code
                    }
                }                       
            }
            Err(err) => {
                if err.is_timeout() {
                    eprintln!("Encountered timeout");
                } else {
                    eprintln!("\x1B[31mError. Maybe your API key is incorrect or expired. Renew your API key by running htb-toolkit -k reset.\x1B[0m");
                }
                process::exit(1); // Exit with a non-zero status code
            }
        }
    
        // Retrieve User IP address
        let interface_name = "tun0";
        let ip_address = get_interface_ip(interface_name);
    
        match ip_address {
            Some(ip) => {
                userip = ip;
            }
            None => println!("{}Failed to retrieve IP address of {}. Be sure your HTB VPN is active.{}", RED, interface_name, RESET),
        }
    
        PlayingUser {
            id: id,
            name: username,
            ip: userip,
            vpnname: vpnname,
        }
    }
    
}

pub struct HTBConfig {
    pub promptchange: bool,
}

impl HTBConfig {

    pub fn get_current_config(htb_config: &str) -> Self {
        HTBConfig {
            promptchange: Self::get_prompt_change(htb_config),
        }
    }

    fn get_prompt_change(htb_config: &str) -> bool {
        let prompt_change = fs::read_to_string(htb_config).expect("Failed to read htconfig.");

        let change_prompt = prompt_change.lines()
            .find(|line| line.starts_with("prompt_change="))
            .map(|line| line.split("=").nth(1).unwrap_or_default())
            .unwrap_or_default();

        // Convert the change_prompt string to a bool
        let change_prompt_bool = match change_prompt {
            "true" => true,
            "false" => false,
            _ => {
                // Handle other cases if needed, e.g., return a default value
                false
            }
        };
        change_prompt_bool
    }
}