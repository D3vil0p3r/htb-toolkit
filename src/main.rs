//mod only on main.rs or lib.rs , not on siblings. On siblings, use crate::
mod api;
mod appkey;
mod colors;
mod list;
mod manage;
mod play;
mod types;
mod utils;
use std::env;
use crate::list::*;
use crate::manage::*;
use crate::play::*;
use crate::utils::*;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Handle the case where no command-line arguments are provided
        println!("Usage: {} [options]", args[0]);
        return;
    }

    // Initialization
    // Create avatar folder for free machine icons
    let home = env::var("HOME").unwrap_or_default();
    let folder_path = format!("{}/.local/share/icons/hackthebox/avatar", home);

    if !fs::metadata(&folder_path).is_ok() { // Create the folder only if it does not exist
        if let Err(err) = fs::create_dir_all(&folder_path) {
            eprintln!("Error creating folder: {}", err);
        } else {
            println!("Folder created successfully.");
        }
    }
    
    // Create HTB config file if not existing
    let htb_config = format!("{}/.htb.conf", home);

    let file = Path::new(&htb_config);
    if !file.exists() {
        let lines = ["# HTB configuration file.\n\n", "# Enable/Disable shell prompt change\n", "prompt_change=false\n"];
        fs::write(&htb_config, lines.join(""))
            .expect("Failed to create HTB config file");
    }

    // Initialize Xorg in WSL for secret-tool popup window
    if is_wsl() && is_display_zero() {
        Command::new("sh")
            .arg("-c")
            .arg("source /etc/X11/xinit/xinitrc.d/50-systemd-user.sh 2> /dev/null")
            .status()
            .expect("Failed to execute shell command");
    }
    //////////////////////////////////

    match args[1].as_str() {
        "-h" => {
            match print_banner() {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("Error: {}", error);
                }
            }
            get_help();
        }
        "-a" => {
            get_active_machine_info();
        }
        "-f" => {
            submit_flag();
        }
        "-k" => {
            if args.len() < 3 || (args[2] != "set" && args[2] != "reset" && args[2] != "delete") {
                println!("Usage: {} -k <set|reset|delete>", args[0]);
            } else {
                manage_app_key(&args[2]);
            }
        }
        "-m" => {
            if args.len() < 3 {
                println!("Usage: {} -m <machine-name>", args[0]);
            } else {
                play_machine(&args[2]);
            }
        }
        "-l" => {
            if args.len() < 3 || (args[2] != "free" && args[2] != "retired" && args[2] != "starting") {
                println!("Usage: {} -l <free|retired|starting>", args[0]);
            } else if args[2] == "free" || args[2] == "retired" {
                list_machines(&args[2]);
            } else if args[2] == "starting" {
                list_sp_machines();
            } 
        }
        "-p" => {
            if args.len() < 3 || (args[2] != "true" && args[2] != "false") {
                println!("Usage: {} -p <true|false>", args[0]);
            } else {
                prompt_setting(&args[2]);
            }
        }
        "-r" => {
            reset_machine();
        }
        "-s" => {
            stop_machine();
        }
        "-u" => {
            update_machines();
        }
        "-v" => {
            set_vpn();
        }
        _ => {
            match print_banner() {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("Error: {}", error);
                }
            }
            println!("Invalid command: {}", args[1]);
            get_help();
        }
    }
}