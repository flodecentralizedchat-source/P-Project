use axum::{
    routing::{get, post},
    Router,
};
use p_project_core::database::MySqlDatabase;
use std::sync::Arc;

mod handlers;
mod middleware;
mod shared;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize middleware
    middleware::init_middleware();

    // Initialize database
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or("mysql://user:password@localhost/p_project".to_string());
    let db = MySqlDatabase::new(&db_url).await?;
    db.init_tables().await?;

    let app_state = shared::AppState { db: Arc::new(db) };

    // Build router
    let app = Router::new()
        .route("/", get(handlers::root))
        .route("/users", post(handlers::create_user))
        .route(
            "/users/:id",
            get(handlers::get_user).patch(handlers::update_user),
        )
        .route("/transfer", post(handlers::transfer_tokens))
        .route("/stake", post(handlers::stake_tokens))
        .route("/unstake", post(handlers::unstake_tokens))
        .route("/airdrop/claim", post(handlers::claim_airdrop))
        .route("/airdrop/create", post(handlers::create_airdrop))
        .route("/airdrop/batch-claim", post(handlers::batch_claim_airdrops))
        // Bridge endpoints
        .route("/bridge", post(handlers::bridge_tokens))
        .route("/bridge/status", post(handlers::get_bridge_status))
        // Performance metrics endpoint
        .route("/metrics", get(handlers::get_performance_metrics))
        // DAO endpoints
        .route("/dao/proposals", post(handlers::create_proposal))
        .route("/dao/proposals/vote", post(handlers::vote_on_proposal))
        .route("/dao/proposals/tally", post(handlers::tally_votes))
        .route("/dao/proposals/execute", post(handlers::execute_proposal))
        .route("/dao/delegate", post(handlers::delegate_vote))
        .route("/dao/proposals", get(handlers::get_proposals))
        // Staking endpoints
        .route("/staking/yield", post(handlers::calculate_staking_yield))
        .route("/staking/tiers", get(handlers::get_staking_tiers))
        // Airdrop endpoints
        .route("/airdrop/status", get(handlers::get_airdrop_status))
        .route("/airdrop/recipients", get(handlers::get_airdrop_recipients))
        // AI Service endpoints
        .route("/ai/verify-impact", post(handlers::verify_impact))
        .route("/ai/generate-nft-art", post(handlers::generate_peace_nft_art))
        .route("/ai/detect-fraud", post(handlers::detect_fraud))
        // IoT Service endpoints
        .route("/iot/register-donation-box", post(handlers::register_donation_box))
        .route("/iot/record-donation", post(handlers::record_donation))
        .route("/iot/donation-box-status", post(handlers::get_donation_box_status))
        .route("/iot/register-wristband", post(handlers::register_wristband))
        .route("/iot/add-funds-wristband", post(handlers::add_funds_to_wristband))
        .route("/iot/process-wristband-transaction", post(handlers::process_wristband_transaction))
        .route("/iot/wristband-status", post(handlers::get_wristband_status))
        .route("/iot/create-food-qr", post(handlers::create_food_qr))
        .route("/iot/claim-food-qr", post(handlers::claim_food_qr))
        .route("/iot/qr-status", post(handlers::get_qr_status))
        // Web2 Service endpoints
        .route("/web2/create-donation-widget", post(handlers::create_donation_widget))
        .route("/web2/process-social-donation", post(handlers::process_social_media_donation))
        .route("/web2/generate-widget-html", post(handlers::generate_widget_html))
        .route("/web2/create-youtube-tip-config", post(handlers::create_youtube_tip_config))
        .route("/web2/process-youtube-tip", post(handlers::process_youtube_tip))
        .route("/web2/register-messaging-bot", post(handlers::register_messaging_bot))
        .route("/web2/process-bot-command", post(handlers::process_bot_command))
        .with_state(app_state);

    // Run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}