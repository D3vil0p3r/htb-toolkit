use std::process::{Command, Stdio};
use std::io::{self, Write};
use std::env;
use crate::colors::*;
use crate::types::*;
use crate::vpn::*;
use pnet::datalink;
use std::fs::{self, File};
use std::net::IpAddr;

pub fn change_shell(machine_info: &mut PlayingMachine, user_info: &mut PlayingUser) {
    let result = std::env::var("SHELL").unwrap_or_default();
    let mut file_bak = String::new();
    let mut file = String::new();
    let mut prompt = String::new();
    let mut prompt_field = "";

    if result.contains("bash") {
        file_bak = format!("{}/.bashrc.htb.bak", std::env::var("HOME").unwrap_or_default());
        file = format!("{}/.bashrc", std::env::var("HOME").unwrap_or_default());
        prompt = format!(
            "PS1=\"\\e[32m\\]┌──[Target:{}🚀🌐IP:{}🔥\\e[34m\\]Attacker:{}📡IP:{}\\e[32m\\]🏅Prize:{} points]\\\n└──╼[👾]\\\\[\\e[36m\\]\\$(pwd) $ \\[\\e[0m\\]\"",
            machine_info.machine.name,
            machine_info.ip,
            user_info.user.name,
            get_interface_ip("tun0").expect("Error on getting tun0 IP address").to_string(),
            machine_info.machine.points
        );
        prompt_field = "PS1=.*";
    } else if result.contains("fish") {
        file_bak = format!("{}/.config/fish/functions/fish_prompt.fish.htb.bak", std::env::var("HOME").unwrap_or_default());
        file = format!("{}/.config/fish/functions/fish_prompt.fish", std::env::var("HOME").unwrap_or_default());
        prompt = format!(
            r#"function fish_prompt
    set_color 00ff00
    echo -n "┌──[Target:{}🚀🌐IP:{}"
    set_color ff00d7
    echo -n "🔥Attacker:{}📡IP:{}"
    set_color 00ff00
    echo "🏅Prize:{} points]"
    set_color 00ff00
    echo -n "└──╼[👾]"
    set_color 00ffff
    echo (pwd) '$' (set_color normal)
end"#,
            machine_info.machine.name,
            machine_info.ip,
            user_info.user.name,
            get_interface_ip("tun0").expect("Error on getting tun0 IP address").to_string(),
            machine_info.machine.points
        );
    } else if result.contains("zsh") {
        file_bak = format!("{}/.zshrc.htb.bak", std::env::var("HOME").unwrap_or_default());
        file = format!("{}/.zshrc", std::env::var("HOME").unwrap_or_default());
        prompt = format!(
            "PROMPT=\"%F{{46}}┌──[Target:{}🚀🌐IP:{}🔥%F{{201}}Attacker:{}📡IP:{}%F{{46}}Prize:{} points]\"$'\\n'\"└──╼[👾]%F{{44}}%~ $%f \"" ,
            machine_info.machine.name,
            machine_info.ip,
            user_info.user.name,
            get_interface_ip("tun0").expect("Error on getting tun0 IP address").to_string(),
            machine_info.machine.points
        );
        prompt_field = "PROMPT=.*";
    }

    if !std::path::Path::new(&file_bak).exists() {
        std::fs::copy(&file_bak, &file).unwrap_or_default();
    }
    
    if result.contains("bash") && result.contains("zsh") {
        let file_content = std::fs::read_to_string(&file).unwrap_or_default();
        let new_file_content = file_content.replace(&prompt_field, &prompt);
        std::fs::write(&file, &new_file_content).unwrap_or_default();
    } else if result.contains("fish") {
        std::fs::write(&file, &prompt).unwrap_or_default();
    }
}

pub fn restore_shell() {
    let result = env::var("SHELL").unwrap_or_default();
    let mut file_bak = String::new();
    let mut file = String::new();

    if result.contains("bash") {
        file_bak = format!("{}/.bashrc.htb.bak", env::var("HOME").unwrap());
        file = format!("{}/.bashrc", env::var("HOME").unwrap());
    } else if result.contains("fish") {
        file_bak = format!("{}/.config/fish/functions/fish_prompt.fish.htb.bak", env::var("HOME").unwrap());
        file = format!("{}/.config/fish/functions/fish_prompt.fish", env::var("HOME").unwrap());
    } else if result.contains("zsh") {
        file_bak = format!("{}/.zshrc.htb.bak", env::var("HOME").unwrap());
        file = format!("{}/.zshrc", env::var("HOME").unwrap());
    }
    if fs::metadata(&file).is_ok() {
        fs::copy(&file_bak, &file).expect("Failed to copy file");
    }
}

