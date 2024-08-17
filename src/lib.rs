use std::error::Error;
use std::fs;

#[derive(Debug)]
pub enum Priority {
    A,
    B,
    C,
}

#[derive(Debug)]
pub enum Status {
    Done,
    Open,
}

#[derive(Debug)]
pub struct Task {
    pub task: String,
    pub priority: Priority,
    pub status: Status,
}

impl Task {
    /// Builds todo struct
    pub fn build(args: &[String], path: &str) -> Result<(), Box<dyn Error>> {
        let mut todo_instance = Task {
            task: String::new(),
            status: Status::Open,
            priority: Priority::B,
        };
        if args.len() == 1 {
            let _ = run(path).unwrap_or_else(|_| {
                println!("No todo file found. \nConsider: td init");
            });
            Ok(())
        } else if args.len() == 2 {
            todo_instance = Task {
                task: args[1].clone(),
                ..todo_instance
            };
            //TODO: call to write to end of file here
            let file = write_todo(path, todo_instance);
            return Ok(());
        } else {
            //return Into::into(Err("too much for now"));
            Ok(())
        }
    }
}

pub fn run(path: &str) -> Result<(), Box<dyn Error>> {
    let todos = fs::read_to_string(path)?;

    //TODO: read only the stuff after the second tab (\t) -> prob by for loop over todos per
    //line and appending each to a new string and then printing out that one
    println!("Todos: \n{}", todos);

    Ok(())
}

pub fn write_todo(path: &str, todo: Task) {}
