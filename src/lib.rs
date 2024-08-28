use crossterm::{
    cursor::{self, SavePosition},
    event,
    style::{self, style, Attribute, Color, Print, PrintStyledContent, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode, size, window_size, WindowSize},
    ExecutableCommand, QueueableCommand,
};
use std::{env, io::Read, os::linux::raw::stat};
use std::{error::Error, io::stdout};
use std::{
    fmt::format,
    io::{self, Write},
};
use std::{fs, os::unix::process};
use std::{io::Stdout, time::Duration};
use std::{path::PathBuf, u16};
use std::{thread::sleep, usize};

#[derive(Debug, PartialEq)]
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

//struct Win {
//    cols: u16,
//    rows: u16,
//}

#[derive(PartialEq)]
enum Modification {
    Default,
    Delete,
    Rename,
    SwitchStatus,
}

struct Pos {
    row: u16,
    col: u16,
    status: Status,
    mod_row: i8,
    modifier: Modification,
}

impl Pos {
    //fn set_pos(&mut self, col: u16, row: u16) -> &mut Self {
    //    self.col = col;
    //    self.row = row;
    //    self
    //}
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
    //TODO: switch this function for a trait implementation of Not for Status and then just to
    //pos.status = !pos.status;
    //FIX: does not yet work as intended as it sometimes jumps to the first line even though it
    //could stay at the line -> has to do with the row being two bigger than the length i think
    fn switch_status(&mut self, len: u16) -> &mut Self {
        match self.status {
            Status::Done => {
                self.status = Status::Open;
                self.row = std::cmp::min(len, self.row);
            }
            Status::Open => {
                self.status = Status::Done;
                self.row = std::cmp::min(len, self.row);
            }
        }
        self
    }
}

fn main_tui(path: PathBuf) -> io::Result<()> {
    // TODO: perhaps use alternate screen
    let mut stdout = io::stdout();

    // I don't think I need this
    let _ = enable_raw_mode();
    stdout.queue(cursor::Hide)?;

    let mut file = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(path)
        .expect("error while trying to set options for opening file or opening file itself");

    let mut exit = true;

    let mut contents: String = String::new();
    let _ = file.read_to_string(&mut contents);

    // Split the content by lines and create a new vector to store the tuples
    let mut todo_list: Vec<(&str, &str)> = contents
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .collect();

    let mut pos = Pos {
        row: 2,
        col: 1,
        status: Status::Open,
        mod_row: -1,
        modifier: Modification::Default,
    };

    let base_open_style = "Open".with(Color::White);
    let base_done_style = "Done".with(Color::White);

    while exit {
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;

        stdout.queue(cursor::MoveTo(0, 0)).unwrap();

        // Re-initialize the styles to their base state at the start of each iteration
        let mut done_styled = base_done_style;
        let mut open_styled = base_open_style;

        // Apply status-specific styles
        match pos.status {
            Status::Done => {
                done_styled = done_styled.attribute(Attribute::Bold);
                open_styled = open_styled.attribute(Attribute::Dim);
            }
            Status::Open => {
                done_styled = done_styled.attribute(Attribute::Dim);
                open_styled = open_styled.attribute(Attribute::Bold);
            }
        }

        stdout.queue(PrintStyledContent(open_styled)).unwrap();
        stdout.queue(Print("\t")).unwrap();
        stdout.queue(PrintStyledContent(done_styled)).unwrap();

        stdout.queue(cursor::MoveTo(1, 1)).unwrap();
        stdout.queue(cursor::MoveToNextLine(1)).unwrap();

        let mut x_visible = 0;

        for (i, &(status, task)) in todo_list.iter().enumerate() {
            let matches_status = (pos.status == Status::Open && status == "[ ]")
                || (pos.status == Status::Done && status == "[X]");

            if matches_status {
                x_visible += 1;
                let task_to_print = format!("{}\t{}", status, task);

                let style_task = if pos.row
                    == cursor::position()
                        .expect("error while trying to get cursor position")
                        .1
                {
                    pos.mod_row = i as i8;
                    style(&task_to_print).attribute(Attribute::Bold)
                } else {
                    style(&task_to_print).attribute(Attribute::Dim)
                };

                stdout
                    .queue(PrintStyledContent(style_task))
                    .expect("failed to print styled line");
                stdout.queue(cursor::MoveToNextLine(1)).unwrap();
            }
        }

        stdout
            .queue(cursor::MoveTo(pos.row, pos.col))
            .expect("error while moving cursor back to current position");

        stdout.flush()?;

        //this waits until event happes
        match event::read()? {
            event::Event::Key(event) => match event.code {
                event::KeyCode::Char('q') => exit = false,
                event::KeyCode::Char('j') => {
                    //TODO: call function that does the navigation
                    pos.one_down((x_visible + 1) as u16);
                }
                event::KeyCode::Char('k') => {
                    //TODO: call function that does the navigation
                    pos.one_up();
                }
                event::KeyCode::Tab => {
                    //TODO: call function that does the navigation to the right -> pos.status =
                    //Status::Open
                    //HACK: maybe we need to go the first line when switching due to it being more
                    //easy
                    pos.switch_status((todo_list.len() - x_visible + 1) as u16);
                    ()
                }
                event::KeyCode::Char('r') => {
                    //TODO: should move currently highlighted todo to other side
                    pos.mod_row = pos.row as i8;
                    pos.modifier = Modification::Rename;
                    todo_list = modification(&mut pos, todo_list.clone());
                    ()
                }
                event::KeyCode::Char('d') => {
                    //TODO: should move currently highlighted todo to other side
                    pos.modifier = Modification::Delete;
                    todo_list = modification(&mut pos, todo_list.clone());
                    ()
                }
                event::KeyCode::Enter => {
                    //TODO: should move currently highlighted todo to other side
                    // pos.mod_row = pos.row as i8;
                    pos.modifier = Modification::SwitchStatus;
                    todo_list = modification(&mut pos, todo_list.clone());
                    ()
                }
                event::KeyCode::Char('h') => {
                    //TODO: call function that does the navigation to the left -> pos.status =
                    //Status::Done
                    pos.switch_status((todo_list.len() - x_visible + 1) as u16);
                    ()
                }
                event::KeyCode::Char('l') => {
                    //TODO: call function that does the navigation to the left -> pos.status =
                    //Status::Done
                    pos.switch_status((todo_list.len() - x_visible + 1) as u16);
                    ()
                }
                event::KeyCode::Char('a') => {
                    //TODO: call function that lets one insert new todo at current position
                    ()
                }
                _ => {}
            },
            _ => {} // Event::Resize(width, height) => println!("New size {}x{}", width, height),
        };
        stdout
            .queue(cursor::MoveToRow(pos.row))
            .expect("error moving to new line after navigation");

        sleep(Duration::from_millis(50));
    }

    // Clean up stuff
    {
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;
        stdout.queue(cursor::MoveTo(0, 0))?;
        stdout.flush()?;
        stdout.queue(cursor::Show)?;
        let _ = disable_raw_mode();
    }

    Ok(())
}