pub fn display_target_info(machine_info: &PlayingMachine, user_info: &PlayingUser) {
    println!();
    println!("{}Our secret agent gathered some information about the target:{}", BYELLOW, RESET);
    println!("{}┌────────────────────────────────────────────────────┐{}", BGREEN, RESET);
    println!("{}| Target Name       : {}{}{}", BGREEN, BCYAN, machine_info.machine.name, RESET);
    println!("{}| Target OS         : {}{}{}", BGREEN, BCYAN, machine_info.os, RESET);
    println!("{}| Target IP         : {}{}{}", BGREEN, BCYAN, machine_info.ip, RESET);
    println!("{}| Points            : {}{}{}", BGREEN, BCYAN, machine_info.machine.points, RESET);
    println!("{}| Difficulty        : {}{}{}", BGREEN, BCYAN, machine_info.machine.difficulty_str, RESET);
    println!("{}| User Flag         : {}{}{}", BGREEN, BCYAN, machine_info.machine.user_pwn, RESET);
    println!("{}| Root Flag         : {}{}{}", BGREEN, BCYAN, machine_info.machine.root_pwn, RESET);
    println!("{}|────────────────────────────────────────────────────|{}", BGREEN, RESET);
    println!("{}| Attacker          : {}{}{}", BGREEN, RED, user_info.user.name, RESET);
    println!("{}| Attacker IP       : {}{}{}", BGREEN, RED, user_info.ip, RESET);
    println!("{}└────────────────────────────────────────────────────┘{}", BGREEN, RESET);
    println!();
    println!("{}The agent left this information in the console.{}\n", BYELLOW, RESET);
}

pub fn get_interface_ip(interface_name: &str) -> Option<String> {
    // Get a list of network interfaces
    let interfaces = datalink::interfaces();

    // Find the desired interface by name
    if let Some(interface) = interfaces.into_iter().find(|iface| iface.name == interface_name) {
        // Iterate through the IP addresses of the interface
        for addr in &interface.ips {
            match addr.ip() {
                IpAddr::V4(ipv4) => return Some(ipv4.to_string()),
                _ => (),
            }
        }
    } else {
        println!("Interface not found: {}", interface_name);
        std::process::exit(1);
    }
    
    None // Return None if interface not found or IP not found
}

pub fn print_banner() -> Result<(), Box<dyn std::error::Error>> {
    let encoded = "H4sIAAAAAAAAA+1byw6CMBC8+xW9aAxpWvVm/BQ9eefu5/sgkQIFCrR02N09mYmH6c50uxRWqbixC/vb43V+3k+366U8Vj/LeYjtQnFY6lgM8ZFl2QrSRJB1ED1PRrRlpEPMJ8hubTtJeYNGP14ajBNo7BJ6O0R5ssVZG0/QXW5bbFdoNK5rIspng3/QLf3GYwU0kgl3w2h8fcEoIU0rEDkEx1Wuo0Ajv0RHWjJGy02ID9BIi52oI0O2JNt/cLW15dRUdBFdNVaNRguNY14k9wmUb2toOYCdmyjW26KocrDqZUx24xN6CpmHMG93nDORn/HZl358pNW4MfWoofySoIsUnodSvsoDVKme2wLRRDQRTUQT0UQ0EU1Ek7iabPIyypdFYRTfG3s0eukQLPEadlJ0L8+sbhruF2SfwQtEf6ElKR3ifhHf+RiW0fW8QhC+IrHJ7mO5+7wO3Gw3Ng8xubecoTvYZQ8ea/XP/JA9cAWZ3KL0eWR4TFIclLGMDUTACC7aeqLVwN4RuAnJWeDtwfnLmsMbuwsrDnU9AAA=";

    // Decode using base64
    let mut child = Command::new("base64")
        .arg("-d")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(encoded.as_bytes())?;
    }
    let output = child.wait_with_output()?;

    // Decompress using gunzip
    let mut gunzip_child = Command::new("gunzip")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(ref mut stdin) = gunzip_child.stdin {
        stdin.write_all(&output.stdout)?;
    }
    let gunzip_output = gunzip_child.wait_with_output()?;

    if gunzip_output.status.success() {
        let decompressed = String::from_utf8_lossy(&gunzip_output.stdout).into_owned().replace("\\x1b", "\x1b"); // .replace is needed to apply the colors on the banner string
        
        let mut stdout = io::stdout();
        writeln!(stdout, "{}", decompressed)?;
    } else {
        eprintln!("'gunzip' command failed");
    }

    Ok(())
}

