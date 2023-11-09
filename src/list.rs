use crate::appkey::get_appkey;
use crate::api::fetch_api_async;
use crate::colors::*;
use crate::types::*;

pub async fn list_sp_machines() -> Vec<SPMachine> {
    let tiers = 3;

    println!("\n\x1B[93mConnecting to HTB server...\x1B[0m\n");
    let appkey = get_appkey();

    let mut all_sp_machine_list: Vec<SPMachine> = Vec::new();

    for index in 1..=tiers {
        let mut sp_machine_list: Vec<SPMachine> = Vec::new();
        let tier_lvl = index - 1;
        let tier_url = format!("https://www.hackthebox.com/api/v4/sp/tier/{}", index);
        let result = fetch_api_async(&tier_url, &appkey);

        match result.await {
            Ok(json_data) => {
                println!("\x1B[92mDone.\x1B[0m\n");

                for entry in json_data["data"]["machines"].as_array().unwrap().iter() {
                    let id = entry["id"].as_u64().unwrap();
                    let name = entry["name"].as_str().unwrap_or("Name not available").to_string();
                    let os = entry["os"].as_str().unwrap_or("OS not available").to_string();
                    let machine_name_os_icon = PlayingMachine::get_os_icon(name, &os, "left");
                    let difficulty_str = entry["difficultyText"].as_str().unwrap_or("Difficulty not available").to_string();
                    let avatar_path = entry["avatar"].as_str().unwrap_or("Avatar not available").to_string();

                    let sp_machine = SPMachine { id, name: machine_name_os_icon, difficulty_str, tier: tier_lvl, avatar: avatar_path };

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
                    eprintln!("\x1B[31mError. Maybe your API key is incorrect or expired. Renew your API key by running htb-toolkit -k reset.\x1B[0m");
                }
            }
        }
    }
    
    all_sp_machine_list
}

pub async fn list_machines(machine_type: &str) -> Vec<Machine> {
    let mut machine_list: Vec<Machine> = Vec::new();

    println!("Listing machines... This operation could require some minutes...");
    println!("\n\x1B[93mConnecting to HTB server...\x1B[0m\n");

    let appkey = get_appkey(); // Retrieve the app key

    let result = match machine_type {
        "free" => fetch_api_async("https://www.hackthebox.com/api/v4/machine/paginated", &appkey),
        "retired" => fetch_api_async("https://www.hackthebox.com/api/v4/machine/list/retired/paginated", &appkey),
        _ => {
            eprintln!("\x1B[31mInvalid machine type: {}\x1B[0m", machine_type);
            return machine_list;
        }
    };
    
    match result.await {
        Ok(json_data) => {
            println!("\x1B[92mDone.\x1B[0m\n");
            println!("\x1B[93mCalculating the number of machines...\x1B[0m\n");
            std::thread::sleep(std::time::Duration::from_secs(1));

            let mut array_index_free_machines = Vec::new();

            println!("\x1B[92mDone.\x1B[0m\n");

            for (sequence, entry) in json_data["data"].as_array().unwrap().iter().enumerate() {
                let index = sequence;

                let id = entry["id"].as_u64().unwrap_or(0);
                let name = entry["name"].as_str().unwrap_or("Name not available").to_string();
                let os = entry["os"].as_str().unwrap_or("OS not available").to_string();
                let machine_name_os_icon = PlayingMachine::get_os_icon(name, &os, "left");
                let points = entry["points"].as_u64().unwrap_or(0);
                let difficulty_str = entry["difficultyText"].as_str().unwrap_or("Difficulty not available").to_string();
                let user_pwn = entry["authUserInUserOwns"].to_string();
                let root_pwn = entry["authUserInRootOwns"].to_string();
                let free = entry["free"].as_bool().unwrap_or(false);
                let avatar_path = entry["avatar"].as_str().unwrap_or("Avatar not available").to_string();

                if free && machine_type == "retired" {
                    array_index_free_machines.push(index);
                }

                let machine = Machine {
                    id,
                    name: machine_name_os_icon,
                    points,
                    difficulty_str,
                    user_pwn,
                    root_pwn,
                    free,
                    avatar: avatar_path,
                };

                machine_list.push(machine);
            }

            display_table(&machine_list);

            if machine_type == "retired" {
                println!();
                println!("\x1B[92mToday, the free retired machines are:\x1B[0m\n");

                for index in array_index_free_machines {
                    let name = json_data["data"][index]["name"].as_str().unwrap();
                    println!("{}", name);
                }
            }
        }
        Err(err) => {
            if err.is_timeout() {
                eprintln!("Encountered timeout");
            } else {
                eprintln!("\x1B[31mError. Maybe your API key is incorrect or expired. Renew your API key by running htb-toolkit -k reset.\x1B[0m");
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
        println!(
            "{}{:<8}{} {}{:<20}{} {}{:<8}{} {}{:<15}{} {}{:<10}{} {}{:<10}{} {}{:<8}{}",
            BBLUE, machine.id, RESET,
            BGREEN, machine.name, RESET,
            BYELLOW, machine.points, RESET,
            BCYAN, machine.difficulty_str, RESET,
            BYELLOW, machine.user_pwn, RESET,
            RED, machine.root_pwn, RESET,
            BGREEN, machine.free, RESET,
        );
    }
}