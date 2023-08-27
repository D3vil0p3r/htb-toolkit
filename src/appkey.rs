use crate::utils::*;
use std::fs;
use std::io::{self, Read};
use std::process::{Command, Stdio};

fn read_file_contents(path: &str) -> Result<String, io::Error> {
    let mut content = String::new();
    let mut file = fs::File::open(path)?;
    file.read_to_string(&mut content)?;
    Ok(content)
}

pub fn get_appkey() -> String {
    if is_inside_container() && is_display_empty() {
        let secret_path = "/run/secrets/htb-api";
        
        match read_file_contents(secret_path) {
            Ok(secret_content) => {
                let result_content = secret_content.replace("\n", "");
                result_content
            }
            Err(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    eprintln!("File not found");
                    String::new()
                } else {
                    eprintln!("Error: {}", error);
                    String::new()
                }
            }
        }
    } else {
        let output = Command::new("secret-tool")
            .arg("lookup")
            .arg("htb-api")
            .arg("user-htb-api")
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).to_string()
                } else {
                    let error_output = String::from_utf8_lossy(&output.stderr);
                    eprintln!("'secret-tool' command failed:\n{}", error_output);
                    String::new()
                }
            }
            Err(error) => {
                eprintln!("Error: {}", error);
                String::new()
            }
        }
    }
}

pub fn set_appkey() {
    let appkey = get_appkey();

    if appkey.is_empty() {
        if is_inside_container() && is_display_empty() {
            println!("API token not set. On the host machine, store the HTB API token in the htb-api-file and run: [docker|podman] secret create htb-api htb-api-file");
            std::process::exit(1);
        } else {
            println!("Hack The Box API Key not set. Please, insert your App Token after the 'Password' label, it will be stored in a secure keyring.");
            let store_command = "secret-tool";
            let store_args = ["store", "--label='HTB API key'", "htb-api", "user-htb-api"];

            let mut store_process = Command::new(store_command)
                .args(&store_args)
                .stdin(Stdio::inherit()) // Pass stdin from parent process
                .stdout(Stdio::inherit()) // Pass stdout to parent process
                .stderr(Stdio::inherit()) // Pass stderr to parent process
                // They are needed to invoke the prompt to type the App Token
                .spawn()
                .expect("Failed to execute secret-tool command");

            let store_result = store_process.wait();

            if store_result.is_ok() {
                println!("Hack The Box App Token successfully stored.");
            } else {
                eprintln!("Error storing API Key: {:?}", store_result);
                std::process::exit(1);
            }
        }
    } else {
        println!("Hack The Box App Token already set.");
    }
}

pub fn reset_appkey() {
    delete_appkey();
    set_appkey();
}

pub fn delete_appkey() {
    let appkey = get_appkey();

    if !appkey.is_empty() {
        if is_inside_container() && is_display_empty() {
            println!("You are in a container. For deleting the API token on the host machine, run: [docker|podman] secret rm htb-api");
            std::process::exit(1);
        } else {
            let clear_command = "secret-tool";
            let clear_args = ["clear", "htb-api", "user-htb-api"];

            let clear_output = Command::new(clear_command)
                .args(&clear_args)
                .output()
                .expect("Failed to execute secret-tool command");

            if clear_output.status.success() {
                println!("Hack The Box API Key successfully deleted.");
            } else {
                let error_output = String::from_utf8_lossy(&clear_output.stderr);
                eprintln!("Error deleting API Key:\n{}", error_output);
            }
        }
    } else {
        println!("Hack The Box App Token does not exist. Cannot delete.");
    }
}