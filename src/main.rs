use std::env;
use td;

const HELP: &str =  "A simple to use, minimal and idomatic rust todo cli\n \n 1. Usage: td [EXE] \"task\" \n \n $ td \"write usage guide\" \n \n 2. Usage: td [EXE] \n ";

fn main() {
    let args: Vec<String> = env::args().collect();
    //TODO: make that dynamic instead of hardcoded for me
    let todo_path = td::get_todo_file_path();
    //man page ordered on wish
    if args.len() > 2 {
        println!("{}", HELP);
    } else {
        let _ = td::Task::build(&args, todo_path).unwrap_or_else(|err| {
            println!("Problem parsing arguments: {err}");
        });
    }
}
