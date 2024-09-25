use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use td;

use std::fs;
use std::path::PathBuf;

const HELP: &str =  "A simple to use, minimal and idomatic rust todo cli\n \n 1. Usage: td [EXE] \"task\" \n \n $ td \"write usage guide\" \n \n 2. Usage: td [EXE] \n ";

const HELP_PATH: &str = "Usage: td [EXE] --file x \nWhere x is an entry in config.yml";

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
            "file" | "f" | "--file" | "-f" => println!("{HELP_PATH}"),
            _ => {
                let todo_path = get_todo_file_path("default".to_string());
                let _ = td::Task::build(&args, todo_path).unwrap_or_else(|err| {
                    println!("Problem parsing arguments: {err}");
                });
            }
        },
        3 => match args[1].as_str() {
            "file" | "f" | "--file" | "-f" => {
                // opens up tui let todo_path = get_todo_file_path(args[2].clone());
                let todo_path = get_todo_file_path(args[2].clone());
                let _ = td::main_tui(todo_path);
                //TODO: perhaps delete args[1-2] and pass that to build like below
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
                // opens up tui let todo_path = get_todo_file_path(args[2].clone());
                let todo_path = get_todo_file_path(args[2].clone());
                args.remove(1);
                args.remove(1);
                let _ = td::Task::build(&args, todo_path).unwrap_or_else(|err| {
                    println!("Problem parsing arguments: {err}");
                });
                //TODO: perhaps delete args[1-2] and pass that to build like below
            }
            _ => {
                let todo_path = get_todo_file_path("default".to_string());
                let _ = td::Task::build(&args, todo_path).unwrap_or_else(|err| {
                    println!("Problem parsing arguments: {err}");
                });
            }
        },
    }
    //if args.len() == 1 {
    //    // opens up tui
    //    let todo_path = get_todo_file_path("default".to_string());
    //    let _ = td::main_tui(todo_path);
    //} else if args.len() == 2 {
    //    if args[1] == "help" || args[1] == "h" || args[1] == "--help" || args[1] == "-h" {
    //        println!("{}", HELP);
    //    }
    //    if args[1] == "path" || args[1] == "p" || args[1] == "--path" || args[1] == "-p" {
    //        // opens up tui let todo_path = get_todo_file_path(args[2].clone());
    //        let todo_path = get_todo_file_path(args[2]);
    //        let _ = td::main_tui(todo_path);
    //        //TODO: perhaps delete args[1-2] and pass that to build like below
    //    }
    //} else {
    //    let todo_path = get_todo_file_path("default".to_string());
    //    let _ = td::Task::build(&args, todo_path).unwrap_or_else(|err| {
    //        println!("Problem parsing arguments: {err}");
    //    });
    //}
}

fn get_todo_file_path(path_to_use: String) -> PathBuf {
    let config_path = PathBuf::from("config.yaml");
    let config_contents = fs::read_to_string(config_path)
        .expect("Couldnt read config file. Make sure config.yml exists in root");

    let config: Config =
        serde_yml::from_str(&config_contents).expect("Could not deserialize config.yml");

    // Determine home directory based on the operating system
    #[cfg(target_os = "windows")]
    let home_dir = env::var("USERPROFILE").unwrap_or_else(|_| String::from("."));

    #[cfg(not(target_os = "windows"))]
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("."));

    let mut path = PathBuf::from(home_dir);
    path.push(config.root);
    fs::create_dir_all(&path).expect("Failed to create directory");
    match config.file.get(&path_to_use) {
        Some(x) => {
            path.push(x);
        }
        None => (),
    }
    path
}