fn modification<'a>(
    pos: &mut Pos,
    mut todo_list: Vec<(&'a str, &'a str)>,
) -> Vec<(&'a str, &'a str)> {
    match pos.modifier {
        Modification::Delete => {
            if pos.mod_row >= 0
                && (pos.mod_row as usize) < todo_list.len()
                && pos.status == Status::Done
            {
                todo_list.remove(pos.mod_row as usize);
                pos.mod_row = -1;
                pos.modifier = Modification::Default;
            }
        }
        Modification::SwitchStatus => {
            //if let Some(todo) = todo_list.get_mut(pos.mod_row as usize) {
            //    let (status, task) = *todo;
            //    let new_status = match status {
            //        "[ ]" => "[X]",
            //        "[X]" => "[ ]",
            //        _ => status,
            //    };
            //    //*todo = (new_status, task);
            //    todo_list.push((new_status, task));
            //}

            let new_status = match todo_list[pos.mod_row as usize].0 {
                "[ ]" => "[X]",
                "[X]" => "[ ]",
                _ => "[ ]",
            };
            todo_list.push((new_status, todo_list[pos.mod_row as usize].1));
            todo_list.remove(pos.mod_row as usize);
            pos.mod_row = -1;
            pos.modifier = Modification::Default;
        }
        _ => {}
    }
    todo_list
}

//fn modification<'a>(pos: &mut Pos, todo_list: &'a mut Vec<&'a str>) -> Vec<&'a str> {
//    match pos.modifier {
//        Modification::Delete => {
//            //TODO: perhaps only let one remove a todo if it is status == done?
//            //FIX: doesnt work as intended
//            //might actually be harder than i thought. The problem is that we get different rows
//            //for done and open
//            //-> perhaps use differnt files for open and done
//            if pos.status == Status::Done && pos.mod_row != -1 as i8 {
//                todo_list.remove((pos.mod_row - 2) as usize);
//                pos.mod_row = -1;
//                pos.modifier = Modification::Default;
//            }
//        }
//        //TODO: write other modifications
//        _ => (),
//    }
//    //creates new vec to return
//    todo_list.to_vec()
//}

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
