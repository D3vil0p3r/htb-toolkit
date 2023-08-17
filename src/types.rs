use crate::api::fetch_api;
use crate::colors::*;
use crate::utils::get_interface_ip;

pub struct PlayingMachine {
    pub id: u64,
    pub name: String,
    pub points: u64,
    pub difficulty_str: String,
    pub user_pwn: String,
    pub root_pwn: String,
    pub free: bool,
    pub os: String,
    pub ip: String,
    pub review: bool,
}

impl PlayingMachine {

    pub fn new() -> Self {
        PlayingMachine {
            id: 0,
            name: String::new(),
            points: 0,
            difficulty_str: String::new(),
            user_pwn: String::new(),
            root_pwn: String::new(),
            free: false,
            os: String::new(),
            ip: String::new(),
            review: false,
        }
    }

    pub fn get_machine(machine_name: &str, appkey: &str) -> Self {

        let base_api: &str = "https://www.hackthebox.com/api/v4/machine/profile/";
        let call_api = format!("{}{}", base_api, machine_name);

        let result = fetch_api(&call_api, &appkey);

        match result {
            Ok(json_data) => {
                if let Some(message) = json_data["message"].as_str() {
                    if message.contains("Machine not found") {
                        panic!("\x1B[31mMachine not found.\x1B[0m");
                    }
                }

                let entry = &json_data["info"];

                PlayingMachine {
                    id: entry["id"].as_u64().unwrap(),
                    name: entry["name"].as_str().unwrap_or("Name not available").to_string(),
                    points: entry["points"].as_u64().unwrap_or(0),
                    difficulty_str: entry["difficultyText"].as_str().unwrap_or("Difficulty not available").to_string(),
                    user_pwn: entry["authUserInUserOwns"].as_str().unwrap_or("null").to_string(),
                    root_pwn: entry["authUserInRootOwns"].as_str().unwrap_or("null").to_string(),
                    free: entry["free"].as_bool().unwrap_or(false),
                    os: entry["os"].as_str().unwrap_or("null").to_string(),
                    ip: entry["ip"].as_str().unwrap_or("null").to_string(),
                    review: entry["authUserHasReviewed"].as_bool().unwrap_or(false),
                }         
            }
            Err(err) => {
                if err.is_timeout() {
                    panic!("Encountered timeout");
                } else {
                    panic!("\x1B[31mError. Maybe your API key is incorrect or expired. Renew your API key by running htb-update.\x1B[0m");
                }
            }
        }
    }
}

pub struct PlayingUser {
    pub name: String,
    pub ip: String,
}

impl PlayingUser {
    
    pub fn new() -> Self {
        PlayingUser {
            name: String::new(),
            ip: String::new(),
        }
    }
    
    pub fn get_user(appkey: &str) -> Self {
        let username: String;
        let mut userip: String = String::new();

        // Retrieve User username
        let result = fetch_api("https://www.hackthebox.com/api/v4/user/info", &appkey);
    
        match result {
            Ok(json_user) => {
                username = json_user["info"]["name"].as_str().unwrap().to_string();
            }
            Err(err) => {
                if err.is_timeout() {
                    panic!("Encountered timeout");
                } else {
                    panic!("\x1B[31mError. Maybe your API key is incorrect or expired. Renew your API key by running htb-update.\x1B[0m");
                }
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
            name: username,
            ip: userip,
        }
    }
    
}