pub fn get_help() {
    // Display Help
    println!("Spawn and play Hack The Box machines directly on your system.");
    println!();
    println!("{} [-h] [-a] [-f] [-k] <set|reset|delete> [-m] <machine-name> [-l] <free|retired|starting> [-p] <true|false> [-r] [-s] [-u] [-v] <vpn-name>", env::args().nth(0).unwrap());
    println!();
    println!("Options:");
    println!("-a                            Print information about the current active machine.");
    println!("-f                            Submit a flag.");
    println!("-h                            Print this help.");
    println!("-k <set|reset|delete>         Set, reset or delete the Hack The Box App Key.");
    println!("-m <machine-name>             Specify the machine name to play.");
    println!("-l <free|retired|starting>    List free, retired or starting point machines.");
    println!("-p <true|false>               Set if the shell prompt should be changed.");
    println!("-r                            Reset the playing machine.");
    println!("-s                            Stop the playing machine.");
    println!("-u                            Update free machines in the Red Team menu.");
    println!("-v <vpn-name>                 Set a Hack The Box VPN.");
    println!();
    println!("Available VPN Servers:");
    print_vpn_sp_list();
    print_vpn_machine_list();
    println!();
    println!("Usage Examples:");
    println!("{} ", env::args().nth(0).unwrap());
    println!("{} -k set", env::args().nth(0).unwrap());
    println!("{} -l free", env::args().nth(0).unwrap());
    println!("{} -m RouterSpace", env::args().nth(0).unwrap());
    println!("{} -u", env::args().nth(0).unwrap());
    println!("{} -v EUFree1", env::args().nth(0).unwrap());
}

pub fn is_inside_container() -> bool {
    if let Ok(cgroup) = fs::read_to_string("/proc/1/cgroup") {
        cgroup.contains("/docker/") || cgroup.contains("/podman/")
    } else {
        false
    }
}

pub fn is_wsl() -> bool {
    if let Ok(uname) = fs::read_to_string("/proc/sys/kernel/osrelease") {
        uname.contains("Microsoft") || uname.contains("WSL")
    } else {
        false
    }
}

pub fn is_display_empty() -> bool {
    if let Ok(display_value) = env::var("DISPLAY") {
        display_value.is_empty()
    } else {
        false
    }
}

pub fn is_display_zero() -> bool {
    if let Ok(display_value) = env::var("DISPLAY") {
        display_value == ":0"
    } else {
        false
    }
}

pub fn htb_machines_to_flypie<T: CommonTrait>(machine_list: Vec<T>) -> String {
    let terminal = "gnome-terminal --";
    let fly_new = machine_list
        .iter()
        .map(|machine| {
            let home = env::var("HOME").unwrap();
            let machine_name = &machine.get_name().splitn(2, ' ').nth(1).unwrap(); //Remove the "[os icon] " from the machine name
            let machine_avatar = &machine.get_avatar();

            let avatar_url = format!("https://www.hackthebox.com{}", machine_avatar);
            let avatar_filename = format!(
                "{}/.local/share/icons/hackthebox/avatar/{}.png",
                home, machine_name
            );

            let shell = env::var("SHELL").unwrap();

            let machine_command = format!(
                "{} /usr/bin/bash -c 'htb-spawn {};'{}''",
                terminal, machine_name, shell
            );

            let response = reqwest::blocking::get(&avatar_url);
            if let Ok(mut r) = response {
                if r.status().is_success() {
                    let mut image_data = Vec::new();
                    r.copy_to(&mut image_data).unwrap();

                    let mut avatar_file = File::create(&avatar_filename).unwrap();
                    avatar_file.write_all(&image_data).unwrap();

                    Command::new("convert")
                        .args(&[&avatar_filename, "-resize", "200x", &avatar_filename])
                        .status()
                        .unwrap();
                } else {
                    println!("Image Couldn't be retrieved");
                }
            }

            format!(
                "{{\\\"name\\\":\\\"{}\\\",\\\"icon\\\":\\\"{}\\\",\\\"type\\\":\\\"Command\\\",\\\"data\\\":{{\\\"command\\\":\\\"{}\\\"}},\\\"angle\\\":-1}},",
                machine_name, avatar_filename, machine_command
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!("[{}]", &fly_new[..fly_new.len() - 1]) // Return the dconf string with all free machines and delete the last character (, comma)
}