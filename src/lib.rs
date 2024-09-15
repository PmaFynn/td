use crossterm::event::{Event, KeyCode};
use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Color,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
    Terminal,
};

use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use std::usize;
use std::{env, io::Read};

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
        let todo_instance = Task {
            task: concated_task,
            status: Status::Open,
        };
        write_todo(path, todo_instance);
        return Ok(());
    }
}

#[derive(PartialEq)]
enum Modification {
    Default,
    Delete,
    Rename,
    SwitchStatus,
    New,
}
struct InputState {
    input: String,
    cursor_position: usize,
    canceled: bool,
    submitted: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            canceled: false,
            submitted: false,
        }
    }
    pub fn reset() -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            canceled: false,
            submitted: false,
        }
    }
    pub fn handle_input(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.input.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.input.len() {
                    self.cursor_position += 1;
                }
            }

            KeyCode::Enter => self.submitted = true,
            KeyCode::Esc => self.canceled = true,
            _ => {}
        }
    }
}

struct App {
    status: Status,
    mod_item: i8,
    modifier: Modification,
    input_state: InputState,
    show_modal: bool,
}

impl App {
    //TODO: switch this function for a trait implementation of Not for Status and then just to
    //pos.status = !pos.status;
    //FIX: does not yet work as intended as it sometimes jumps to the first line even though it
    //could stay at the line -> has to do with the row being two bigger than the length i think
    pub fn new() -> Self {
        Self {
            status: Status::Open,
            mod_item: 0,
            modifier: Modification::Default,
            input_state: InputState::new(),
            show_modal: false,
        }
    }

