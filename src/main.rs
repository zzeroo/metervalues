use std::env;

use sqlx::PgPool;
use tracing::info;

use metervalues::create_app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "metervalues=info,tower_http=info".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".into());

    let db = PgPool::connect(&database_url).await?;

    sqlx::migrate!("./migrations").run(&db).await?;

    let app = create_app(db);

    let listener = tokio::net::TcpListener::bind(&bind_address).await?;

    info!("Meter Values listening on http://{}", bind_address);

    axum::serve(listener, app).await?;

    Ok(())
}
