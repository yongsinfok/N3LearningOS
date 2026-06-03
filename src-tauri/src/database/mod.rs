use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{SqlitePool, migrate::Migrator};
use std::path::PathBuf;
use std::sync::OnceLock;

static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn init_database(app_dir: PathBuf) -> Result<SqlitePool, sqlx::Error> {
    let db_path = app_dir.join("n3_learning.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(&pool)
        .await?;

    sqlx::query("PRAGMA foreign_keys=ON;")
        .execute(&pool)
        .await?;

    MIGRATOR.run(&pool).await?;

    DB_POOL.set(pool.clone()).map_err(|_| {
        sqlx::Error::Protocol("Database pool already initialized".into())
    })?;

    Ok(pool)
}

pub fn get_pool() -> Result<&'static SqlitePool, sqlx::Error> {
    DB_POOL.get().ok_or_else(|| {
        sqlx::Error::Protocol("Database not initialized".into())
    })
}
