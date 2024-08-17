use std::cmp::Ordering;
use std::env;
use td;

fn main() {
    let args: Vec<String> = env::args().collect();
    // with any input the len is one
    let default_len = 2;
    match &args.len().cmp(&default_len) {
        Ordering::Less => {
            //TODO: print out list of todos
        }
        Ordering::Equal => {
            // let todo_instance = td::Status::Open(td::PrioOfTask::B(args[1].clone()));
            let todo_instance = td::Task {
                task: args[1].clone(),
                status: td::Status::Open,
                priority: td::Priority::B,
            };
            println!("{:?}", todo_instance);
        }
        Ordering::Greater => {
            //TODO: either usage guide or selecting prio
        }
    }
}
