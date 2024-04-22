use crate::appkey::get_appkey;
use crate::appkey::set_appkey;
use crate::appkey::reset_appkey;
use crate::appkey::delete_appkey;
use crate::colors::*;
use crate::list::*;
use crate::types::*;
use crate::utils::*;
use reqwest::Client;
use std::env;
use std::fs::{self,File};
use std::io::{self,Read,Write};
use std::process::Command;
use regex::Regex;
use tokio::spawn;

pub async fn get_active_machine_info() {
    let appkey = get_appkey();

    let active_machine = ActiveMachine::get_active(&appkey).await;

    // Checking if the active_machine.name exists (so there is an active machine running) to avoid a generic error on api.rs file invoked by fetch_api in get_machine(). The check is done here and not inside ActiveMachine code because only here I need to exit if I don't have an Active Machine running
    if !active_machine.name.is_empty() {
        let mut machine_info = PlayingMachine::get_machine(&active_machine.name, &appkey).await;
        if machine_info.sp_flag {
            machine_info.ip = get_ip(&appkey).await;
        }
        
        PlayingMachine::print_machine(machine_info);
    }
    else {
        println!("Exiting...");
    }
}

pub async fn reset_machine() {
    let appkey = get_appkey();
    let active_machine = ActiveMachine::get_active(&appkey).await;

    // Example of moving the blocking operation to a separate task
    let appkey_clone = appkey.clone(); // Clone the necessary data
    let active_machine_clone = active_machine.clone(); // Clone other data if needed

    let blocking_task = spawn(async move {
        let client = Client::new();
        let reset_data = serde_json::json!({"machine_id": active_machine_clone.id});
        
        // Perform the HTTP request asynchronously
        match client
            .post("https://labs.hackthebox.com/api/v4/vm/reset")
            .header("Authorization", format!("Bearer {}", appkey_clone))
            .json(&reset_data)
            .send()
            .await
        {
            Ok(reset_response) => {
                if reset_response.status().is_success() {                    
                    //let reset_message = reset_response.text().await.expect("Failed to get response text."); //Print the response as text for debug
                    let reset_message = reset_response.json::<serde_json::Value>().await.expect("Failed to parse JSON response. Probably there is no an Active Machine to reset.");
                    let reset_message = reset_message.get("message").unwrap_or(&serde_json::Value::Null).as_str().unwrap();
                    println!("{}{}{}", BGREEN, reset_message, RESET);
                } else {
                    eprintln!("Failed to reset the machine. HTTP status code: {}", reset_response.status());
                }
            }
            Err(err) => {
                eprintln!("Error on POST request: {:?}", err);
            }
        }
    });

    // Await the result of the blocking task
    blocking_task.await.expect("Blocking task failed");

    let active_machine = ActiveMachine::get_active(&appkey).await; //Declared 2nd time because reset machine
    let mut machine_info = PlayingMachine::get_machine(active_machine.name.as_str(), &appkey).await;
    if machine_info.ip.is_empty() { //Starting Point case because SP IP address is assigned only after spawn of the machine
        machine_info.ip = active_machine.ip;
    }
    let mut user_info = PlayingUser::get_playinguser(&appkey).await;

    // SP Machines change IP address when reset, so need to ask to write /etc/hosts
    if machine_info.sp_flag {
        let _ = add_hosts(&machine_info);
    }

    change_shell(&mut machine_info, &mut user_info);
}

pub async fn stop_machine() {
    let htb_path = format!("{}/.htb.conf", env::var("HOME").unwrap());
    let htbconfig = HTBConfig::get_current_config(&htb_path);
    let appkey = get_appkey();
    let active_machine = ActiveMachine::get_active(&appkey).await;

    // Example of moving the blocking operation to a separate task
    let appkey_clone = appkey.clone(); // Clone the necessary data
    let active_machine_clone = active_machine.clone(); // Clone other data if needed

    let account = User::get_user(&appkey).await;
    let machine_type = active_machine.mtype;

    if !active_machine_clone.name.is_empty() { //If there is an active machine, stop it
        let post_req:  String = if machine_type.contains("Starting Point") || (account.vpnname.contains("VIP") ) { //If you are using a VIP or VIP+ VPN, the machine can be stopped only by api/v4/vm/terminate API (even if the machine is free)
            String::from("https://labs.hackthebox.com/api/v4/vm/terminate")
        }
        else {
            String::from("https://labs.hackthebox.com/api/v4/machine/stop")
        };

        let blocking_task = spawn(async move {
            let client = Client::new();
            let stop_data = serde_json::json!({"machine_id": active_machine_clone.id});
            let stop_response = client
                .post(post_req)
                .header("Authorization", format!("Bearer {}", appkey_clone))
                .json(&stop_data)
                .send()
                .await
                .expect("Error on POST request.");

            let stop_message = stop_response.json::<serde_json::Value>().await.expect("Failed to parse JSON response.");
            let stop_message = stop_message.get("message").unwrap_or(&serde_json::Value::Null).as_str().unwrap();
            println!("{}{}{}", BGREEN, stop_message, RESET);
        });

        // Await the result of the blocking task
        blocking_task.await.expect("Blocking task failed");

        if htbconfig.promptchange { //If the prompt is set to change during the playing, when you stop the machine, it should restore the original shell
            restore_shell();
        }
    }
}

