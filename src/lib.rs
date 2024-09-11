use crossterm::event::{Event, KeyCode};
use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::time::Duration;
use std::time::Instant;
use std::usize;
use std::{env, io::Read};
use std::{path::PathBuf, u16};

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
        //this should skip the first entry of args which is always some random value
        let mut concated_task = String::new();
        for (i, arg) in args.into_iter().enumerate().skip(1) {
            concated_task.push_str(arg);
            if i != args.len() {
                concated_task.push(' ')
            }
        }
        //TODO: if first arg == "help" -> print out guide or something
        let todo_instance = Task {
            task: concated_task,
            status: Status::Open,
        };
        write_todo(path, todo_instance);
        return Ok(());
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
    New,
}

struct Pos {
    cur_item: u16,
    status: Status,
    mod_item: i8,
    modifier: Modification,
}

impl Pos {
    fn go_bottom(&mut self, bottom: u16) -> &mut Self {
        self.cur_item = bottom;
        self
    }
    fn go_top(&mut self) -> &mut Self {
        self.cur_item = 0;
        self
    }
    fn one_down(&mut self, max_row: u16) -> &mut Self {
        if self.cur_item < max_row - 1 {
            self.cur_item += 1;
        }
        self
    }
    fn one_up(&mut self) -> &mut Self {
        if self.cur_item > 0 {
            self.cur_item -= 1;
        }
        self
    }
    //TODO: switch this function for a trait implementation of Not for Status and then just to
    //pos.status = !pos.status;
    //FIX: does not yet work as intended as it sometimes jumps to the first line even though it
    //could stay at the line -> has to do with the row being two bigger than the length i think

    //FIX: acutally sometimes when swapping it is not highlighted at all
    fn switch_status(&mut self, len: u16) -> &mut Self {
        match self.status {
            Status::Done => {
                self.status = Status::Open;
                self.cur_item = std::cmp::min(len, self.cur_item);
            }
            Status::Open => {
                self.status = Status::Done;
                self.cur_item = std::cmp::min(len, self.cur_item);
            }
        }
        self
    }
}

