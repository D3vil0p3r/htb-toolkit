use crate::appkey::get_appkey;
use crate::appkey::set_appkey;
use crate::appkey::reset_appkey;
use crate::appkey::delete_appkey;
use crate::colors::*;
use crate::types::*;
use crate::utils::*;
use reqwest::blocking::{Client, Response};
use std::env;
use std::fs;
use regex::Regex;

pub fn reset_machine() {
    let appkey = get_appkey();
    let client = Client::new();

    let active_machine = ActiveMachine::get_active(&appkey);    

    let reset_data = serde_json::json!({"machine_id": active_machine.id});
    let reset_response: Response = client
        .post("https://www.hackthebox.com/api/v4/vm/reset")
        .header("Authorization", format!("Bearer {}", appkey))
        .json(&reset_data)
        .send()
        .expect("Error on POST request.");
    let reset_message = reset_response.json::<serde_json::Value>().expect("Failed to parse JSON response.");
    let reset_message = reset_message.get("message").unwrap_or(&serde_json::Value::Null).as_str().unwrap();
    println!("{}{}{}", BGREEN, reset_message, RESET);

    let active_machine = ActiveMachine::get_active(&appkey);
    let mut machine_info = PlayingMachine::get_machine(active_machine.name.as_str(), &appkey);
    let mut user_info = PlayingUser::get_user(&appkey);

    change_shell(&mut machine_info, &mut user_info);
}

pub fn stop_machine() {
    let htb_path = format!("{}/.htb.conf", env::var("HOME").unwrap());
    let htbconfig = HTBConfig::get_current_config(&htb_path);
    let appkey = get_appkey();
    let active_machine = ActiveMachine::get_active(&appkey);
    let client = Client::new();
    let account = PlayingUser::get_user(&appkey);
    let machine_type = active_machine.mtype;

    let mut post_req = String::new();

    if machine_type.contains("Starting Point") || account.vpnname.contains("VIP") { //If you are using a VIP VPN, the machine can be stopped only by api/v4/vm/terminate API (even if the machine is free)
        post_req = String::from("https://www.hackthebox.com/api/v4/vm/terminate");
    }
    else {
        post_req = String::from("https://www.hackthebox.com/api/v4/machine/stop");
    }
    let stop_data = serde_json::json!({"machine_id": active_machine.id});
    let stop_response: Response = client
        .post(post_req.as_str())
        .header("Authorization", format!("Bearer {}", appkey))
        .json(&stop_data)
        .send()
        .expect("Error on POST request.");
    let stop_message = stop_response.json::<serde_json::Value>().expect("Failed to parse JSON response.");
    let stop_message = stop_message.get("message").unwrap_or(&serde_json::Value::Null).as_str().unwrap();
    println!("{}{}{}", BGREEN, stop_message, RESET);
    
    println!("{}", stop_message);

    if htbconfig.promptchange == true { //If the prompt is set to change during the playing, when you stop the machine, it should restore the original shell
        restore_shell();
    }
}

pub fn prompt_setting(option: &str) {
    let home = env::var("HOME").unwrap_or_default();
    let htb_config = format!("{}/.htb.conf", home);

    let mut content = fs::read_to_string(&htb_config)
        .expect("Failed to read HTB config file");

    let re = Regex::new(r"prompt_change=\w+")
        .expect("Failed to create regular expression");

    let new_content = re.replace(&content, format!("prompt_change={}", option));

    fs::write(&htb_config, new_content.to_string())
        .expect("Failed to write updated content to HTB config file");

    println!("Prompt setting updated to: {}", option);
}

pub fn update_machines() {
    let appkey = get_appkey();
}

pub fn set_vpn() {
    println!("Setting a Hack The Box VPN...");
    // Your implementation here
}

pub fn manage_app_key(option: &str) {
    if option == "set" {
        set_appkey();
    }
    else if option == "reset" {
        reset_appkey();
    }
    else if option == "delete" {
        delete_appkey();
    }
}