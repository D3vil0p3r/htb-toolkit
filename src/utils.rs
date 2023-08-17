use std::process::{Command, Stdio};
use std::io::{self, Write};
use std::env;
use crate::colors::*;
use crate::types::*;
use pnet::datalink;
use std::net::IpAddr;

pub fn change_shell(machine_info: &mut PlayingMachine, user_info: &mut PlayingUser) {
    let result = std::env::var("SHELL").unwrap_or_default();
    
    if result.contains("bash") {
        let file = format!("{}/.bashrc.htb.bak", std::env::var("HOME").unwrap_or_default());
        if !std::path::Path::new(&file).exists() {
            std::fs::copy(format!("{}/.bashrc", std::env::var("HOME").unwrap_or_default()), &file).unwrap_or_default();
        }
        let ps1 = format!(
            "PS1=\"\\e[32m\\]┌──[Target:{}🚀🌐IP:{}🔥\\e[34m\\]Attacker:{}📡IP:{}\\e[32m\\]🏅Prize:{} points]\\\n└──╼[👾]\\\\[\\e[36m\\]\\$(pwd) $ \\[\\e[0m\\]\"",
            machine_info.name,
            machine_info.ip,
            user_info.name,
            get_interface_ip("tun0").expect("Error on getting tun0 IP address").to_string(),
            machine_info.points
        );
        let bashrc = format!("{}/.bashrc", std::env::var("HOME").unwrap_or_default());
        let bashrc_content = std::fs::read_to_string(&bashrc).unwrap_or_default();
        let new_bashrc_content = bashrc_content.replace("PS1=.*", &ps1);
        std::fs::write(&bashrc, &new_bashrc_content).unwrap_or_default();
        let _ = std::process::Command::new("bash").arg("-c").arg("source ~/.bashrc").output();
    } else if result.contains("fish") {
        let file = format!("{}/.config/fish/functions/fish_prompt.fish.htb.bak", std::env::var("HOME").unwrap_or_default());
        if !std::path::Path::new(&file).exists() {
            let _ = std::fs::rename(format!("{}/.config/fish/functions/fish_prompt.fish", std::env::var("HOME").unwrap_or_default()), &file);
        }
        let fish_prompt = format!(
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
            machine_info.name,
            machine_info.ip,
            user_info.name,
            get_interface_ip("tun0").expect("Error on getting tun0 IP address").to_string(),
            machine_info.points
        );
        let fish_prompt_file = format!("{}/.config/fish/functions/fish_prompt.fish", std::env::var("HOME").unwrap_or_default());
        std::fs::write(&fish_prompt_file, &fish_prompt).unwrap_or_default();
    } else if result.contains("zsh") {
        let file = format!("{}/.zshrc.htb.bak", std::env::var("HOME").unwrap_or_default());
        if !std::path::Path::new(&file).exists() {
            std::fs::copy(format!("{}/.zshrc", std::env::var("HOME").unwrap_or_default()), &file).unwrap_or_default();
        }
        let prompt = format!(
            "PROMPT=\"%F{{46}}┌──[Target:{}🚀🌐IP:{}🔥%F{{201}}Attacker:{}📡IP:{}%F{{46}}Prize:{} points]\"$'\\n'\"└──╼[👾]%F{{44}}%~ $%f \"" ,
            machine_info.name,
            machine_info.ip,
            user_info.name,
            get_interface_ip("tun0").expect("Error on getting tun0 IP address").to_string(),
            machine_info.points
        );
        let zshrc = format!("{}/.zshrc", std::env::var("HOME").unwrap_or_default());
        let zshrc_content = std::fs::read_to_string(&zshrc).unwrap_or_default();
        let new_zshrc_content = zshrc_content.replace("PROMPT=.*", &prompt);
        std::fs::write(&zshrc, &new_zshrc_content).unwrap_or_default();
    }
}

pub fn display_target_info(machine_info: &PlayingMachine, user_info: &PlayingUser) {
    println!();
    println!("{}Our secret agent gathered some information about the target:{}", BYELLOW, RESET);
    println!("{}┌────────────────────────────────────────────────────┐{}", BGREEN, RESET);
    println!("{}| Target Name       : {}{}{}", BGREEN, BCYAN, machine_info.name, RESET);
    println!("{}| Target OS         : {}{}{}", BGREEN, BCYAN, machine_info.os, RESET);
    println!("{}| Target IP         : {}{}{}", BGREEN, BCYAN, machine_info.ip, RESET);
    println!("{}| Points            : {}{}{}", BGREEN, BCYAN, machine_info.points, RESET);
    println!("{}| Difficulty        : {}{}{}", BGREEN, BCYAN, machine_info.difficulty_str, RESET);
    println!("{}| User Flag         : {}{}{}", BGREEN, BCYAN, machine_info.user_pwn, RESET);
    println!("{}| Root Flag         : {}{}{}", BGREEN, BCYAN, machine_info.root_pwn, RESET);
    println!("{}|────────────────────────────────────────────────────|{}", BGREEN, RESET);
    println!("{}| Attacker          : {}{}{}", BGREEN, RED, user_info.name, RESET);
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
    println!("HTB Play allows you to spawn Hack The Box machines from CLI.");
    println!();
    println!("{} [-h] [-a] [-f] [-m] <machine-name> [-l] [-r] [-s] [-t] [-u] [-v]", env::args().nth(0).unwrap());
    println!();
    println!("Options:");
    println!("a     List active machines.");
    println!("f     Submit a flag.");
    println!("h     Print this Help.");
    println!("m     Specify the machine name to play.");
    println!("l     List retired machines.");
    println!("r     Reset the playing machine.");
    println!("s     Stop the playing machine.");
    println!("t     List starting point machines.");
    println!("u     Update the Hack The Box App Key.");
    println!("v     Set a Hack The Box VPN.");
    println!();
    println!("Usage Examples:");
    println!("{} ", env::args().nth(0).unwrap());
    println!("{} -a", env::args().nth(0).unwrap());
    println!("{} -m RouterSpace", env::args().nth(0).unwrap());
}