    //FIX: acutally sometimes when swapping it is not highlighted at all
    fn switch_status(&mut self) -> &mut Self {
        match self.status {
            Status::Done => {
                self.status = Status::Open;
            }
            Status::Open => {
                self.status = Status::Done;
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

    let mut app_state = App::new();

    let mut last_event_time = Instant::now();
    let debounce_duration = Duration::from_millis(1); // Adjust the debounce duration to suit your need

    let mut visible_list_length = 0;

    // Add list state for managing scrolling
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(0)); // Start at the top of the list
                                //
                                // Application state
    let mut search_for: (String, i32) = (String::from(""), 0);

    let mut last_relevant_key: char = '/';

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
                    if app_state.status == Status::Open {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        status_style
                    },
                ),
                Span::raw("    ".to_string()),
                Span::styled(
                    "Done".to_string(),
                    if app_state.status == Status::Done {
                        Style::default().add_modifier(Modifier::BOLD)
                    } else {
                        status_style
                    },
                ),
            ]);
            let status_paragraph =
                Paragraph::new(status).block(Block::default().borders(Borders::ALL));
            f.render_widget(status_paragraph, chunks[1]);

            visible_list_length = 0;
            let mut searched_n_item = 0;
            let items: Vec<ListItem> = todo_list
                .iter()
                .enumerate()
                .filter_map(|(i, (status, task))| {
                    // Determine if the current item is included based on its status
                    let included = match app_state.status {
                        Status::Open => *status == "[ ]", // Only include items with status "[ ]" when Open
                        Status::Done => *status == "[X]", // Only include items with status "[X]" when Done
                    };

                    if included {
                        if search_for.0 != ""
                            && task.contains(&search_for.0)
                            && searched_n_item == search_for.1
                        {
                            list_state.select(Some(visible_list_length));
                            searched_n_item += 1;
                            //search_for = (String::from(""), search_for.1);
                        } else if search_for.0 != ""
                            && task.contains(&search_for.0)
                            && searched_n_item < search_for.1
                        {
                            searched_n_item += 1;
                        }

                        // Only create the list item if the current item is included
                        if let Some(selected) = list_state.selected() {
                            if visible_list_length == selected {
                                app_state.mod_item = i as i8;
                            };
                        }

                        let task_spans = Line::from(vec![
                            Span::styled(
                                status.to_string(),
                                Style::default().fg(ratatui::prelude::Color::Yellow),
                            ),
                            Span::raw("    ".to_string()),
                            Span::raw(task),
                        ]);
                        // Increment x_visible for items being filtered out
                        visible_list_length += 1;

                        // Return the new ListItem
                        Some(ListItem::new(task_spans))
                    } else {
                        None // Exclude this item from the final list
                    }
                })
                .collect();

            search_for.1 = 0;

            let todos = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("TODOs"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));

            visible_list_length = todos.len();

            f.render_stateful_widget(todos, chunks[0], &mut list_state);

            if app_state.show_modal {
                render_modal(f, &app_state);
            }
        })?;

        // Poll for an event with a timeout to avoid blocking
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Check if enough time has passed to handle the next event
                // also check if windows really needs this or just the polling
                if last_event_time.elapsed() >= debounce_duration {
                    last_event_time = Instant::now(); // Reset event timer

                    if !app_state.show_modal {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => break,

                            // Navigation
                            KeyCode::Char('j') => {
                                if let Some(selected) = list_state.selected() {
                                    let new_index = (selected + 1).min(todo_list.len() - 1);
                                    list_state.select(Some(new_index));
                                }
                            }
                            KeyCode::Char('k') => {
                                if let Some(selected) = list_state.selected() {
                                    let new_index = selected.saturating_sub(1);
                                    list_state.select(Some(new_index));
                                }
                            }
                            KeyCode::Char('g') => {
                                //pos.go_top();
                                list_state.select(Some(0));
                            }

                            KeyCode::Char('G') => {
                                list_state.select(Some(todo_list.len() - 1));
                            }

                            KeyCode::Char('/') => {
                                search_for = (String::from("test"), 0);
                                last_relevant_key = '/';
                                //TODO: Search
                            }
                            KeyCode::Char('n') => {
                                if last_relevant_key == '/' || last_relevant_key == 'n' {
                                    search_for.1 += 1;
                                    println!("test");
                                }
                                last_relevant_key = 'n';
                            }

                            // Switch status (Open/Done)
                            KeyCode::Char('h') | KeyCode::Char('l') | KeyCode::Tab => {
                                app_state.switch_status();
                            }

                            // Switch task status (Open/Done) when pressing Enter
                            KeyCode::Enter => {
                                app_state.modifier = Modification::SwitchStatus;
                                todo_list =
                                    modification(&mut app_state, String::new(), todo_list.clone());
                                app_state.modifier = Modification::Default;
                            }

                            // Adding a new task
                            KeyCode::Char('a') => {
                                //HACK: idk -> maybe i want to be able input todos after reading a
                                //done todo
                                //
                                //if app_state.status == Status::Open {
                                //    app_state.modifier = Modification::New;
                                //    app_state.show_modal = true;
                                //}
                                app_state.modifier = Modification::New;
                                app_state.show_modal = true;
                                //todo_list = modification(&mut app_state, todo_list.clone());
                                //app_state.modifier = Modification::Default;
                            }

                            // Renaming a task
                            KeyCode::Char('r') => {
                                app_state.modifier = Modification::Rename;
                                app_state.show_modal = true;
                            }

                            // Deleting a task
                            KeyCode::Char('d') => {
                                app_state.modifier = Modification::Delete;
                                todo_list =
                                    modification(&mut app_state, String::new(), todo_list.clone());
                                app_state.modifier = Modification::Default;
                            }

                            _ => {
                                app_state.show_modal = !app_state.show_modal;
                            }
                        }
                    } else {
                        match key.code {
                            _ => {
                                app_state.input_state.handle_input(key);
                                //match app_state.modifier {
                                //    Modification::Rename => {}
                                //    Modification::New => {}
                                //    _ => {
                                //        //TODO: display help here
                                //    }
                                //}
                                if app_state.input_state.submitted {
                                    // Save the input
                                    let new_todo = app_state.input_state.input.clone();
                                    todo_list = modification(&mut app_state, new_todo, todo_list);
                                    //todo_list.push(("[ ]", new_todo)); // Assuming you have a Vec<String> for todos
                                    app_state.show_modal = false;
                                    app_state.input_state = InputState::new();
                                    app_state.modifier = Modification::Default;
                                } else if app_state.input_state.canceled {
                                    // Cancel input
                                    app_state.show_modal = false;
                                    app_state.input_state = InputState::new();
                                    app_state.modifier = Modification::Default;
                                }
                            }
                        }
                    }
                }
            }
        }

        std::thread::sleep(Duration::from_millis(33));
    }

    // Writing changes back to file
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
    app_state: &mut App,
    todo_item_from_input: String,
    mut todo_list: Vec<(&'a str, String)>,
) -> Vec<(&'a str, String)> {
    if app_state.mod_item as usize >= todo_list.len() {
        return todo_list;
    }

    match app_state.modifier {
        //TODO: if the item is the last item it pos.item should go one up
        Modification::Delete => {
            if app_state.status == Status::Done {
                todo_list.remove(app_state.mod_item as usize);
            }
        }
        Modification::SwitchStatus => {
            let new_status = match todo_list[app_state.mod_item as usize].0 {
                "[ ]" => "[X]",
                "[X]" => "[ ]",
                _ => "[ ]",
            };
            let task = todo_list[app_state.mod_item as usize].1.clone();
            todo_list[app_state.mod_item as usize] = (new_status, task);
        }
        Modification::Rename => {
            //TODO: Implement a function to get a new task name, for now, it's unchanged
            todo_list[app_state.mod_item as usize].1 = todo_item_from_input;
        }
        Modification::New => {
            //TODO: Implement a function to get a new task name, for now, it's unchanged
            todo_list.push(("[ ]", todo_item_from_input));
        }
        _ => {}
    }
    todo_list
}