pub fn prompt_setting(option: &str) {
    let home = env::var("HOME").unwrap_or_default();
    let htb_config = format!("{}/.htb.conf", home);

    let content = fs::read_to_string(&htb_config)
        .expect("Failed to read HTB config file");

    let re = Regex::new(r"prompt_change=\w+")
        .expect("Failed to create regular expression");

    let new_content = re.replace(&content, format!("prompt_change={}", option));

    fs::write(&htb_config, new_content.to_string())
        .expect("Failed to write updated content to HTB config file");

    println!("Prompt setting updated to: {}", option);
}

pub async fn update_machines() -> io::Result<()> {

    println!("Retrieving updated data from Hack The Box... Gimme some time hackerzzz...");
    let home = env::var("HOME").unwrap_or_default();
    let input_config = format!("{}/.input_config.txt", home);
    let output_config = format!("{}/.output_config.txt", home);

    let free_machine_list = list_machines("free");

    let fly_new = htb_machines_to_flypie(free_machine_list.await).await;
    
    let dump_command = format!("dconf dump /org/gnome/shell/extensions/flypie/ > {}", input_config);
    let _ = Command::new("sh")
        .arg("-c")
        .arg(&dump_command)
        .status()?;

    let mut fly_file = File::open(&input_config)?;
    let mut contents = String::new();
    fly_file.read_to_string(&mut contents)?;

    let fly_pattern = r#"(.*?)(\{\\"name\\":\\"Available Machines\\",\\"icon\\":\\"/usr/share/icons/htb-toolkit/htb-machines.png\\",\\"type\\":\\"CustomMenu\\",\\"children\\":)(.*?)(,\\"angle\\":-1,\\"data\\":\{\})(.*)"#;
    let re = Regex::new(fly_pattern).unwrap();

    let modified_contents = re.replace(&contents, |caps: &regex::Captures| {
        format!("{}{}{}{}{}", &caps[1], &caps[2], fly_new, &caps[4], &caps[5])
    });

    let mut f = File::create(&output_config)?;
    f.write_all(modified_contents.as_bytes())?;

    let load_command = format!("dconf load /org/gnome/shell/extensions/flypie/ < {}", output_config);
    let _ = Command::new("sh")
        .arg("-c")
        .arg(&load_command)
        .status()?;

    // Starting Point Machines
    let sp_machine_list = list_sp_machines().await;

    let tiers = 3;
    for index in 1..=tiers {
        let tier_lvl = index - 1;

        // Create a sublist of SPMachines with tier equal to 0
        let tiered_list: Vec<SPMachine> = sp_machine_list
            .iter()  // Use iter() instead of into_iter() to borrow instead of move
            .filter(|machine| machine.tier == tier_lvl)
            .cloned()  // Clone the filtered machines
            .collect();

        let fly_new = htb_machines_to_flypie(tiered_list).await;
        
        let dump_command = format!("dconf dump /org/gnome/shell/extensions/flypie/ > {}", input_config);
        let _ = Command::new("sh")
            .arg("-c")
            .arg(&dump_command)
            .status()?;

        let mut fly_file = File::open(&input_config)?;
        let mut contents = String::new();
        fly_file.read_to_string(&mut contents)?;

        let fly_pattern = format!(
            r#"(.*?)(\{{\\"name\\":\\"Tier {}\\",\\"icon\\":\\"/usr/share/icons/htb-toolkit/Tier-{}.svg\\",\\"type\\":\\"CustomMenu\\",\\"children\\":)(.*?)(,\\"angle\\":-1,\\"data\\":{})"#,
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
        
        let mut f = File::create(&output_config)?;
        f.write_all(modified_contents_str.as_bytes())?;

        let load_command = format!("dconf load /org/gnome/shell/extensions/flypie/ < {}", output_config);
        let _ = Command::new("sh")
            .arg("-c")
            .arg(&load_command)
            .status()?;
    }

    fs::remove_file(input_config)?;
    fs::remove_file(output_config)?;

    print!("\n{}Machines updated. Press Enter to continue...{}", BGREEN, RESET);
    let mut input = String::new();
    io::stdout().flush().expect("Flush failed!");
    io::stdin().read_line(&mut input).expect("Failed to read line");

    Ok(())
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