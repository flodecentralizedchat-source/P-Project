use axum::{
    routing::{get, post},
    Router,
};
use p_project_core::database::MySqlDatabase;
use std::sync::Arc;
use tokio::net::TcpListener;

mod handlers;
mod middleware;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<MySqlDatabase>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize middleware
    middleware::init_middleware();
    
    // Initialize database
    let db_url = std::env::var("DATABASE_URL").unwrap_or("mysql://user:password@localhost/p_project".to_string());
    let db = MySqlDatabase::new(&db_url).await?;
    db.init_tables().await?;
    
    let app_state = AppState {
        db: Arc::new(db),
    };
    
    // Build router
    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/users", post(handlers::create_user))
        .route("/users/:id", get(handlers::get_user))
        .route("/transfer", post(handlers::transfer_tokens))
        .route("/stake", post(handlers::stake_tokens))
        .route("/unstake", post(handlers::unstake_tokens))
        .route("/airdrop/claim", post(handlers::claim_airdrop))
        .route("/airdrop/create", post(handlers::create_airdrop))
        .route("/airdrop/batch-claim", post(handlers::batch_claim_airdrops))
        .with_state(app_state);
    
    // Run server
    let listener = TcpListener::bind("0.0.0.0:3001").await?;
    println!("Server running on http://localhost:3001");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}