// Separate function to render the modal
fn render_modal(f: &mut ratatui::Frame, app_state: &App) {
    // Define the rows and columns for the table
    let rows = vec![
        Row::new(vec![Cell::from("quit application"), Cell::from("q")]),
        Row::new(vec![Cell::from("exit out of modal"), Cell::from("Esc")]),
        Row::new(vec![Cell::from("down"), Cell::from("j")]),
        Row::new(vec![Cell::from("up"), Cell::from("k")]),
        Row::new(vec![Cell::from("switch status"), Cell::from("h, l, tab")]),
        Row::new(vec![Cell::from("goTop"), Cell::from("g")]),
        Row::new(vec![Cell::from("delete done todo"), Cell::from("d")]),
        Row::new(vec![Cell::from("add new todo"), Cell::from("a")]),
        Row::new(vec![Cell::from("rename selected todo"), Cell::from("n")]),
        Row::new(vec![Cell::from("goBottom"), Cell::from("G")]),
        Row::new(vec![
            Cell::from("switch status of selected todo"),
            Cell::from("enter"),
        ]),
    ];
    // Create a centered Rect for the modal
    let modal_width = (f.area().width * 60) / 100; // 60% of the terminal width
    let modal_height = match app_state.modifier {
        Modification::Rename | Modification::New => 3,
        //_ => (f.area().height * 30) / 100,
        _ => rows.len() as u16 + 2, //length of help_rows + 2 for border
    };
    let modal_layout = Rect {
        x: (f.area().width - modal_width) / 2,
        y: (f.area().height - modal_height) / 2,
        width: modal_width,
        height: modal_height,
    };

    // Clear the area where the modal will be rendered
    f.render_widget(Clear, modal_layout);

    match app_state.modifier {
        Modification::Rename => {
            let input = Paragraph::new(app_state.input_state.input.as_str())
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Color::Blue)
                        .title("Rename selected todo"),
                );
            f.render_widget(input, modal_layout);
        }
        Modification::New => {
            let input = Paragraph::new(app_state.input_state.input.as_str())
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Color::Blue)
                        .title("Create new todo"),
                );
            f.render_widget(input, modal_layout);
        }
        _ => {
            // Create a paragraph with static text for the modal
            let modal_block = Block::default()
                .title("Keybinds")
                .borders(Borders::ALL)
                .border_style(Color::Blue)
                .style(Style::default().fg(Color::White));

            // Create the table with width constraints
            let table = Table::new(
                rows,
                [Constraint::Percentage(90), Constraint::Percentage(10)],
            )
            .block(modal_block)
            .column_spacing(2) // Add space between columns
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
            f.render_widget(table, modal_layout);
        }
    }
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
    path.push("mega/.todo_app");
    fs::create_dir_all(&path).expect("Failed to create directory");
    path.push("todos.txt");
    path
}
