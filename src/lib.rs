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
