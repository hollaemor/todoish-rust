use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use sqlx::{self, postgres::PgRow, Row};
use tracing;
use uuid::Uuid;
use crate::tasks::domain::{Task, TaskError};

#[async_trait]
pub trait TaskRepository {
    async fn save(&self, task: &Task) -> Result<(), TaskError>;
    async fn get_all(&self) -> Result<Vec<Task>, TaskError>;
    async fn get_by_id(&self, id: &Uuid) -> Result<Task, TaskError>;
    async fn update(&self, id: &Uuid) -> Result<(), TaskError>;
}

pub struct PostgresTaskRepository {
    pool: sqlx::postgres::PgPool,
}

impl PostgresTaskRepository {
    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self { pool }
    }
}

fn map_to_task_error<E>(e: E, task_error: TaskError) -> TaskError
where
    E: std::error::Error,
{
    tracing::error!("DB error in context {:?}. Error: {}", task_error, e);
    task_error
}

#[async_trait]
impl TaskRepository for PostgresTaskRepository {
    async fn save(&self, task: &Task) -> Result<(), TaskError> {
        sqlx::query("INSERT INTO tasks(id, name, done) VALUES ($1, $2, $3)")
            .bind(&task.id)
            .bind(&task.name)
            .bind(&task.done)
            .execute(&self.pool)
            .await
            .map_err(|e| map_to_task_error(e, TaskError::CreationError))?;
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<Task>, TaskError> {
        sqlx::query("SELECT id, name, done FROM tasks ORDER BY created_at DESC")
            .map(|row: PgRow| Task {
                id: row.get("id"),
                name: row.get("name"),
                done: row.get("done"),
            })
            .fetch_all(&self.pool)
            .await
            .map_err(|e| map_to_task_error(e, TaskError::GetTaskError))
    }

    async fn get_by_id(&self, id: &Uuid) -> Result<Task, TaskError> {
        sqlx::query("SELECT id, name, done FROM tasks WHERE id = $1")
            .bind(id)
            .map(|row: PgRow| Task {
                id: row.get("id"),
                name: row.get("name"),
                done: row.get("done"),
            })
            .fetch_one(&self.pool)
            .await
            .map_err(|e| map_to_task_error(e, TaskError::IdNotFound))
    }

    async fn update(&self, id: &Uuid) -> Result<(), TaskError> {
        let count = sqlx::query("UPDATE tasks SET done = 't' WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| map_to_task_error(e, TaskError::DbError))?;

        if count.rows_affected() == 0 {
            Err(TaskError::IdNotFound)
        } else {
            Ok(())
        }
    }
}


pub struct InMemoryTaskRepository {
    tasks: Arc<Mutex<Vec<Task>>>,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl TaskRepository for InMemoryTaskRepository {
    async fn save(&self, task: &Task) -> Result<(), TaskError> {
        self.tasks.lock().unwrap().push(task.clone());
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<Task>, TaskError> {
        Ok(self.tasks.lock().unwrap().clone())
    }

    async fn get_by_id(&self, id: &Uuid) -> Result<Task, TaskError> {
        self.tasks
            .lock()
            .unwrap()
            .iter()
            .find(|t| t.id == *id)
            .cloned()
            .ok_or_else(|| TaskError::IdNotFound)
    }

    async fn update(&self, id: &Uuid) -> Result<(), TaskError> {
        match self
            .tasks
            .lock()
            .unwrap()
            .iter_mut()
            .find(|t| t.id == *id)
            .map(|t| {
                t.done = true;
                t
            })
            .ok_or(TaskError::IdNotFound)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
