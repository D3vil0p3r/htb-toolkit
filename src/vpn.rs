use crate::api::fetch_api;
use crate::appkey::get_appkey;
use crate::colors::*;
use crate::utils::*;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::process::Command;
use std::thread;
use std::time::Duration;

use serde_json::Value;

pub fn print_vpn_sp_list() {
    println!("{}┌────────────────────────────────────────────────────┐{}", BCYAN, RESET);
    println!("{}| Starting Point Free VPN : EUSPFree1 USSPFree1      |{}", BCYAN, RESET);
    println!("{}| Starting Point VIP VPN  : EUSPVIP1  USSPVIP1       |{}", BYELLOW, RESET);
    println!("{}└────────────────────────────────────────────────────┘{}", BYELLOW, RESET);
}

pub fn print_vpn_machine_list() {
    println!("{}┌────────────────────────────────────────────────────┐{}", BCYAN, RESET);
    println!("{}| Machines Free VPN       : EUFree1 EUFree2 EUFree3  |{}", BCYAN, RESET);
    println!("{}|                           USFree1 USFree2 USFree3  |{}", BCYAN, RESET);
    println!("{}|                           AUFree1 SGFree1          |{}", BCYAN, RESET);
    println!("{}| Machines VIP VPN        : EUVIP1 to EUVIP28        |{}", BYELLOW, RESET);
    println!("{}|                           USVIP1 to USVIP27        |{}", BYELLOW, RESET);
    println!("{}|                           SGVIP1 SGVIP2 AUVIP1     |{}", BYELLOW, RESET);
    println!("{}| Machines VIP+ VPN       : EUVIP+1 EUVIP+2          |{}", RED, RESET);
    println!("{}| Machines VIP+ VPN       : USVIP+1 SGVIP+1          |{}", RED, RESET);
    println!("{}└────────────────────────────────────────────────────┘{}", RED, RESET);
}

fn vpn_type() -> Option<Vec<String>> {
    let appkey = get_appkey();
    let result = fetch_api("https://www.hackthebox.com/api/v4/connection/status", &appkey);
    let mut vpntype: Vec<String> = Vec::new();

    match result {
        Ok(json_value) => {
            if let Some(json_vpn) = json_value.as_array() {
                for item in json_vpn {
                    if let Some(vpntype_value) = item["type"].as_str() {
                        vpntype.push(vpntype_value.to_string());
                    }
                }
            } else {
                println!("Expected JSON array");
            }
        }
        Err(err) => {
            if err.is_timeout() {
                eprintln!("Encountered timeout");
            } else {
                eprintln!(
                    "\x1B[31mError. Maybe your API key is incorrect or expired. Renew your API key by running htb-toolkit -k reset.\x1B[0m"
                );
            }
        }
    }
    
    // If HTB VPN is active and tun0 is also active on your client...
    if get_interface_ip("tun0").is_some() {
        Some(vpntype)
    } else {
        None
    }
}

pub fn check_vpn(machine_spflag: bool) {
    let mut vpn = String::new();
    if let Some(vpntypes) = vpn_type() {
        let vpntypes_str = vpntypes.join(", "); // Join the VPN types with a comma and space. Note that if we switch two different VPNs, they can still leave together for some time
        let mut yn = String::new();
        if vpntypes.len() > 1 {
            println!(
                "\nThe following VPN types are already running: {}. You have multiple VPNs running. The oldest one will go down automatically in some minutes.",
                vpntypes_str
            );
        } else {
            println!(
                "\nThe following VPN type is already running: {}.",
                vpntypes_str
            );
        }

        println!("Do you want to terminate the listed VPN and choose a new one (y/n)?");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut yn).expect("Failed to read input");
        loop {
            match yn.trim() {
                "y" | "Y" => {
                    if machine_spflag {
                        print_vpn_sp_list();
                    } else {
                        print_vpn_machine_list();
                    }
                    println!("Please, provide one VPN server you prefer to connect:");
                    io::stdout().flush().expect("Flush failed!");
                    io::stdin()
                        .read_line(&mut vpn)
                        .expect("Failed to read line");
                    run_vpn(&vpn);
                    break;
                }
                "n" | "N" => break,
                _ => println!("Invalid answer."),
            }
        }
    }
    else {
        if machine_spflag {
            print_vpn_sp_list();
        } else {
            print_vpn_machine_list();
        }
        println!("Please, provide one VPN server you prefer to connect:");
        io::stdout().flush().expect("Flush failed!");
        io::stdin()
            .read_line(&mut vpn)
            .expect("Failed to read line");
        run_vpn(&vpn);
    }
}

