use crate::appkey::get_appkey;
use crate::appkey::set_appkey;
use crate::appkey::reset_appkey;
use crate::appkey::delete_appkey;
use crate::colors::*;
use crate::list::*;
use crate::types::*;
use crate::utils::*;
use reqwest::blocking::{Client, Response};
use std::env;
use std::fs::{self,File};
use std::io::{self,Read,Write};
use std::process::Command;
use regex::Regex;

pub fn get_active_machine_info() {
    let appkey = get_appkey();

    let active_machine = ActiveMachine::get_active(&appkey);
    let machine_info = PlayingMachine::get_machine(&active_machine.name, &appkey);

    PlayingMachine::print_machine(machine_info);
}

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

pub fn update_machines() -> io::Result<()> {
    let appkey = get_appkey();

    println!("Retrieving updated data from Hack The Box... Gimme some time hackerzzz...");

    let input_config = "input_config.txt";
    let output_config = "output_config.txt";

    let free_machine_list = list_machines("free");

    let fly_new = htb_machines_to_flypie(free_machine_list);
    
    let dump_command = format!("dconf dump /org/gnome/shell/extensions/flypie/ > {}", input_config);
    let _ = Command::new("sh")
        .arg("-c")
        .arg(&dump_command)
        .status()?;

    let mut fly_file = File::open(input_config)?;
    let mut contents = String::new();
    fly_file.read_to_string(&mut contents)?;

    let fly_pattern = r#"(.*?)(\{\\"name\\":\\"Available Machines\\",\\"icon\\":\\"/usr/share/icons/htb-tools/htb-machines.png\\",\\"type\\":\\"CustomMenu\\",\\"children\\":)(.*?)(,\\"angle\\":-1,\\"data\\":\{\}\})(.*)"#;
    let re = Regex::new(fly_pattern).unwrap();

    let modified_contents = re.replace(&contents, |caps: &regex::Captures| {
        format!("{}{}{}{}{}", &caps[1], &caps[2], fly_new, &caps[4], &caps[5])
    });

    let mut f = File::create(output_config)?;
    f.write_all(modified_contents.as_bytes())?;

    let load_command = format!("dconf load /org/gnome/shell/extensions/flypie/ < {}", output_config);
    let _ = Command::new("sh")
        .arg("-c")
        .arg(&load_command)
        .status()?;

    // Starting Point Machines

    let sp_machine_list = list_sp_machines();

    let tiers = 3;
    for index in 1..=tiers {
        let tier_lvl = index - 1;

        // Create a sublist of SPMachines with tier equal to 0
        let tiered_list: Vec<SPMachine> = sp_machine_list
            .iter()  // Use iter() instead of into_iter() to borrow instead of move
            .filter(|machine| machine.tier == tier_lvl)
            .cloned()  // Clone the filtered machines
            .collect();

        let fly_new = htb_machines_to_flypie(tiered_list);
        
        let dump_command = format!("dconf dump /org/gnome/shell/extensions/flypie/ > {}", input_config);
        let _ = Command::new("sh")
            .arg("-c")
            .arg(&dump_command)
            .status()?;

        let mut fly_file = File::open(input_config)?;
        let mut contents = String::new();
        fly_file.read_to_string(&mut contents)?;

        let fly_pattern = format!(
            r#"(.*?)(\{{\\"name\\":\\"Tier {}\\",\\"icon\\":\\"/usr/share/icons/htb-tools/Tier-{}.svg\\",\\"type\\":\\"CustomMenu\\",\\"children\\":)(.*?)(,\\"angle\\":-1,\\"data\\":{})"#,
            tier_lvl,
            tier_lvl,
            ""
        );
        let re = Regex::new(&fly_pattern).unwrap();

        let modified_contents = re.replace_all(&contents, |caps: &regex::Captures| {
            format!(
                "{}{}{}{}",
                &caps[1], &caps[2], fly_new, &caps[4]
            )
        });
        let modified_contents_str = modified_contents.to_string();
        
        let mut f = File::create(output_config)?;
        f.write_all(modified_contents_str.as_bytes())?;

        let load_command = format!("dconf load /org/gnome/shell/extensions/flypie/ < {}", output_config);
        let _ = Command::new("sh")
            .arg("-c")
            .arg(&load_command)
            .status()?;
    }

    fs::remove_file(input_config)?;
    fs::remove_file(output_config)?;

    println!("Machines updated. Press Enter to continue...");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");

    Ok(())
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