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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Handle the case where no command-line arguments are provided
        println!("Usage: {} [options]", args[0]);
        return;
    }

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
            let mtype = "active";
            list_machines(mtype);
        }
        "-f" => {
            submit_flag();
        }
        "-m" => {
            if args.len() < 3 {
                println!("Usage: {} -m <machine-name>", args[0]);
            } else {
                play_machine(&args[2]);
            }
        }
        "-l" => {
            let mtype = "retired";
            list_machines(mtype);
        }
        "-r" => {
            reset_machine();
        }
        "-s" => {
            stop_machine();
        }
        "-t" => {
            let mtype = "starting";
            list_sp_machines(mtype);
        }
        "-u" => {
            update_app_key();
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