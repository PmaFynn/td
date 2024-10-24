use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use td;

use std::fs;
use std::path::PathBuf;

const HELP: &str =  "A simple to use, minimal and idomatic rust todo cli\n \n 1. Usage: td [EXE] \"task\" \n \n $ td \"write usage guide\" \n \n 2. Usage: td [EXE] \n ";

const HELP_FILE: &str = "Usage: td [EXE] --file x \nWhere x is an entry in config.yml";

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    file: HashMap<String, String>,
    root: String,
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    //TODO: make that dynamic instead of hardcoded for me
    //man page ordered on wish
    match args.len() {
        1 => {
            let todo_path = get_todo_file_path("default".to_string());
            let _ = td::main_tui(todo_path);
        }
        2 => match args[1].as_str() {
            "help" | "h" | "--help" | "-h" => println!("{HELP}"),
            "file" | "f" | "--file" | "-f" => {
                println!("{HELP_FILE}\n");
                print_possible_td_store();
            }
            _ => {
                let todo_path = get_todo_file_path("default".to_string());
                let _ = td::Task::build(&args, todo_path).unwrap_or_else(|err| {
                    println!("Problem parsing arguments: {err}");
                });
            }
        },
        3 => match args[1].as_str() {
            "file" | "f" | "--file" | "-f" => {
                let todo_path = get_todo_file_path(args[2].clone());
                let _ = td::main_tui(todo_path);
            }
            _ => {
                let todo_path = get_todo_file_path("default".to_string());
                let _ = td::Task::build(&args, todo_path).unwrap_or_else(|err| {
                    println!("Problem parsing arguments: {err}");
                });
            }
        },
        _ => match args[1].as_str() {
            "file" | "f" | "--file" | "-f" => {
                let todo_path = get_todo_file_path(args[2].clone());
                //the following two lines delete the second and third arg entry
                args.remove(1);
                args.remove(1);
                let _ = td::Task::build(&args, todo_path).unwrap_or_else(|err| {
                    println!("Problem parsing arguments: {err}");
                });
            }
            _ => {
                let todo_path = get_todo_file_path("default".to_string());
                let _ = td::Task::build(&args, todo_path).unwrap_or_else(|err| {
                    println!("Problem parsing arguments: {err}");
                });
            }
        },
    }
}

fn print_possible_td_store() {
    // Determine home directory based on the operating system
    #[cfg(target_os = "windows")]
    let home_dir = env::var("USERPROFILE").unwrap_or_else(|_| String::from("."));

    #[cfg(not(target_os = "windows"))]
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("."));
    let config_dir = PathBuf::from(&home_dir).join(".config/td");
    let config_path = config_dir.join("config.yaml");

    if !config_path.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
        let default_config = r#"root: ".todo_app/"
file:
    default: "main.txt"
    example: "example.txt"
"#;
        fs::write(&config_path, default_config).expect("Failed to write default config");
    }

    let config_contents = fs::read_to_string(&config_path).expect("Failed to read config file");
    let config: Config =
        serde_yml::from_str(&config_contents).expect("Could not deserialize config.yml");
    println!("All of the following are possible, already set up, td stores\nwith default being--as the name might suggest--the default store\nthat will be accessed with out the -f flag:\n");
    println!("\"default\"");
    for (key, _) in config.file {
        if key != "default" {
            println!("{:?}", key);
        }
    }
}

fn get_todo_file_path(path_to_use: String) -> PathBuf {
    // Determine home directory based on the operating system
    #[cfg(target_os = "windows")]
    let home_dir = env::var("USERPROFILE").unwrap_or_else(|_| String::from("."));

    #[cfg(not(target_os = "windows"))]
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("."));
    let config_dir = PathBuf::from(&home_dir).join(".config/td");
    let config_path = config_dir.join("config.yaml");

    if !config_path.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
        let default_config = r#"root: ".todo_app/"
file:
    default: "main.txt"
    example: "example.txt"
"#;
        fs::write(&config_path, default_config).expect("Failed to write default config");
    }

    let config_contents = fs::read_to_string(&config_path).expect("Failed to read config file");
    let config: Config =
        serde_yml::from_str(&config_contents).expect("Could not deserialize config.yml");

    let mut path = PathBuf::from(&home_dir).join(config.root);
    fs::create_dir_all(&path).expect("Failed to create directory");
    match config.file.get(&path_to_use) {
        Some(x) => {
            path.push(x);
        }
        None => {
            println!(
                "As of yet, there is no todo file called: {}.txt in the specified location.\nFeel free to add it in your .config/td/config.yaml",
                &path_to_use
            );
            std::process::exit(1)
        }
    }
    path
}
