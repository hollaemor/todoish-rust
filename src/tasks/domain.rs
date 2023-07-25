use uuid::Uuid;

#[derive(Clone)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub done: bool,
}

#[derive(Debug)]
pub enum TaskError {
    IdNotFound,
    CreationError,
    GetTaskError,
    DbError,
}
