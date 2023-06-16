use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{extract::Path, routing::Router, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let repository = TaskRepository::new();

    let app = Router::new()
        .route("/", get(get_tasks).post(create_task))
        .route("/:id", get(get_task).patch(patch_task))
        .with_state(repository);

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_tasks(State(repository): State<TaskRepository>) -> Json<Vec<Task>> {
    Json(repository.get_all().await)
}

#[axum::debug_handler]
async fn create_task(
    State(repository): State<TaskRepository>,
    Json(task): Json<CreateTask>,
) -> Json<Task> {
    let task = Task {
        id: Uuid::new_v4(),
        name: task.name,
        done: false,
    };
    repository.save(&task).await;
    Json(task)
}

type JsonTaskResult = Result<Json<Task>, ApiError>;

async fn get_task(
    State(repository): State<TaskRepository>,
    Path(id): Path<Uuid>,
) -> JsonTaskResult {
    Ok(Json(repository.get_by_id(&id).await?))
}

async fn patch_task(
    State(repository): State<TaskRepository>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    repository.update(&id).await?;
    Ok(StatusCode::OK)
}

enum ApiError {
    TaskIdNotFound,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            ApiError::TaskIdNotFound => (StatusCode::NOT_FOUND, "Task not found"),
        };

        (status, msg).into_response()
    }
}

#[derive(Default, Clone)]
struct TaskRepository {
    tasks: Arc<Mutex<Vec<Task>>>,
}

impl TaskRepository {
    fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn save(&self, task: &Task) {
        self.tasks.lock().unwrap().push(task.clone())
    }

    pub async fn get_all(&self) -> Vec<Task> {
        self.tasks.lock().unwrap().clone()
    }

    pub async fn get_by_id(&self, id: &Uuid) -> Result<Task, ApiError> {
        self.tasks
            .lock()
            .unwrap()
            .iter()
            .find(|t| t.id == *id)
            .cloned()
            .ok_or_else(|| ApiError::TaskIdNotFound)
    }

    pub async fn update(&self, id: &Uuid) -> Result<(), ApiError> {
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
            .ok_or(ApiError::TaskIdNotFound)
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[derive(Deserialize)]
struct CreateTask {
    name: String,
}

#[derive(Serialize, Debug, Clone)]
struct Task {
    id: Uuid,
    name: String,
    done: bool,
}

#[derive(Serialize, Debug)]
struct TaskList {
    tasks: Vec<Task>,
}
