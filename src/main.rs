use serde::{Deserialize, Serialize};
use std::env;
use td;

use std::fs;
use std::path::PathBuf;

const HELP: &str =  "A simple to use, minimal and idomatic rust todo cli\n \n 1. Usage: td [EXE] \"task\" \n \n $ td \"write usage guide\" \n \n 2. Usage: td [EXE] \n ";

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    path: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    //TODO: make that dynamic instead of hardcoded for me
    let todo_path = get_todo_file_path();
    //man page ordered on wish
    if args.len() == 1 {
        // opens up tui
        let _ = td::main_tui(todo_path);
    } else if args.len() == 2
        && (args[1] == "help" || args[1] == "h" || args[1] == "-help" || args[1] == "-h")
    {
        // prints usage
        println!("{}", HELP);
    } else {
        let _ = td::Task::build(&args, todo_path).unwrap_or_else(|err| {
            println!("Problem parsing arguments: {err}");
        });
    }
}

fn get_todo_file_path() -> PathBuf {
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
    path.push(config.path);
    fs::create_dir_all(&path).expect("Failed to create directory");
    path.push("todos.txt");
    path
}
