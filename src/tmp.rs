pub fn main_tui(path: PathBuf) -> io::Result<()> {
    // TODO: perhaps use alternate screen
    let mut stdout = io::stdout();

    // I don't think I need this
    let _ = enable_raw_mode();
    let _ = EnterAlternateScreen;
    stdout.queue(cursor::Hide)?;

    let mut file = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(&path)
        .expect("error while trying to set options for opening file or opening file itself");

    let mut exit = true;

    let mut contents: String = String::new();
    let _ = file.read_to_string(&mut contents);

    // Split the content by lines and create a new vector to store the tuples
    let mut todo_list: Vec<(&str, String)> = contents
        .lines()
        .filter_map(|line| {
            line.split_once('\t')
                .map(|(key, value)| (key, value.to_string())) // Convert the second part to String
        })
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

    stdout.queue(terminal::Clear(terminal::ClearType::All))?;

    while exit {
        //stdout.queue(terminal::Clear(terminal::ClearType::All))?;

        //HACK: keep an eye on the following -> idk

        // Clear the specific lines that will be updated
        let screen_height = todo_list.len() + 1; // Adjust based on your content height
        for i in 0..screen_height {
            stdout.queue(cursor::MoveTo(0, i as u16))?;
            stdout.queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        }

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

        for (i, &(status, ref task)) in todo_list.iter().enumerate() {
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
                    pos.one_down((x_visible + 1) as u16);
                }
                event::KeyCode::Char('k') => {
                    pos.one_up();
                }
                event::KeyCode::Char('r') => {
                    //TODO: rename todo -> should actually be rather similar to swapping status as
                    //we are just chaning the task instead of the status
                    pos.modifier = Modification::Rename;
                    todo_list = modification(&mut pos, todo_list.clone());
                    ()
                }
                event::KeyCode::Char('d') => {
                    pos.modifier = Modification::Delete;
                    todo_list = modification(&mut pos, todo_list.clone());
                    pos.mod_row = -1;
                    pos.modifier = Modification::Default;
                    ()
                }
                event::KeyCode::Char('g') => {
                    pos.go_top();
                    ()
                }
                event::KeyCode::Char('G') => {
                    pos.go_bottom(x_visible);
                    ()
                }
                event::KeyCode::Enter => {
                    pos.modifier = Modification::SwitchStatus;
                    todo_list = modification(&mut pos, todo_list.clone());
                    pos.mod_row = -1;
                    pos.modifier = Modification::Default;
                    ()
                }
                event::KeyCode::Tab => {
                    //HACK: maybe we need to go the first line when switching due to it being more
                    //easy
                    pos.switch_status((todo_list.len() - (x_visible - 2) as usize) as u16);
                    ()
                }
                event::KeyCode::Char('h') => {
                    pos.switch_status((todo_list.len() - (x_visible - 2) as usize) as u16);
                    ()
                }
                event::KeyCode::Char('l') => {
                    pos.switch_status((todo_list.len() - (x_visible - 2) as usize) as u16);
                    ()
                }
                event::KeyCode::Char('a') => {
                    //TODO: call function that lets one insert new todo (at current position or
                    //just append for simplicity)
                    pos.modifier = Modification::New;
                    todo_list = modification(&mut pos, todo_list.clone());
                    pos.mod_row = -1;
                    pos.modifier = Modification::Default;
                    ()
                }
                _ => {}
            },
            _ => {} // Event::Resize(width, height) => println!("New size {}x{}", width, height),
        };

        stdout
            .queue(cursor::MoveToRow(pos.row))
            .expect("error moving to new line after navigation");

        //30 fps
        sleep(Duration::from_millis(33));
    }

    // Clean up stuff
    {
        stdout.queue(terminal::Clear(terminal::ClearType::All))?;
        stdout.queue(cursor::MoveTo(0, 0))?;
        stdout.flush()?;
        stdout.queue(cursor::Show)?;
        let _ = disable_raw_mode();
        let _ = LeaveAlternateScreen;
    }
    //writing to file
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)
            .expect("error while trying to set options for opening file or opening file itself");
        for todo in todo_list {
            let status = todo.0;
            let task = todo.1;
            let line_to_write = format!("{status}\t{task}");
            writeln!(file, "{}", line_to_write).expect("idk");
        }
    }

    Ok(())
}