pub fn run_vpn(chosen_server: &str) {

    let appkey = get_appkey();
    
    let mut vpn_tcp = String::from("/1");

    let mut vpn_servers = HashMap::new();
    vpn_servers.insert(String::from("EUFree1"), "1");
    vpn_servers.insert(String::from("EUFree2"), "201");
    vpn_servers.insert(String::from("EUFree3"), "253");
    vpn_servers.insert(String::from("USFree1"), "113");
    vpn_servers.insert(String::from("USFree2"), "202");
    vpn_servers.insert(String::from("USFree3"), "254");
    vpn_servers.insert(String::from("AUFree1"), "117");
    vpn_servers.insert(String::from("SGFree1"), "251");
    vpn_servers.insert(String::from("EUVIP1"), "2");
    vpn_servers.insert(String::from("EUVIP2"), "5");
    vpn_servers.insert(String::from("EUVIP3"), "6");
    vpn_servers.insert(String::from("EUVIP4"), "7");
    vpn_servers.insert(String::from("EUVIP5"), "8");
    vpn_servers.insert(String::from("EUVIP6"), "9");
    vpn_servers.insert(String::from("EUVIP7"), "18");
    vpn_servers.insert(String::from("EUVIP8"), "21");
    vpn_servers.insert(String::from("EUVIP9"), "28");
    vpn_servers.insert(String::from("EUVIP10"), "30");
    vpn_servers.insert(String::from("EUVIP11"), "33");
    vpn_servers.insert(String::from("EUVIP12"), "36");
    vpn_servers.insert(String::from("EUVIP13"), "42");
    vpn_servers.insert(String::from("EUVIP14"), "44");
    vpn_servers.insert(String::from("EUVIP15"), "47");
    vpn_servers.insert(String::from("EUVIP16"), "49");
    vpn_servers.insert(String::from("EUVIP17"), "51");
    vpn_servers.insert(String::from("EUVIP18"), "54");
    vpn_servers.insert(String::from("EUVIP19"), "57");
    vpn_servers.insert(String::from("EUVIP20"), "61");
    vpn_servers.insert(String::from("EUVIP21"), "66");
    vpn_servers.insert(String::from("EUVIP22"), "68");
    vpn_servers.insert(String::from("EUVIP23"), "70");
    vpn_servers.insert(String::from("EUVIP24"), "73");
    vpn_servers.insert(String::from("EUVIP25"), "77");
    vpn_servers.insert(String::from("EUVIP26"), "219");
    vpn_servers.insert(String::from("EUVIP27"), "222");
    vpn_servers.insert(String::from("EUVIP28"), "122");
    vpn_servers.insert(String::from("USVIP1"), "11");
    vpn_servers.insert(String::from("USVIP2"), "14");
    vpn_servers.insert(String::from("USVIP3"), "17");
    vpn_servers.insert(String::from("USVIP4"), "20");
    vpn_servers.insert(String::from("USVIP5"), "23");
    vpn_servers.insert(String::from("USVIP6"), "27");
    vpn_servers.insert(String::from("USVIP7"), "29");
    vpn_servers.insert(String::from("USVIP8"), "31");
    vpn_servers.insert(String::from("USVIP9"), "35");
    vpn_servers.insert(String::from("USVIP10"), "38");
    vpn_servers.insert(String::from("USVIP11"), "41");
    vpn_servers.insert(String::from("USVIP12"), "45");
    vpn_servers.insert(String::from("USVIP13"), "46");
    vpn_servers.insert(String::from("USVIP14"), "48");
    vpn_servers.insert(String::from("USVIP15"), "50");
    vpn_servers.insert(String::from("USVIP16"), "52");
    vpn_servers.insert(String::from("USVIP17"), "56");
    vpn_servers.insert(String::from("USVIP18"), "58");
    vpn_servers.insert(String::from("USVIP19"), "65");
    vpn_servers.insert(String::from("USVIP20"), "67");
    vpn_servers.insert(String::from("USVIP21"), "69");
    vpn_servers.insert(String::from("USVIP22"), "71");
    vpn_servers.insert(String::from("USVIP23"), "74");
    vpn_servers.insert(String::from("USVIP24"), "86");
    vpn_servers.insert(String::from("USVIP25"), "89");
    vpn_servers.insert(String::from("USVIP26"), "220");
    vpn_servers.insert(String::from("USVIP27"), "223");
    vpn_servers.insert(String::from("AUVIP1"), "182");
    vpn_servers.insert(String::from("SGVIP1"), "252");
    vpn_servers.insert(String::from("SGVIP2"), "280");
    vpn_servers.insert(String::from("EUVIP+1"), "288");
    vpn_servers.insert(String::from("EUVIP+2"), "314");
    vpn_servers.insert(String::from("USVIP+1"), "289");
    vpn_servers.insert(String::from("SGVIP+1"), "426");
    vpn_servers.insert(String::from("EUSPFree1"), "412");
    vpn_servers.insert(String::from("EUSPVIP1"), "413");
    vpn_servers.insert(String::from("USSPFree1"), "414");
    vpn_servers.insert(String::from("USSPVIP1"), "415");

    let mut key = chosen_server.trim().to_string();
    loop {
        if !vpn_servers.contains_key(&key) {
            // Server not found, run the print_list() function
            key.clear(); // Clear the variable's content
            print_vpn_sp_list();
            print_vpn_machine_list();
            println!("Please, provide one VPN server you prefer to connect:");
            io::stdout().flush().expect("Flush failed!");
            io::stdin()
                .read_line(&mut key)
                .expect("Failed to read line");

            key = key.trim().to_string();
        }
        else {
            break;
        }
    }

    let vpn_id = vpn_servers[&key];

    loop {
        println!("\n{}Would you like to connect to Hack The Box VPN by UDP or TCP? [UDP]{}", BGREEN, RESET);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        input = input.trim().to_string(); //remove \n from Enter keyboard button

        if input.is_empty() {
            input = "udp".to_string();
        }

        match input.trim().to_lowercase().as_str() {
            "udp" => {
                break;
            }
            "tcp" => {
                vpn_tcp = String::from("/0");
                break;
            }
            _ => {
                println!("{}Please select UDP or TCP:{}", BGREEN, RESET);
            }
        }
    }

    println!("\nConnecting to {} server [id={}]\n", key, vpn_id);

    let _output = Command::new("sudo")
        .arg("killall")
        .arg("openvpn")
        .output()
        .expect("Failed to execute command");

    let switch_url = format!("https://www.hackthebox.com/api/v4/connections/servers/switch/{}", vpn_id);
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(switch_url)
        .header("Authorization", format!("Bearer {}", appkey))
        .send();

    match response {
        Ok(response) => {
            if response.status().is_success() {
                let response_text = response.text().unwrap();
                let response_json: Value = serde_json::from_str(&response_text).unwrap();
                let message = response_json["message"].as_str().unwrap();
                println!("Switch response: {}", message);
            } else {
                eprintln!("API call failed with status: {}", response.status());
                std::process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("API call error: {:?}", err);
            std::process::exit(1);
        }
    }

    let ovpn_url = format!(
        "https://www.hackthebox.com/api/v4/access/ovpnfile/{}{}",
        vpn_id, vpn_tcp
    );
    let ovpn_response = client
        .get(ovpn_url)
        .header("Authorization", format!("Bearer {}", appkey))
        .send();

    match ovpn_response {
        Ok(response) => {
            if response.status().is_success() {
                let ovpn_content = response.text().unwrap();
                let ovpn_file_path = format!("{}/lab-vpn.ovpn", std::env::var("HOME").unwrap_or_default());
                // Write content to a file named "lab-vpn.ovpn"
                if let Err(err) = fs::write(&ovpn_file_path, ovpn_content) {
                    eprintln!("Error writing to file: {}", err);
                } else {
                    println!("File saved successfully.");
                }

                let mut openvpn_cmd = Command::new("sudo");
                openvpn_cmd
                    .arg("openvpn")
                    .arg("--config")
                    .arg(ovpn_file_path)
                    .arg("--daemon");

                let status = openvpn_cmd.status().expect("Failed to execute openvpn command");
                if status.success() {
                    println!("OpenVPN started successfully");
                } else {
                    eprintln!("OpenVPN process exited with error: {:?}", status);
                    std::process::exit(1);
                }
            } else {
                eprintln!("API call failed with status: {}", response.status());
                std::process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("API call error: {:?}", err);
            std::process::exit(1);
        }
    }

    thread::sleep(Duration::from_secs(5));

    println!("You are running OpenVPN in background. For terminating it, close this window by mouse right-click on the window bar. If you type 'exit', OpenVPN will restart due to its native configuration.");
}