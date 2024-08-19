use crossterm::{
    cursor, event,
    style::{self, Print, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode, size, window_size, WindowSize},
    ExecutableCommand, QueueableCommand,
};
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use std::{env, io::Read};

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
    pub fn build(args: &[String], path: PathBuf) -> Result<(), Box<dyn Error>> {
        assert!(args.len() < 3);

        if args.len() == 1 {
            let _ = main_tui(path);
            //let _ = prints_todo(path).unwrap_or_else(|_| {
            //    println!("No todo file found. \nConsider: td init");
            //});
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
fn prints_todo(path: PathBuf) -> Result<(), Box<dyn Error>> {
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

fn main_tui(path: PathBuf) -> io::Result<()> {
    let mut stdout = io::stdout();

    // I dont think I need this
    let _ = enable_raw_mode();

    let mut file = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(path)
        .unwrap();

    // init empty string
    let mut contents: String = String::new();
    //return the amount of bytes appended to contents string <- useless
    let _ = file.read_to_string(&mut contents);

    //let mut count = 0;

    let (width, height) = size().unwrap();
    loop {
        // Move the cursor to position (5, 5)
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        stdout
            .execute(cursor::MoveTo(width / 2, height / 2))
            .unwrap();

        for line in contents.lines() {
            //let split_lines: Vec<&str> = line.split('\t').collect();
            if let Some((status, task)) = line.split_once('\t') {
                if status == "[ ]" {
                    //println!("{}\t{}", status, task);
                    let task_to_print = format!("{}\t{}", status, task);
                    stdout.queue(Print(&task_to_print)).unwrap();
                }
            }
        }
        stdout.flush()?;

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                event::Event::Key(event) => match event.code {
                    event::KeyCode::Char('q') => break,
                    _ => (),
                },
                _ => (),
                // Event::Resize(width, height) => println!("New size {}x{}", width, height),
            }
        }
        // Add a small delay to reduce CPU usage and prevent flickering
        sleep(Duration::from_millis(50));
    }
    let _ = disable_raw_mode();
    Ok(())
}

/// appends new todo to the end of todo file
pub fn write_todo(path: PathBuf, todo: Task) {
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

pub fn get_todo_file_path() -> PathBuf {
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("."));
    let mut path = PathBuf::from(home_dir);
    path.push(".todo_app");
    fs::create_dir_all(&path).expect("Failed to create directory");
    path.push("todos.txt");
    path
}
