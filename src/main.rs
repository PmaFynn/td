use std::env;
use std::process;
use td;

fn main() {
    let args: Vec<String> = env::args().collect();
    //let todo_path: &str = "~/todos.txt";
    let todo_path: &str = "/home/fynn/todos.txt";
    if args.len() == 1 {
        td::run(todo_path).unwrap_or_else(|_| {
            println!("No todo file found. \nConsider: td init");
        });
    } else {
        let _todo_instance = td::Task::build(&args, todo_path).unwrap_or_else(|err| {
            println!("Problem parsing arguments: {err}");
            process::exit(1);
        });
    }
    //match &args.len().cmp(&default_len) {
    //    Ordering::Less => {
    //        //TODO: print out list of todos
    //    }
    //    Ordering::Equal => {
    //        // let todo_instance = td::Status::Open(td::PrioOfTask::B(args[1].clone()));
    //        let todo_instance = td::Task {
    //            task: args[1].clone(),
    //            status: td::Status::Open,
    //            priority: td::Priority::B,
    //        };
    //        println!("{:?}", todo_instance);
    //    }
    //    Ordering::Greater => {
    //        //TODO: either usage guide or selecting prio
    //    }
    //}
}
