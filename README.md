
# Rust t(o)d(o) Application

<!--![Build Status](https://img.shields.io/github/workflow/status/yourusername/yourprojectname/CI)-->
<!-- ![License](https://img.shields.io/github/license/yourusername/yourprojectname)-->
<!--![Version](https://img.shields.io/github/v/release/yourusername/yourprojectname)-->

A simple, fast, and efficient TUI t(o)d(o) application written in Rust. This application allows you to manage your tasks directly from the terminal with ease.

## Features

- Add new tasks with a description and due date.
- List all pending tasks.
- Mark tasks as completed.
- Remove completed or outdated tasks.
- Prioritize tasks.
- Save tasks persistently using local storage.

## Installation

### Prerequisites

- Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed on your machine.

### Clone the repository

```sh
git clone https://github.com/yourusername/yourprojectname.git
cd yourprojectname
```

### Build the application

```sh
cargo build --release
```

### Run the application

```sh
./target/release/td
```

## Usage

Here are some common commands you can use with this ToDo application:

### Add a new task

```sh
td "new task"
```
or 
```sh
td new task
```
### Enter TUI

```sh
td
```

### For more options and usage, run:

```sh
todo --help
```

## License

[LICENSE](LICENSE.md)

## Acknowledgements

- Thanks to the [Rust](https://www.rust-lang.org/) community for their amazing work on the language and ecosystem.
