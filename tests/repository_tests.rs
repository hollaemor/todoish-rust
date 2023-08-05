use sqlx::{self, PgPool, Row};
use todo_ish::tasks::domain::Task;
use todo_ish::tasks::repository::PostgresTaskRepository;
use todo_ish::tasks::repository::TaskRepository;
use uuid::Uuid;

#[sqlx::test]
async fn test_task_is_persisted(pool: PgPool) -> sqlx::Result<()> {
    let repo = PostgresTaskRepository::new(pool.clone());

    let task = Task {
        id: Uuid::new_v4(),
        name: "Test task".into(),
        done: false,
    };

    repo.save(&task).await.unwrap();

    let count = sqlx::query("SELECT COUNT(*) FROM tasks WHERE id = $1")
        .bind(&task.id)
        .fetch_one(&pool)
        .await?;

    assert_eq!(1, count.get::<i64, _>(0));

    Ok(())
}

#[sqlx::test]
async fn task_is_updated(pool: PgPool) -> sqlx::Result<()> {
    let repo = PostgresTaskRepository::new(pool.clone());

    let task = Task {
        id: Uuid::new_v4(),
        name: "Feed the cow".into(),
        done: false,
    };

    repo.save(&task).await.unwrap();
    repo.update(&task.id).await.unwrap();

    let row = sqlx::query("SELECT done FROM tasks WHERE id = $1")
        .bind(&task.id)
        .fetch_one(&pool)
        .await?;

    assert!(row.get::<bool, _>("done"));

    Ok(())
}

#[sqlx::test]
async fn task_is_retrieved(pool: PgPool) -> sqlx::Result<()> {
    let repo = PostgresTaskRepository::new(pool.clone());

    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO tasks (id, name, done) VALUES ($1, 'another one', 't')")
        .bind(&id)
        .execute(&pool)
        .await?;

    let task = repo.get_by_id(&id).await.unwrap();

    assert_eq!(task.name, "another one".to_owned());
    assert!(task.done);

    Ok(())
}
