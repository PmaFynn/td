use crossterm::{
    cursor, event,
    style::{self, style, Attribute, Color, Print, PrintStyledContent, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode, size, window_size, WindowSize},
    ExecutableCommand, QueueableCommand,
};
use std::time::Duration;
use std::{env, io::Read};
use std::{error::Error, io::stdout};
use std::{
    fmt::format,
    io::{self, Write},
};
use std::{fs, os::unix::process};
use std::{path::PathBuf, u16};
use std::{thread::sleep, usize};

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
    cols: u16,
    rows: u16,
}

struct Pos {
    row: u16,
    col: u16,
    status: Status,
}

impl Pos {
    fn set_pos(&mut self, col: u16, row: u16) -> &mut Self {
        self.col = col;
        self.row = row;
        self
    }
    fn one_down(&mut self, max_row: u16) -> &mut Self {
        if self.row < max_row {
            self.row += 1;
        }
        self
    }
    fn one_up(&mut self) -> &mut Self {
        if self.row > 2 {
            self.row -= 1;
        }
        self
    }
}

fn main_tui(path: PathBuf) -> io::Result<()> {
    //TODO: perhaps use alternate screen
    let mut stdout = io::stdout();

    // I dont think I need this
    let _ = enable_raw_mode();
    stdout.queue(cursor::Hide)?;

    let mut file = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(path)
        .unwrap();

    let mut exit = true;

    let mut contents: String = String::new();
    //returns the amount of bytes appended to contents string <- useless
    let _ = file.read_to_string(&mut contents);

    let mut pos = Pos {
        row: 2,
        col: 1,
        status: Status::Open,
    };

    let mut x_todo = 0;
    //let mut x_todo = contents.lines().count();

    while exit {
        let current_window = Win {
            rows: size().unwrap().0,
            cols: size().unwrap().1,
        };
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;

        let headline: &str = "Open\tDone";
        let mut open_styled = "Open"
            .with(Color::Yellow)
            .on(Color::Blue)
            .attribute(Attribute::Bold);
        let mut done_styled = "Done"
            .with(Color::Yellow)
            .on(Color::Blue)
            .attribute(Attribute::Bold);

        //FIX: does not appear in the middle
        stdout
            //.queue(cursor::MoveTo(
            //    (current_window.cols / 2) - (headline.len() / 2) as u16,
            //    0,
            //))
            .queue(cursor::MoveTo(
                (current_window.cols / 2) - (headline.len() / 2) as u16,
                0,
            ))
            .unwrap();

        //TODO: current_window.cols -> each col is a character so str.len() of 5 is 5 cols -> fix
        //it
        stdout.queue(PrintStyledContent(open_styled)).unwrap();

        //TODO: height should not be 0 here but at least the height of the first line
        stdout.queue(cursor::MoveTo(1, 1)).unwrap();
        stdout.queue(cursor::MoveToNextLine(1)).unwrap();
        x_todo = 0;
        for (i, line) in contents.lines().enumerate() {
            //let split_lines: Vec<&str> = line.split('\t').collect();

            if let Some((status, task)) = line.split_once('\t') {
                if true {
                    if status == "[ ]" {
                        //println!("{}\t{}", status, task);
                        //println!("i is {}, while pos.row is {}", i, pos.row - 2);
                        x_todo += 1;
                        let task_to_print = format!("{}\t{}", status, task);
                        if pos.row == (i + 2) as u16 {
                            let style = style(&task_to_print).with(Color::Black).on(Color::Grey);
                            stdout
                                .queue(PrintStyledContent(style))
                                .expect("failed to print styled line");
                        } else {
                            stdout.queue(Print(&task_to_print)).unwrap();
                        }
                        stdout.queue(cursor::MoveToNextLine(1)).unwrap();
                    } else {
                        //TODO: only increment one x_todo depending on which status
                        x_todo += 1;
                    }
                }
            }
        }

        //move back to real position
        stdout
            .queue(cursor::MoveTo(pos.row, pos.col))
            .expect("error while moving cursor back to current position");

        stdout.flush()?;

        //this waits until event happens
        match event::read()? {
            event::Event::Key(event) => match event.code {
                event::KeyCode::Char('q') => exit = false,
                event::KeyCode::Char('j') => {
                    //TODO: call function that does the navigation
                    pos.one_down((x_todo + 1) as u16);
                }
                event::KeyCode::Char('k') => {
                    //TODO: call function that does the navigation
                    pos.one_up();
                }
                event::KeyCode::Char('h') => {
                    //TODO: call function that does the navigation to the right -> pos.status =
                    //Status::Open
                    ()
                }
                event::KeyCode::Char('l') => {
                    //TODO: call function that does the navigation to the left -> pos.status =
                    //Status::Done
                    ()
                }
                event::KeyCode::Char('a') => {
                    //TODO: call function that lets one insert new todo at current position
                    ()
                }
                _ => {}
            },
            _ => {} // Event::Resize(width, height) => println!("New size {}x{}", width, height),
        }
        stdout
            .queue(cursor::MoveToRow(pos.row))
            .expect("error moving to new line after navigation");

        // Add a small delay to reduce CPU usage and prevent flickering
        sleep(Duration::from_millis(50));
    }

    //clean up stuff
    {
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;
        stdout.queue(cursor::MoveTo(0, 0))?;
        stdout.flush()?;
        stdout.queue(cursor::Show)?;
        let _ = disable_raw_mode();
    }
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
