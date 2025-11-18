use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use p_project_core::database::MySqlDatabase;
use p_project_core::{
    ComponentStatus, EcosystemComponent, EcosystemComponentType, EcosystemLink,
    UniqueValueProposition,
};
use std::sync::Arc;
use std::time::Duration;
use tower::limit::ConcurrencyLimitLayer;
use tower::timeout::TimeoutLayer;

mod handlers;
mod middleware;
mod ratelimit;
mod shared;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize middleware
    crate::middleware::init_middleware();

    // Initialize database (env-only; fail if missing)
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "DATABASE_URL not set"))?;
    let db = MySqlDatabase::new(&db_url).await?;
    db.init_tables().await?;

    // Innovation state: UVPs, Partners, Ecosystem graph
    let uvp_engine = {
        let uvps = vec![
            UniqueValueProposition {
                id: "uvp_impact_oracles".into(),
                name: "Proof-of-Impact Oracles".into(),
                description: "AI+IoT verified impact boosts rewards".into(),
                metric_key: "impact_verified".into(),
                multiplier: 1.15,
            },
            UniqueValueProposition {
                id: "uvp_peace_staking".into(),
                name: "Peace Staking Bonus".into(),
                description: "Stakers for social good earn more".into(),
                metric_key: "peace_staker".into(),
                multiplier: 1.10,
            },
            UniqueValueProposition {
                id: "uvp_trusted_partner_tx".into(),
                name: "Trusted Partnership Flow".into(),
                description: "Transactions via trusted partners get a small boost".into(),
                metric_key: "trusted_partner_tx".into(),
                multiplier: 1.05,
            },
        ];
        p_project_core::UvpEngine::new(uvps)
    };

    let ecosystem_graph = {
        let mut g = p_project_core::EcosystemGraph::new();
        g.add_component(EcosystemComponent {
            id: "core".into(),
            name: "Core".into(),
            component_type: EcosystemComponentType::Service,
            version: "1.0.0".into(),
            status: ComponentStatus::Healthy,
            metadata: serde_json::json!({}),
        });
        g.add_component(EcosystemComponent {
            id: "api".into(),
            name: "API".into(),
            component_type: EcosystemComponentType::API,
            version: "1.0.0".into(),
            status: ComponentStatus::Healthy,
            metadata: serde_json::json!({}),
        });
        g.add_component(EcosystemComponent {
            id: "web".into(),
            name: "Web".into(),
            component_type: EcosystemComponentType::UI,
            version: "1.0.0".into(),
            status: ComponentStatus::Healthy,
            metadata: serde_json::json!({}),
        });
        let _ = g.add_link(EcosystemLink {
            from_id: "core".into(),
            to_id: "api".into(),
            relation: "exposes".into(),
        });
        let _ = g.add_link(EcosystemLink {
            from_id: "api".into(),
            to_id: "web".into(),
            relation: "serves".into(),
        });
        g
    };

    let partner_registry = p_project_core::PartnerRegistry::new();

    let app_state = shared::AppState {
        db: Arc::new(db),
        rate_limiter: Arc::new(crate::ratelimit::RateLimiter::from_env()),
        strict_rate_limiter: Arc::new(crate::ratelimit::RateLimiter::from_env_with_prefix(
            "STRICT_RATE_LIMIT_",
        )),
        uvp_engine: Arc::new(tokio::sync::RwLock::new(uvp_engine)),
        partner_registry: Arc::new(tokio::sync::RwLock::new(partner_registry)),
        ecosystem_graph: Arc::new(tokio::sync::RwLock::new(ecosystem_graph)),
    };

    // Build routers
    // Public endpoints
    let public = Router::new()
        .route("/", get(handlers::root))
        .route("/metrics", get(handlers::get_performance_metrics))
        .route("/staking/tiers", get(handlers::get_staking_tiers))
        .route("/airdrop/status", get(handlers::get_airdrop_status))
        .route("/airdrop/recipients", get(handlers::get_airdrop_recipients))
        .route("/tokenomics/summary", get(handlers::get_tokenomics_summary))
        .route(
            "/strategy/exchange-listings",
            get(handlers::get_exchange_listings_strategy),
        )
        .route("/learning/content", get(handlers::list_learning_content))
        .route("/learning/content/:id", get(handlers::get_learning_content))
        .route("/events", get(handlers::list_events))
        .route("/events/:id", get(handlers::get_event))
        .route(
            "/budget/alternatives",
            get(handlers::list_budget_alternatives),
        )
        .route(
            "/budget/alternatives/:id",
            get(handlers::get_budget_alternative),
        )
        .route(
            "/budget/alternatives/:id/simulate",
            post(handlers::simulate_budget_alternative),
        )
        // Innovation public endpoints
        .route("/innovation/uvp", get(handlers::get_uvp_summary))
        .route(
            "/innovation/ecosystem",
            get(handlers::get_ecosystem_overview),
        );

    // Protected endpoints (require JWT)
    let protected = Router::new()
        .route("/auth/whoami", get(handlers::whoami))
        .route("/users", post(handlers::create_user))
        .route(
            "/users/:id",
            get(handlers::get_user).patch(handlers::update_user),
        )
        .route("/transfer", post(handlers::transfer_tokens))
        // Referral endpoints
        .route(
            "/referrals/code/generate",
            post(handlers::generate_referral_code),
        )
        .route("/referrals/accept", post(handlers::accept_referral))
        .route("/referrals/stats/:id", get(handlers::get_referral_stats))
        // Remittance endpoints
        .route("/remittance/quote", post(handlers::remittance_quote))
        .route("/remittance/initiate", post(handlers::remittance_initiate))
        .route("/remittance/:id", get(handlers::remittance_get))
        .route(
            "/remittance/user/:id",
            get(handlers::remittance_list_for_user),
        )
        .route(
            "/learning/complete",
            post(handlers::record_learning_completion),
        )
        .route(
            "/learning/completions/:user_id",
            get(handlers::list_learning_completions_for_user),
        )
        .route("/stake", post(handlers::stake_tokens))
        .route("/unstake", post(handlers::unstake_tokens))
        .route("/airdrop/claim", post(handlers::claim_airdrop))
        // admin variant provided in admin router
        .route("/airdrop/batch-claim", post(handlers::batch_claim_airdrops))
        // Bridge endpoints
        .route("/bridge", post(handlers::bridge_tokens))
        .route("/bridge/status", post(handlers::get_bridge_status))
        // DAO endpoints
        .route("/dao/proposals", post(handlers::create_proposal))
        .route("/dao/proposals/vote", post(handlers::vote_on_proposal))
        .route("/dao/proposals/tally", post(handlers::tally_votes))
        .route("/dao/proposals/execute", post(handlers::execute_proposal))
        // admin variant provided in admin router
        .route("/dao/delegate", post(handlers::delegate_vote))
        .route("/dao/proposals", get(handlers::get_proposals))
        // Staking endpoints
        .route("/staking/yield", post(handlers::calculate_staking_yield))
        // Peace Staking endpoints
        .route(
            "/peace-staking/record-donation",
            post(handlers::record_donation_event),
        )
        .route(
            "/peace-staking/calculate-bonus",
            post(handlers::calculate_peace_staking_bonus),
        )
        // AI Service endpoints
        .route("/ai/verify-impact", post(handlers::verify_impact))
        .route(
            "/ai/generate-nft-art",
            post(handlers::generate_peace_nft_art),
        )
        .route("/ai/detect-fraud", post(handlers::detect_fraud))
        .route("/ai/generate-meme", post(handlers::generate_ai_meme))
        // IoT Service endpoints
        .route(
            "/iot/register-donation-box",
            post(handlers::register_donation_box),
        )
        .route("/iot/record-donation", post(handlers::record_donation))
        .route(
            "/iot/donation-box-status",
            post(handlers::get_donation_box_status),
        )
        .route(
            "/iot/register-wristband",
            post(handlers::register_wristband),
        )
        .route(
            "/iot/add-funds-wristband",
            post(handlers::add_funds_to_wristband),
        )
        .route(
            "/iot/process-wristband-transaction",
            post(handlers::process_wristband_transaction),
        )
        .route(
            "/iot/wristband-status",
            post(handlers::get_wristband_status),
        )
        .route("/iot/create-food-qr", post(handlers::create_food_qr))
        .route("/iot/claim-food-qr", post(handlers::claim_food_qr))
        .route("/iot/qr-status", post(handlers::get_qr_status))
        // Web2 Service endpoints
        // admin variant provided in admin router
        .route(
            "/web2/process-social-donation",
            post(handlers::process_social_media_donation),
        )
        .route(
            "/web2/generate-widget-html",
            post(handlers::generate_widget_html),
        )
        // admin variant provided in admin router
        .route(
            "/web2/process-youtube-tip",
            post(handlers::process_youtube_tip),
        )
        // admin variant provided in admin router
        .route(
            "/web2/process-bot-command",
            post(handlers::process_bot_command),
        )
        .route("/game/complete-mission", post(handlers::complete_mission))
        .route("/game/record-behavior", post(handlers::record_behavior))
        .route(
            "/game/balance/:player_id",
            get(handlers::get_player_balance),
        )
        .route(
            "/credit/add-impact-event",
            post(handlers::add_social_impact_event),
        )
        .route("/credit/request-loan", post(handlers::request_micro_loan))
        .route("/credit/repay-loan", post(handlers::repay_micro_loan))
        .route("/credit/loan/:loan_id", get(handlers::get_micro_loan))
        .route("/credit/score/:user_id", get(handlers::get_credit_score))
        // Merchant Service endpoints
        .route("/merchant", post(handlers::create_merchant))
        .route("/merchant/:id", get(handlers::get_merchant))
        .route(
            "/merchant/payment",
            post(handlers::process_merchant_payment),
        )
        .route("/merchant/qr", post(handlers::create_payment_qr))
        // Digital Goods endpoints
        .route(
            "/merchant/digital-goods",
            post(handlers::add_digital_goods_product),
        )
        .route(
            "/merchant/digital-goods/purchase",
            post(handlers::purchase_digital_goods),
        )
        .route(
            "/merchant/digital-goods/transaction",
            post(handlers::get_digital_goods_transaction),
        )
        // Cashback and Loyalty endpoints
        .route(
            "/merchant/cashback/process-purchase",
            post(handlers::process_purchase_with_cashback),
        )
        .route(
            "/merchant/cashback/loyalty-points",
            post(handlers::get_customer_loyalty_points),
        )
        .route(
            "/merchant/cashback/redeem-points",
            post(handlers::redeem_loyalty_points),
        )
        .route(
            "/merchant/cashback/transaction",
            post(handlers::get_cashback_transaction),
        )
        // Innovation protected endpoints
        .route(
            "/innovation/uvp/compute",
            post(handlers::compute_uvp_multiplier),
        )
        .route("/innovation/partners", get(handlers::list_partners))
        .route_layer(middleware::from_fn(crate::middleware::require_jwt));

    // Admin-only routes
    let admin = Router::new()
        .route("/airdrop/create", post(handlers::create_airdrop))
        .route("/events", post(handlers::create_event))
        .route(
            "/budget/alternatives",
            post(handlers::create_budget_alternative),
        )
        .route(
            "/web2/create-donation-widget",
            post(handlers::create_donation_widget),
        )
        .route(
            "/web2/create-youtube-tip-config",
            post(handlers::create_youtube_tip_config),
        )
        .route(
            "/web2/register-messaging-bot",
            post(handlers::register_messaging_bot),
        )
        .route(
            "/web2/create-ecommerce-integration",
            post(handlers::create_ecommerce_integration),
        )
        .route("/game/register-mission", post(handlers::register_mission))
        .route("/learning/content", post(handlers::create_learning_content))
        .route("/credit/register-ngo", post(handlers::register_ngo))
        .route(
            "/merchant/cashback/configure",
            post(handlers::configure_cashback),
        )
        // Innovation admin endpoints
        .route(
            "/innovation/partners/register",
            post(handlers::register_partner),
        )
        .route_layer(middleware::from_fn(crate::middleware::require_admin))
        .route_layer(middleware::from_fn(crate::middleware::require_jwt));

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .merge(admin)
        .route(
            "/web2/process-ecommerce-payment",
            post(handlers::process_ecommerce_payment),
        )
        .route(
            "/web2/verify-webhook",
            post(handlers::verify_webhook_signature),
        )
        .route("/web2/telegram/webhook", post(handlers::telegram_webhook))
        .route("/web2/discord/webhook", post(handlers::discord_webhook))
        .with_state(app_state)
        .layer(middleware::from_fn(crate::middleware::require_rate_limit))
        .layer(ConcurrencyLimitLayer::new(128))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(crate::middleware::build_body_limit_from_env())
        .layer(crate::middleware::build_cors_from_env());

    // Run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
