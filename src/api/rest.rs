pub mod server {
    use std::sync::Arc;

    use crate::config::AppConfig;
    use crate::tasks::domain::TaskError;
    use crate::tasks::repository::TaskRepository;
    use axum::response::IntoResponse;
    use axum::Json;
    use axum::Router;
    use hyper::StatusCode;
    use serde_json::json;
    use std::error::Error;
    use tokio;
    use tracing;

    use super::endpoints::tasks::get_routes;

    #[derive(Clone)]
    pub struct AppState {
        pub repository: Arc<dyn TaskRepository + Send + Sync>,
    }

    impl IntoResponse for TaskError {
        fn into_response(self) -> axum::response::Response {
            let (status, msg) = match self {
                TaskError::IdNotFound => (StatusCode::NOT_FOUND, "Task not found"),
                TaskError::CreationError => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Could not create task")
                }
                TaskError::DbError => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "System failure. Please try again later",
                ),
                TaskError::GetTaskError => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Could not get tasks")
                }
            };

            (status, Json(json!({ "message": msg }))).into_response()
        }
    }

    pub fn create_app(arc: Arc<dyn TaskRepository + Send + Sync>) -> Router {
        let app_state = AppState { repository: arc };
        let task_routes = get_routes(app_state.clone());

        let app = Router::new().nest("/v1/tasks", task_routes);
        app
    }

    pub async fn create_server(
        arc: Arc<dyn TaskRepository + Send + Sync>,
        config: &AppConfig,
    ) -> Result<(), Box<dyn Error>> {
        let listener =
            tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.server_port)).await?;
        tracing::debug!(
            "HTTP Server listening on: {}",
            listener.local_addr().unwrap()
        );

        axum::Server::from_tcp(listener.into_std()?)?
            .serve(create_app(arc).into_make_service())
            .await?;
        Ok(())
    }
}

pub mod endpoints {

    pub mod tasks {

        use crate::api::rest::server::AppState;
        use crate::tasks::domain::{Task, TaskError};
        use axum;
        use axum::extract::State;
        use axum::routing::get;
        use axum::{extract::Path, routing::Router, Json};
        use hyper::StatusCode;
        use serde::{Deserialize, Serialize};
        use uuid::Uuid;

        type GetAllTaskResult = Result<Json<Vec<TaskResponse>>, TaskError>;
        type JsonTaskResult = Result<Json<TaskResponse>, TaskError>;

        async fn get_tasks(State(app): State<AppState>) -> GetAllTaskResult {
            let tasks = app.repository.get_all().await?;

            let response = tasks.iter().map(|t| TaskResponse::from_task(t)).collect();

            Ok(Json(response))
        }
        async fn create_task(
            State(app): State<AppState>,
            Json(task): Json<CreateTask>,
        ) -> Result<(StatusCode, Json<TaskResponse>), TaskError> {
            let task = Task {
                id: Uuid::new_v4(),
                name: task.name,
                done: false,
            };
            app.repository.save(&task).await?;
            Ok((StatusCode::CREATED, Json(TaskResponse::from_task(&task))))
        }

        async fn get_task(State(app): State<AppState>, Path(id): Path<Uuid>) -> JsonTaskResult {
            let task = app.repository.get_by_id(&id).await?;
            Ok(Json(TaskResponse::from_task(&task)))
        }

        pub async fn patch_task(
            State(app): State<AppState>,
            Path(id): Path<Uuid>,
        ) -> Result<StatusCode, TaskError> {
            app.repository.update(&id).await?;
            Ok(StatusCode::OK)
        }

        pub fn get_routes(state: AppState) -> Router {
            Router::new()
                .route("/", get(get_tasks).post(create_task))
                .route("/:id", get(get_task).patch(patch_task))
                .with_state(state)
        }

        #[derive(Deserialize)]
        struct CreateTask {
            name: String,
        }

        #[derive(Serialize)]
        struct TaskResponse {
            name: String,
            id: Uuid,
            done: bool,
        }

        impl TaskResponse {
            fn from_task(task: &Task) -> Self {
                Self {
                    done: task.done,
                    name: task.name.clone(),
                    id: task.id,
                }
            }
        }
    }
}
