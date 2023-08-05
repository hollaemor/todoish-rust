use std::error::Error;
use std::sync::Arc;

use todo_ish::api::rest::server::create_server;
use todo_ish::tasks::repository::PostgresTaskRepository;

use todo_ish::config;

use sqlx;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_tracer();
    let app_config = config::create_config().unwrap();
    let pool = sqlx::postgres::PgPool::connect(&app_config.database_url).await?;
    let repository = Arc::new(PostgresTaskRepository::new(pool));
    create_server(repository, &app_config).await.unwrap();
    Ok(())
}

fn init_tracer() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting subscriber failed");
}
