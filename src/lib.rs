// use std::error::Error;

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
    pub fn build(args: &[String]) -> Result<Task, &'static str> {
        let mut todo_instance = Task {
            task: String::from("toBeFilledLater"),
            status: Status::Open,
            priority: Priority::B,
        };
        if args.len() == 1 {
            //TODO: print out all todos or open tui to show and navigate all todos
            return Err("too little for now");
        } else if args.len() == 2 {
            todo_instance = Task {
                task: args[1].clone(),
                ..todo_instance
            };
            return Ok(todo_instance);
        } else {
            return Err("too much for now");
        }
    }
}
