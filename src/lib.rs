use std::error::Error;
use std::fs;
use std::io::Write;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum Status {
    Done,
    Open,
}

#[derive(Debug)]
pub struct Task {
    pub task: String,
    pub status: Status,
}

impl Task {
    /// Builds todo struct
    pub fn build(args: &[String], path: &str) -> Result<(), Box<dyn Error>> {
        assert!(args.len() < 3);

        if args.len() == 1 {
            let _ = prints_todo(path).unwrap_or_else(|_| {
                println!("No todo file found. \nConsider: td init");
            });
            Ok(())
        } else {
            let todo_instance = Task {
                task: args[1].clone(),
                status: Status::Open,
            };
            write_todo(path, todo_instance);
            return Ok(());
        }
    }
}

/// prints current todos to std_out
pub fn prints_todo(path: &str) -> Result<(), Box<dyn Error>> {
    let todos = fs::read_to_string(path)?;

    //TODO: make it a terminal user interface with crossterm i think idk -> start by clearing the
    //terminal before displaying the todos
    for line in todos.lines() {
        //let split_lines: Vec<&str> = line.split('\t').collect();
        if let Some((status, task)) = line.split_once('\t') {
            if status == "[ ]" {
                println!("{}\t{}", status, task);
            }
        }
    }
    Ok(())
}

/// appends new todo to the end of todo file
pub fn write_todo(path: &str, todo: Task) {
    //TODO: use write(true) instead of append(true) and rewrite the entire file similar to the
    //run() function. Further, print the line number at the uttermost left which we might be able
    //to use to delete or set todos as done
    let file = fs::OpenOptions::new()
        .append(true)
        .read(true)
        .create(true)
        .open(path);

    match file {
        Ok(mut file) => {
            let insert = format!("[ ]\t{}", todo.task);
            writeln!(file, "{}", insert).expect("idk");
        }
        Err(i) => println!("Error writing to file: {}", i),
    }
}
