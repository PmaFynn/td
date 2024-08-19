use crossterm::{
    cursor, event,
    style::{self, Print, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode, size, window_size, WindowSize},
    ExecutableCommand, QueueableCommand,
};
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use std::{env, io::Read};
use std::{error::Error, io::stdout};
use std::{fs, os::unix::process};

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

struct Win {
    width: u16,
    height: u16,
}

fn main_tui(path: PathBuf) -> io::Result<()> {
    //TODO: perhaps use alternate screen
    let mut stdout = io::stdout();

    // I dont think I need this
    let _ = enable_raw_mode();

    let mut file = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(path)
        .unwrap();

    let mut contents: String = String::new();
    //returns the amount of bytes appended to contents string <- useless
    let _ = file.read_to_string(&mut contents);

    loop {
        let current_window = Win {
            width: size().unwrap().0,
            height: size().unwrap().1,
        };
        // Move the cursor to position (5, 5)
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;
        stdout
            .queue(cursor::MoveTo(current_window.width / 2, 0))
            .unwrap();
        stdout.queue(Print("Open\tDone")).unwrap();

        //TODO: height should not be 0 here but at least the height of the first line
        stdout.queue(cursor::MoveTo(0, 0)).unwrap();
        stdout.queue(cursor::MoveToNextLine(2)).unwrap();
        for line in contents.lines() {
            //let split_lines: Vec<&str> = line.split('\t').collect();
            if let Some((status, task)) = line.split_once('\t') {
                if status == "[ ]" {
                    //println!("{}\t{}", status, task);
                    let task_to_print = format!("{}\t{}", status, task);
                    stdout.queue(Print(&task_to_print)).unwrap();
                    stdout.queue(cursor::MoveToNextLine(1)).unwrap();
                }
            }
        }
        stdout.flush()?;

        match event::read()? {
            event::Event::Key(event) => match event.code {
                event::KeyCode::Char('q') => break,
                _ => {}
            },
            _ => {} // Event::Resize(width, height) => println!("New size {}x{}", width, height),
        }
        // Add a small delay to reduce CPU usage and prevent flickering
        sleep(Duration::from_millis(50));
    }
    let _ = disable_raw_mode();
    Ok(())
}

fn prints_current(path: PathBuf) -> io::Result<()> {
    let mut stdout = io::stdout();

    // I dont think I need this
    let _ = enable_raw_mode();

    let mut file = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(path)
        .unwrap();

    let mut contents: String = String::new();
    //returns the amount of bytes appended to contents string <- useless
    let _ = file.read_to_string(&mut contents);

    let current_window = Win {
        width: size().unwrap().0,
        height: size().unwrap().1,
    };
    // Move the cursor to position (5, 5)
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    stdout
        .queue(cursor::MoveTo(current_window.width / 2, 0))
        .unwrap();
    stdout.queue(Print("Open\tDone")).unwrap();

    //TODO: height should not be 0 here but at least the height of the first line
    stdout.queue(cursor::MoveTo(0, 0)).unwrap();
    stdout.queue(cursor::MoveToNextLine(2)).unwrap();
    for line in contents.lines() {
        //let split_lines: Vec<&str> = line.split('\t').collect();
        if let Some((status, task)) = line.split_once('\t') {
            if status == "[ ]" {
                //println!("{}\t{}", status, task);
                let task_to_print = format!("{}\t{}", status, task);
                stdout.queue(Print(&task_to_print)).unwrap();
                stdout.queue(cursor::MoveToNextLine(1)).unwrap();
            }
        }
    }
    stdout.flush()?;

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
