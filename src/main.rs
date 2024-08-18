use std::env;
use td;

fn main() {
    let args: Vec<String> = env::args().collect();
    //let todo_path: &str = "~/todos.txt";
    let todo_path: &str = "/home/fynn/.todos.txt";
    if args.len() > 2 {
        println!(
            "A simple to use, minimal and idomatic rust todo cli\n
            \n
            1. Usage: td [EXE] \"task\" \n
            \n
            $ td \"write usage guide\" \n
            \n
            2. Usage: td [EXE] \n
            "
        );
    } else {
        let _ = td::Task::build(&args, todo_path).unwrap_or_else(|err| {
            println!("Problem parsing arguments: {err}");
        });
    }
}
