use tracing::{error, info};

pub async fn create_pool() -> sqlx::PgPool {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    match sqlx::PgPool::connect(&database_url).await {
        Ok(pool) => {
            info!("Connected to Postgres!");
            pool
        },
        Err(e) => {
            error!("Failed to connect to the database: {}", e);
            std::process::exit(1);
        }
    }
}