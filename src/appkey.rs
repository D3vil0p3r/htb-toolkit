use std::env;
use std::fs;
use std::io::ErrorKind;
use std::process::Command;

pub fn get_appkey() -> String {
    if is_inside_container() && is_display_empty() {
        let output = Command::new("cat")
            .arg("/run/secrets/htb-api")
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    String::from_utf8_lossy(&output.stdout).to_string()
                } else {
                    let error_output = String::from_utf8_lossy(&output.stderr);
                    eprintln!("'cat' command failed:\n{}", error_output);
                    String::new()
                }
            }
            Err(error) => {
                if error.kind() == ErrorKind::NotFound {
                    eprintln!("File not found");
                } else {
                    eprintln!("Error: {}", error);
                }
                String::new()
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

fn is_inside_container() -> bool {
    if let Ok(cgroup) = fs::read_to_string("/proc/1/cgroup") {
        cgroup.contains("/docker/") || cgroup.contains("/podman/")
    } else {
        false
    }
}

fn is_display_empty() -> bool {
    if let Ok(display_value) = env::var("DISPLAY") {
        display_value.is_empty()
    } else {
        false
    }
}