pub fn main_tui(path: PathBuf) -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut file = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(&path)
        .expect("error while trying to set options for opening file or opening file itself");

    let mut contents: String = String::new();
    let _ = file.read_to_string(&mut contents);

    let mut todo_list: Vec<(&str, String)> = contents
        .lines()
        .filter_map(|line| {
            line.split_once('\t')
                .map(|(key, value)| (key, value.to_string()))
        })
        .collect();

    let mut pos = Pos {
        cur_item: 0,
        status: Status::Open,
        mod_item: 0,
        modifier: Modification::Default,
    };

    let mut last_event_time = Instant::now();
    let debounce_duration = Duration::from_millis(1); // Adjust the debounce duration to suit your need

    let mut x_visible = 0;

    loop {
        terminal.draw(|f| {
            let size = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(size);

            let status_style = Style::default().fg(ratatui::prelude::Color::White);
            let status = Line::from(vec![
                Span::styled(
                    "Open".to_string(),
                    if pos.status == Status::Open {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        status_style
                    },
                ),
                Span::raw("    ".to_string()),
                Span::styled(
                    "Done".to_string(),
                    if pos.status == Status::Done {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        status_style
                    },
                ),
            ]);
            let status_paragraph =
                Paragraph::new(status).block(Block::default().borders(Borders::ALL));
            f.render_widget(status_paragraph, chunks[1]);

            x_visible = 0;
            let items: Vec<ListItem> = todo_list
                .iter()
                .enumerate()
                .filter_map(|(i, (status, task))| {
                    // Determine if the current item is included based on its status
                    let included = match pos.status {
                        Status::Open => *status == "[ ]", // Only include items with status "[ ]" when Open
                        Status::Done => *status == "[X]", // Only include items with status "[X]" when Done
                    };

                    if included {
                        // Only create the list item if the current item is included
                        let task_style = if x_visible == pos.cur_item as usize {
                            pos.mod_item = i as i8;
                            Style::default().add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };

                        let task_spans = Line::from(vec![
                            Span::styled(
                                status.to_string(),
                                Style::default().fg(ratatui::prelude::Color::Yellow),
                            ),
                            Span::raw("    ".to_string()),
                            Span::styled(task, task_style),
                        ]);
                        // Increment x_visible for items being filtered out
                        x_visible += 1;

                        // Return the new ListItem
                        Some(ListItem::new(task_spans))
                    } else {
                        None // Exclude this item from the final list
                    }
                })
                .collect();
            let todos = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("TODOs"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));

            f.render_stateful_widget(
                todos,
                chunks[0],
                &mut ratatui::widgets::ListState::default(),
            );
        })?;

        // Poll for an event with a timeout to avoid blocking
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Check if enough time has passed to handle the next event
                if last_event_time.elapsed() >= debounce_duration {
                    last_event_time = Instant::now(); // Reset event timer

                    match key.code {
                        KeyCode::Char('q') => break,

                        // Navigation
                        KeyCode::Char('j') => {
                            pos.one_down(x_visible as u16);
                        }
                        KeyCode::Char('k') => {
                            pos.one_up();
                        }
                        KeyCode::Char('g') => {
                            pos.go_top();
                        }
                        KeyCode::Char('G') => {
                            pos.go_bottom(x_visible as u16 - 1);
                        }

                        // Switch status (Open/Done)
                        KeyCode::Char('h') | KeyCode::Char('l') | KeyCode::Tab => {
                            pos.switch_status(todo_list.len() as u16 - x_visible as u16 - 1);
                        }

                        // Switch task status (Open/Done) when pressing Enter
                        KeyCode::Enter => {
                            pos.modifier = Modification::SwitchStatus;
                            todo_list = modification(&mut pos, todo_list.clone());
                            pos.modifier = Modification::Default;
                        }

                        // Adding a new task
                        KeyCode::Char('a') => {
                            pos.modifier = Modification::New;
                            todo_list = modification(&mut pos, todo_list.clone());
                            pos.modifier = Modification::Default;
                        }

                        // Renaming a task
                        KeyCode::Char('r') => {
                            pos.modifier = Modification::Rename;
                            todo_list = modification(&mut pos, todo_list.clone());
                        }

                        // Deleting a task
                        KeyCode::Char('d') => {
                            pos.modifier = Modification::Delete;
                            todo_list = modification(&mut pos, todo_list.clone());
                            pos.modifier = Modification::Default;
                        }

                        _ => {}
                    }
                }
            }
        }

        std::thread::sleep(Duration::from_millis(33));
    }
    // Writing changes back to file
    println!("{}", path.display());
    let mut file = match fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file for writing: {}", e);
            return Err(e); // Exit if there's an error opening the file
        }
    };

    // Write each todo back to the file
    for todo in &todo_list {
        let status = todo.0;
        let task = &todo.1;
        let line_to_write = format!("{status}\t{task}");

        // Attempt to write the line, and handle any errors
        if let Err(e) = writeln!(file, "{}", line_to_write) {
            eprintln!("Error writing to file: {}", e);
            return Err(e); // Exit if there's an error writing to the file
        }
    }

    // Flush the buffer to ensure all data is written to the file
    if let Err(e) = file.flush() {
        eprintln!("Error flushing file: {}", e);
        return Err(e); // Exit if there's an error flushing the file
    }

    // Clean up terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn modification<'a>(
    pos: &mut Pos,
    mut todo_list: Vec<(&'a str, String)>,
) -> Vec<(&'a str, String)> {
    if pos.mod_item as usize >= todo_list.len() {
        return todo_list;
    }

    match pos.modifier {
        //TODO: if the item is the last item it pos.item should go one up
        Modification::Delete => {
            if pos.status == Status::Done {
                todo_list.remove(pos.mod_item as usize);
            }
        }
        Modification::SwitchStatus => {
            let new_status = match todo_list[pos.mod_item as usize].0 {
                "[ ]" => "[X]",
                "[X]" => "[ ]",
                _ => "[ ]",
            };
            let task = todo_list[pos.mod_item as usize].1.clone();
            todo_list[pos.mod_item as usize] = (new_status, task);
        }
        Modification::Rename => {
            //TODO: Implement a function to get a new task name, for now, it's unchanged
            let new_task = new_task();
            todo_list[pos.mod_item as usize].1 = new_task;
        }
        Modification::New => {
            //TODO: Implement a function to get a new task name, for now, it's unchanged
            let new_task = new_task();
            todo_list.push(("[ ]", new_task));
        }
        _ => {}
    }
    todo_list
}

fn new_task() -> String {
    //TODO: Implement that user input is returned -> first really minimal then better looking
    String::from("test2")
}

/// appends new todo to the end of todo file
pub fn write_todo(path: PathBuf, todo: Task) {
    //TODO: maybe just concat all all args if its more than 1 so I dont have to put "" -> like
    //cargo run [executable] this is a new todo
    //this would becomes "[ ]   this is a new todo" in the file
    let file = fs::OpenOptions::new().append(true).create(true).open(path);

    match file {
        Ok(mut file) => {
            let insert = format!("[ ]\t{}", todo.task);
            writeln!(file, "{}", insert).expect("idk");
        }
        Err(i) => println!("Error writing to file: {}", i),
    }
}

pub fn get_todo_file_path() -> PathBuf {
    // Determine home directory based on the operating system
    #[cfg(target_os = "windows")]
    let home_dir = env::var("USERPROFILE").unwrap_or_else(|_| String::from("."));

    #[cfg(not(target_os = "windows"))]
    let home_dir = env::var("HOME").unwrap_or_else(|_| String::from("."));

    // Build the path to the .todo_app directory and todos.txt file
    let mut path = PathBuf::from(home_dir);
    path.push(".todo_app");
    fs::create_dir_all(&path).expect("Failed to create directory");
    path.push("todos.txt");
    path
}
