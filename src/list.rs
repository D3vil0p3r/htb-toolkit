use crate::appkey::get_appkey;
use crate::api::fetch_api;
use crate::colors::*;
use crate::types::*;

pub fn list_sp_machines(machine_type: &str) -> Vec<SPMachine> {
    let tiers = 3;

    println!("\x1B[93mConnecting to HTB server...\x1B[0m\n");
    let appkey = get_appkey();

    let mut all_sp_machine_list: Vec<SPMachine> = Vec::new();

    for index in 1..=tiers {
        let mut sp_machine_list: Vec<SPMachine> = Vec::new();
        let tier_lvl = index - 1;
        let result = match machine_type {
            "starting" => fetch_api(&("https://www.hackthebox.com/api/v4/sp/tier/".to_owned() + index.to_string().as_str()), &appkey),
            _ => {
                eprintln!("\x1B[31mInvalid machine type: {}\x1B[0m", machine_type);
                continue;
            }
        };

        match result {
            Ok(json_data) => {
                println!("\x1B[92mDone.\x1B[0m\n");

                for entry in json_data["data"]["machines"].as_array().unwrap().iter() {
                    let id = entry["id"].as_u64().unwrap();
                    let name = entry["name"].as_str().unwrap_or("Name not available").to_string();
                    let os = entry["os"].as_str().unwrap_or("OS not available").to_string();
                    let machine_name_os_icon = PlayingMachine::get_os_icon(name, &os);
                    let difficulty_str = entry["difficultyText"].as_str().unwrap_or("Difficulty not available").to_string();

                    let sp_machine = SPMachine { id, name: machine_name_os_icon, difficulty_str: difficulty_str, tier: tier_lvl };

                    sp_machine_list.push(sp_machine);
                }
                println!("{}Tier {} Starting Point machines:{}\n", BYELLOW, tier_lvl, RESET);
                display_table_sp(&sp_machine_list);

                all_sp_machine_list.extend(sp_machine_list);
            }
            Err(err) => {
                if err.is_timeout() {
                    eprintln!("Encountered timeout");
                } else {
                    eprintln!("\x1B[31mError. Maybe your API key is incorrect or expired. Renew your API key by running htb-update.\x1B[0m");
                }
            }
        }
    }
    
    all_sp_machine_list
}

pub fn list_machines(machine_type: &str) -> Vec<Machine> {
    let mut machine_list: Vec<Machine> = Vec::new();

    println!("Listing machines...");
    println!("\x1B[93mConnecting to HTB server...\x1B[0m\n");

    let appkey = get_appkey(); // Retrieve the app key

    let result = match machine_type {
        "free" => fetch_api("https://www.hackthebox.com/api/v4/machine/list", &appkey),
        "retired" => fetch_api("https://www.hackthebox.com/api/v4/machine/list/retired", &appkey),
        _ => {
            eprintln!("\x1B[31mInvalid machine type: {}\x1B[0m", machine_type);
            return machine_list;
        }
    };

    match result {
        Ok(json_data) => {
            println!("\x1B[92mDone.\x1B[0m\n");
            println!("\x1B[93mCalculating the number of machines...\x1B[0m\n");
            std::thread::sleep(std::time::Duration::from_secs(1));

            let mut array_index_free_machines = Vec::new();

            println!("\x1B[92mDone.\x1B[0m\n");

            for (sequence, entry) in json_data["info"].as_array().unwrap().iter().enumerate() {
                let index = sequence;

                let id = entry["id"].as_u64().unwrap_or(0);
                let name = entry["name"].as_str().unwrap_or("Name not available").to_string();
                let os = entry["os"].as_str().unwrap_or("OS not available").to_string();
                let machine_name_os_icon = PlayingMachine::get_os_icon(name, &os);
                let points = entry["points"].as_u64().unwrap_or(0);
                let difficulty_str = entry["difficultyText"].as_str().unwrap_or("Difficulty not available").to_string();
                let user_pwn = entry["authUserInUserOwns"].as_str().unwrap_or("null").to_string();
                let root_pwn = entry["authUserInRootOwns"].as_str().unwrap_or("null").to_string();
                let free = entry["free"].as_bool().unwrap_or(false);

                if free && machine_type == "retired" {
                    array_index_free_machines.push(index);
                }

                let machine = Machine {
                    id,
                    name: machine_name_os_icon,
                    points,
                    difficulty_str: difficulty_str,
                    user_pwn: user_pwn,
                    root_pwn: root_pwn,
                    free,
                };

                machine_list.push(machine);
            }

            display_table(&machine_list);

            if machine_type == "retired" {
                println!();
                println!("\x1B[92mToday, the free retired machines are:\x1B[0m\n");

                for index in array_index_free_machines {
                    let name = json_data["info"][index]["name"].as_str().unwrap();
                    println!("{}", name);
                }
            }
        }
        Err(err) => {
            if err.is_timeout() {
                eprintln!("Encountered timeout");
            } else {
                eprintln!("\x1B[31mError. Maybe your API key is incorrect or expired. Renew your API key by running htb-update.\x1B[0m");
            }
        }
    }
    machine_list
}

fn display_table_sp(machine_list: &[SPMachine]) {
    println!(
        "{}{:<8}{} {}{:<20}{} {}{:<15}{}",
        BBLUE, "ID", RESET, BGREEN, "Name", RESET, BCYAN, "Difficulty", RESET, 
    );
    println!("----------------------------------------");

    for machine in machine_list {
        println!(
            "{}{:<8}{} {}{:<20}{} {}{:<15}{}",
            BBLUE, machine.id, RESET,
            BGREEN, machine.name, RESET,
            BCYAN, machine.difficulty_str, RESET,
        );
    }
}

fn display_table(machine_list: &[Machine]) {
    println!(
        "{}{:<8}{} {}{:<20}{} {}{:<8}{} {}{:<15}{} {}{:<10}{} {}{:<10}{} {}{:<8}{}",
        BBLUE, "ID", RESET, BGREEN, "Name", RESET, BYELLOW, "Points", RESET, BCYAN, "Difficulty", RESET, BYELLOW, "User Pwned", RESET, RED, "Root Pwned", RESET, BGREEN, "Is it Free?", RESET
    );
    println!("-------------------------------------------------------------");

    for machine in machine_list {
        let user_pwn_colored = if machine.user_pwn != "null" {
            format!("\x1B[93m{}\x1B[0m", machine.user_pwn)
        } else {
            machine.user_pwn.to_string()
        };

        let root_pwn_colored = if machine.root_pwn != "null" {
            format!("\x1B[91m{}\x1B[0m", machine.root_pwn)
        } else {
            machine.root_pwn.to_string()
        };

        let free_colored = if machine.free {
            format!("\x1B[92m{}\x1B[0m", machine.free)
        } else {
            machine.free.to_string()
        };

        println!(
            "{}{:<8}{} {}{:<20}{} {}{:<8}{} {}{:<15}{} {}{:<10}{} {}{:<10}{} {}{:<8}{}",
            BBLUE, machine.id, RESET,
            BGREEN, machine.name, RESET,
            BYELLOW, machine.points, RESET,
            BCYAN, machine.difficulty_str, RESET,
            BYELLOW, user_pwn_colored, RESET,
            RED, root_pwn_colored, RESET,
            BGREEN, free_colored, RESET,
        );
    }
}