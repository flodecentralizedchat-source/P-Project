use axum::{
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{NaiveDateTime, Utc};
use p_project_bridge::BridgeService;
use p_project_core::budget_alternatives::{BudgetAlternative, BudgetSchedulePoint, BudgetStrategy};
use p_project_core::database::{BalanceError, MySqlDatabase};
use p_project_core::models::{
    LearningCompletion, LearningContent, LearningContentType, Proposal, ProposalStatus, Remittance,
    TransactionType, User,
};
use p_project_core::{
    AIService, AIServiceConfig, CreditService, CreditServiceConfig, GameCurrencyConfig,
    GameCurrencyService, IoTService, IoTServiceConfig, TokenomicsService, Web2Service,
    Web2ServiceConfig,
};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use crate::shared::AppState;

// Root/health
pub async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "p-project-api",
        "status": "ok"
    }))
}

// --- Config helpers ---
fn env_ai_config() -> Result<AIServiceConfig, String> {
    let api_key = env::var("AI_API_KEY").map_err(|_| "AI_API_KEY not set".to_string())?;
    let model = env::var("AI_MODEL").unwrap_or_else(|_| "gpt-4".to_string());
    let temperature: f32 = env::var("AI_TEMPERATURE")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(0.7);
    Ok(AIServiceConfig {
        api_key,
        model,
        temperature,
    })
}

fn env_iot_config() -> Result<IoTServiceConfig, String> {
    let api_endpoint =
        env::var("IOT_API_ENDPOINT").map_err(|_| "IOT_API_ENDPOINT not set".to_string())?;
    let auth_token =
        env::var("IOT_AUTH_TOKEN").map_err(|_| "IOT_AUTH_TOKEN not set".to_string())?;
    Ok(IoTServiceConfig {
        api_endpoint,
        auth_token,
    })
}

fn env_web2_config() -> Result<Web2ServiceConfig, String> {
    // Collect API keys if present; optional
    let mut api_keys = std::collections::HashMap::new();
    if let Ok(v) = env::var("WEB2_FACEBOOK_KEY") {
        api_keys.insert("facebook".to_string(), v);
    }
    if let Ok(v) = env::var("WEB2_YOUTUBE_KEY") {
        api_keys.insert("youtube".to_string(), v);
    }
    if let Ok(v) = env::var("WEB2_TELEGRAM_KEY") {
        api_keys.insert("telegram".to_string(), v);
    }
    if let Ok(v) = env::var("WEB2_DISCORD_KEY") {
        api_keys.insert("discord".to_string(), v);
    }
    let webhook_url = env::var("WEB2_WEBHOOK_URL").unwrap_or_else(|_| "".to_string());
    Ok(Web2ServiceConfig {
        api_keys,
        webhook_url,
    })
}

fn env_learning_reward_source() -> Option<String> {
    env::var("LEARNING_REWARD_SOURCE_ID").ok()
}

#[derive(Debug, Clone)]
struct RemittanceConfig {
    fee_rate: f64,
    min_fee: f64,
    fee_account_id: Option<String>,
}

fn env_remittance_config() -> RemittanceConfig {
    let fee_rate = env::var("REMITTANCE_FEE_RATE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.001);
    let min_fee = env::var("REMITTANCE_MIN_FEE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let fee_account_id = env::var("REMITTANCE_FEE_ACCOUNT_ID").ok();
    RemittanceConfig {
        fee_rate,
        min_fee,
        fee_account_id,
    }
}

fn compute_remittance_fee(amount: Decimal, cfg: &RemittanceConfig) -> Decimal {
    let rate_fee = amount * Decimal::from_f64(cfg.fee_rate).unwrap_or(Decimal::ZERO);
    let min_fee = Decimal::from_f64(cfg.min_fee).unwrap_or(Decimal::ZERO);
    if rate_fee < min_fee {
        min_fee
    } else {
        rate_fee
    }
}

fn env_credit_config() -> Result<CreditServiceConfig, String> {
    let min_credit_score = env::var("CREDIT_MIN_SCORE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(60.0);
    let max_loan_amount = env::var("CREDIT_MAX_LOAN_AMOUNT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1000.0);
    let collateral_ratio = env::var("CREDIT_COLLATERAL_RATIO")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.5);
    let default_interest_rate = env::var("CREDIT_INTEREST_RATE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.08);
    let base_score = env::var("CREDIT_BASE_SCORE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(50.0);
    let max_duration_days = env::var("CREDIT_MAX_DURATION_DAYS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(30);

    Ok(CreditServiceConfig {
        min_credit_score,
        max_loan_amount,
        collateral_ratio,
        default_interest_rate,
        base_score,
        max_duration_days,
    })
}

fn env_tokenomics_path() -> Result<String, String> {
    Ok(std::env::var("TOKENOMICS_CSV_PATH")
        .unwrap_or_else(|_| "tokenomics_master_price.csv".to_string()))
}

fn env_game_currency_config() -> Result<GameCurrencyConfig, String> {
    let base_mission_reward = env::var("GAME_BASE_MISSION_REWARD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(25.0);
    let mut behavior_rewards = HashMap::new();
    behavior_rewards.insert(
        p_project_core::PositiveBehavior::HelpingHands,
        env::var("GAME_BEHAVIOR_HELPING_HANDS")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(15.0),
    );
    behavior_rewards.insert(
        p_project_core::PositiveBehavior::EnvironmentalCare,
        env::var("GAME_BEHAVIOR_ENVIRONMENTAL")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(12.0),
    );
    behavior_rewards.insert(
        p_project_core::PositiveBehavior::ConflictResolution,
        env::var("GAME_BEHAVIOR_CONFLICT")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(18.0),
    );
    behavior_rewards.insert(
        p_project_core::PositiveBehavior::EducationChampion,
        env::var("GAME_BEHAVIOR_EDUCATION")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(10.0),
    );

    Ok(GameCurrencyConfig {
        base_mission_reward,
        behavior_rewards,
    })
}

fn ensure_roles(
    claims: &crate::middleware::Claims,
    allowed: &[&str],
) -> Result<(), (StatusCode, Json<ErrorResponse>)> {
    let role = claims.role.as_deref().unwrap_or("user");
    if allowed.iter().any(|r| *r == role) {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "forbidden".to_string(),
            }),
        ))
    }
}

#[derive(Debug, Serialize)]
pub struct WhoAmIResponse {
    pub sub: String,
    pub role: Option<String>,
    pub exp: usize,
}

pub async fn whoami(
    headers: axum::http::HeaderMap,
) -> Result<Json<WhoAmIResponse>, (StatusCode, Json<ErrorResponse>)> {
    let secret = std::env::var("JWT_SECRET").map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
            }),
        )
    })?;
    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("").trim();
    if token.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
            }),
        ));
    }
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
    let data = jsonwebtoken::decode::<crate::middleware::Claims>(token, &key, &validation)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "unauthorized".to_string(),
                }),
            )
        })?;
    let c = data.claims;
    Ok(Json(WhoAmIResponse {
        sub: c.sub,
        role: c.role,
        exp: c.exp,
    }))
}

// ---------------- Innovation Features: UVP, Partnerships, Ecosystem ----------------

#[derive(Debug, Serialize, Deserialize)]
pub struct UvpComputeResponse {
    pub multiplier: f64,
}

pub async fn get_uvp_summary(
    State(state): State<AppState>,
) -> Result<Json<Vec<p_project_core::UvpSummaryItem>>, (StatusCode, Json<ErrorResponse>)> {
    let guard = state.uvp_engine.read().await;
    let summary = guard.summary();
    Ok(Json(summary))
}

pub async fn compute_uvp_multiplier(
    State(state): State<AppState>,
    Json(ctx): Json<serde_json::Value>,
) -> Result<Json<UvpComputeResponse>, (StatusCode, Json<ErrorResponse>)> {
    let guard = state.uvp_engine.read().await;
    let m = guard.compute_reward_multiplier(&ctx);
    Ok(Json(UvpComputeResponse { multiplier: m }))
}

#[derive(Debug, Serialize)]
pub struct EcosystemOverviewResponse {
    pub components: Vec<p_project_core::EcosystemComponent>,
    pub links: Vec<p_project_core::EcosystemLink>,
    pub health: p_project_core::HealthSummary,
}

pub async fn get_ecosystem_overview(
    State(state): State<AppState>,
) -> Result<Json<EcosystemOverviewResponse>, (StatusCode, Json<ErrorResponse>)> {
    let g = state.ecosystem_graph.read().await;
    let comps = g.list_components().into_iter().cloned().collect();
    let links = g.list_links().clone();
    let health = g.health_summary();
    Ok(Json(EcosystemOverviewResponse {
        components: comps,
        links,
        health,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RegisterPartnerRequest {
    pub name: String,
    pub integration_type: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
    pub webhook_secret: Option<String>,
    #[serde(default = "default_true")]
    pub active: bool,
}

fn default_true() -> bool {
    true
}

fn parse_integration_type(s: &str) -> Option<p_project_core::PartnerIntegrationType> {
    use p_project_core::PartnerIntegrationType as T;
    match s.to_ascii_lowercase().as_str() {
        "payment" => Some(T::Payment),
        "oracle" => Some(T::Oracle),
        "identity" => Some(T::Identity),
        "messaging" => Some(T::Messaging),
        "ecommerce" => Some(T::ECommerce),
        "carboncredits" | "carbon_credits" | "carbon" => Some(T::CarbonCredits),
        "defi" => Some(T::DeFi),
        "nft" => Some(T::NFT),
        "analytics" => Some(T::Analytics),
        "custom" => Some(T::Custom),
        _ => None,
    }
}

pub async fn register_partner(
    State(state): State<AppState>,
    Json(req): Json<RegisterPartnerRequest>,
) -> Result<Json<p_project_core::Partner>, (StatusCode, Json<ErrorResponse>)> {
    let Some(integration_type) = parse_integration_type(&req.integration_type) else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_integration_type".to_string(),
            }),
        ));
    };
    let mut reg = state.partner_registry.write().await;
    let partner = reg.register_partner(
        &req.name,
        integration_type,
        req.metadata,
        req.webhook_secret,
        req.active,
    );
    Ok(Json(partner))
}

pub async fn list_partners(
    State(state): State<AppState>,
) -> Result<Json<Vec<p_project_core::Partner>>, (StatusCode, Json<ErrorResponse>)> {
    let reg = state.partner_registry.read().await;
    let mut v: Vec<_> = reg.list_partners().into_iter().cloned().collect();
    v.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(Json(v))
}

// Users
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub wallet_address: String,
}

#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    pub id: String,
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Basic validation and compliance checks
    if !is_valid_username(&req.username) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_username".to_string(),
            }),
        ));
    }
    if !is_valid_wallet(&req.wallet_address) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_wallet".to_string(),
            }),
        ));
    }
    if is_username_blocked(&req.username) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "username_blocklisted".to_string(),
            }),
        ));
    }
    if is_wallet_blocked(&req.wallet_address) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "wallet_blocklisted".to_string(),
            }),
        ));
    }
    let id = p_project_core::utils::generate_id();
    match state
        .db
        .create_user(&id, &req.username, &req.wallet_address)
        .await
    {
        Ok(_user) => Ok(Json(CreateUserResponse { id })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.get_user(&id).await {
        Ok(Some(u)) => Ok(Json(u)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "user_not_found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLearningContentRequest {
    pub title: String,
    pub description: String,
    pub content_type: LearningContentType,
    pub reward_tokens: Decimal,
    pub reward_points: i64,
}

#[derive(Debug, Deserialize)]
pub struct LearningContentListQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct LearningCompletionRequest {
    pub user_id: String,
    pub content_id: String,
    pub proof_reference: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LearningCompletionListQuery {
    pub limit: Option<i64>,
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .db
        .update_user(&id, req.username.as_deref(), req.wallet_address.as_deref())
        .await
    {
        Ok(Some(u)) => Ok(Json(u)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "user_not_found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

// Transfer
#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from_user_id: String,
    pub to_user_id: String,
    pub amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct TransferResponse {
    pub transaction_id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub amount: Decimal,
}

pub async fn transfer_tokens(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    if req.amount <= Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_amount".to_string(),
            }),
        ));
    }
    // Compliance: blocklisted wallets cannot participate
    if let (Ok(Some(from_u)), Ok(Some(to_u))) = (
        state.db.get_user(&req.from_user_id).await,
        state.db.get_user(&req.to_user_id).await,
    ) {
        if is_wallet_blocked(&from_u.wallet_address) || is_wallet_blocked(&to_u.wallet_address) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "wallet_blocklisted".to_string(),
                }),
            ));
        }
    }
    let tx_id = p_project_core::utils::generate_id();
    match state
        .db
        .transfer_tokens(
            &tx_id,
            &req.from_user_id,
            &req.to_user_id,
            req.amount.round_dp(8),
            TransactionType::Transfer,
        )
        .await
    {
        Ok(()) => Ok(Json(TransferResponse {
            transaction_id: tx_id,
            from_user_id: req.from_user_id,
            to_user_id: req.to_user_id,
            amount: req.amount.round_dp(8),
        })),
        Err(BalanceError::InvalidAmount) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_amount".to_string(),
            }),
        )),
        Err(BalanceError::InsufficientBalance) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "insufficient_balance".to_string(),
            }),
        )),
        Err(BalanceError::UserNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "user_not_found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

// -------------------- Remittance --------------------
#[derive(Debug, Deserialize)]
pub struct RemittanceQuoteRequest {
    pub from_user_id: String,
    pub to_user_id: String,
    pub amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct RemittanceQuoteResponse {
    pub amount: Decimal,
    pub fee: Decimal,
    pub total_debit: Decimal,
    pub net_amount: Decimal,
    pub fee_rate: f64,
}

#[derive(Debug, Deserialize)]
pub struct RemittanceInitiateRequest {
    pub from_user_id: String,
    pub to_user_id: String,
    pub amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct RemittanceInitiateResponse {
    pub remittance_id: String,
    pub status: String,
    pub amount: Decimal,
    pub fee: Decimal,
    pub net_amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct RemittanceListQuery {
    pub limit: Option<i64>,
}

pub async fn remittance_quote(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<RemittanceQuoteRequest>,
) -> Result<Json<RemittanceQuoteResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    if req.amount <= Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_amount".to_string(),
            }),
        ));
    }
    let cfg = env_remittance_config();
    let fee = compute_remittance_fee(req.amount, &cfg).round_dp(8);
    let net_amount = req.amount.round_dp(8);
    let total_debit = (net_amount + fee).round_dp(8);
    Ok(Json(RemittanceQuoteResponse {
        amount: net_amount,
        fee,
        total_debit,
        net_amount,
        fee_rate: cfg.fee_rate,
    }))
}

pub async fn remittance_initiate(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<RemittanceInitiateRequest>,
) -> Result<Json<RemittanceInitiateResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    if req.amount <= Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_amount".to_string(),
            }),
        ));
    }

    let cfg = env_remittance_config();
    let fee = compute_remittance_fee(req.amount, &cfg).round_dp(8);
    let remittance_id = p_project_core::utils::generate_id();
    match state
        .db
        .process_remittance(
            &remittance_id,
            &req.from_user_id,
            &req.to_user_id,
            req.amount.round_dp(8),
            fee,
            cfg.fee_account_id.as_deref(),
        )
        .await
    {
        Ok(remit) => Ok(Json(RemittanceInitiateResponse {
            remittance_id,
            status: match remit.status {
                p_project_core::models::RemittanceStatus::Initiated => "Initiated",
                p_project_core::models::RemittanceStatus::Completed => "Completed",
                p_project_core::models::RemittanceStatus::Failed => "Failed",
            }
            .to_string(),
            amount: remit.amount,
            fee: remit.fee,
            net_amount: remit.net_amount,
        })),
        Err(BalanceError::InvalidAmount) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_amount".to_string(),
            }),
        )),
        Err(BalanceError::InsufficientBalance) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "insufficient_balance".to_string(),
            }),
        )),
        Err(BalanceError::UserNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "user_not_found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn remittance_get(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Path(id): Path<String>,
) -> Result<Json<Remittance>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    match state.db.get_remittance(&id).await {
        Ok(Some(remit)) => {
            if claims.role.as_deref() != Some("admin")
                && claims.sub != remit.from_user_id
                && claims.sub != remit.to_user_id
            {
                return Err((
                    StatusCode::FORBIDDEN,
                    Json(ErrorResponse {
                        error: "forbidden".to_string(),
                    }),
                ));
            }
            Ok(Json(remit))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn remittance_list_for_user(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Path(user_id): Path<String>,
    Query(query): Query<RemittanceListQuery>,
) -> Result<Json<Vec<Remittance>>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    if claims.role.as_deref() != Some("admin") && claims.sub != user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "forbidden".to_string(),
            }),
        ));
    }
    let limit = query.limit.unwrap_or(50).max(1).min(200);
    match state.db.list_user_remittances(&user_id, limit).await {
        Ok(list) => Ok(Json(list)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

// -------------------- Learning --------------------
pub async fn create_learning_content(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<CreateLearningContentRequest>,
) -> Result<Json<LearningContent>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["admin"])?;
    let content_id = p_project_core::utils::generate_id();
    match state
        .db
        .register_learning_content(
            &content_id,
            &req.title,
            &req.description,
            req.content_type,
            req.reward_tokens.round_dp(8),
            req.reward_points,
        )
        .await
    {
        Ok(content) => Ok(Json(content)),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn list_learning_content(
    State(state): State<AppState>,
    Query(query): Query<LearningContentListQuery>,
) -> Result<Json<Vec<LearningContent>>, (StatusCode, Json<ErrorResponse>)> {
    let limit = query.limit.unwrap_or(50).max(1).min(200);
    match state.db.list_learning_content(limit).await {
        Ok(list) => Ok(Json(list)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn get_learning_content(
    State(state): State<AppState>,
    Path(content_id): Path<String>,
) -> Result<Json<LearningContent>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.get_learning_content(&content_id).await {
        Ok(Some(content)) => Ok(Json(content)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "learning_content_not_found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn record_learning_completion(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<LearningCompletionRequest>,
) -> Result<Json<LearningCompletion>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    if claims.role.as_deref() != Some("admin") && claims.sub != req.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "forbidden".to_string(),
            }),
        ));
    }
    let completion_id = p_project_core::utils::generate_id();
    let reward_source = env_learning_reward_source();
    match state
        .db
        .record_learning_completion(
            &completion_id,
            &req.user_id,
            &req.content_id,
            req.proof_reference.as_deref(),
            reward_source.as_deref(),
        )
        .await
    {
        Ok(record) => Ok(Json(record)),
        Err(BalanceError::InvalidAmount) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_amount".to_string(),
            }),
        )),
        Err(BalanceError::AlreadyCompleted) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "already_completed".to_string(),
            }),
        )),
        Err(BalanceError::LearningContentNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "learning_content_not_found".to_string(),
            }),
        )),
        Err(BalanceError::UserNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "user_not_found".to_string(),
            }),
        )),
        Err(BalanceError::Sql(err)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: err.to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn list_learning_completions_for_user(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Path(user_id): Path<String>,
    Query(query): Query<LearningCompletionListQuery>,
) -> Result<Json<Vec<LearningCompletion>>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    if claims.role.as_deref() != Some("admin") && claims.sub != user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "forbidden".to_string(),
            }),
        ));
    }
    let limit = query.limit.unwrap_or(50).max(1).min(200);
    match state
        .db
        .list_user_learning_completions(&user_id, limit)
        .await
    {
        Ok(list) => Ok(Json(list)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

// Staking
#[derive(Debug, Deserialize)]
pub struct StakeRequest {
    pub user_id: String,
    pub amount: Decimal,
    pub duration_days: i64,
}

#[derive(Debug, Serialize)]
pub struct StakingInfoResponse {
    pub user_id: String,
    pub amount: Decimal,
    pub start_time: String,
    pub end_time: Option<String>,
    pub rewards_earned: Decimal,
}

pub async fn stake_tokens(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<StakeRequest>,
) -> Result<Json<StakingInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    if req.amount <= Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_amount".to_string(),
            }),
        ));
    }
    let stake_id = p_project_core::utils::generate_id();
    match state
        .db
        .stake_tokens(
            &stake_id,
            &req.user_id,
            req.amount.round_dp(8),
            req.duration_days,
        )
        .await
    {
        Ok(info) => Ok(Json(StakingInfoResponse {
            user_id: info.user_id,
            amount: info.amount,
            start_time: info.start_time.to_string(),
            end_time: info.end_time.map(|t| t.to_string()),
            rewards_earned: info.rewards_earned,
        })),
        Err(BalanceError::InvalidAmount) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_amount".to_string(),
            }),
        )),
        Err(BalanceError::InsufficientBalance) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "insufficient_balance".to_string(),
            }),
        )),
        Err(BalanceError::UserNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "user_not_found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct UnstakeRequest {
    pub user_id: String,
}

pub async fn unstake_tokens(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<UnstakeRequest>,
) -> Result<Json<StakingInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    match state.db.unstake_tokens(&req.user_id, None).await {
        Ok(info) => Ok(Json(StakingInfoResponse {
            user_id: info.user_id,
            amount: info.amount,
            start_time: info.start_time.to_string(),
            end_time: info.end_time.map(|t| t.to_string()),
            rewards_earned: info.rewards_earned,
        })),
        Err(BalanceError::UserNotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "user_not_found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

// Airdrop
#[derive(Debug, Deserialize)]
pub struct CreateAirdropRequest {
    pub total_amount: Decimal,
    pub recipients: Option<Vec<RecipientAmount>>, // optional batch insert
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RecipientAmount {
    pub user_id: String,
    pub amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct CreateAirdropResponse {
    pub airdrop_id: String,
}

pub async fn create_airdrop(
    State(state): State<AppState>,
    Json(req): Json<CreateAirdropRequest>,
) -> Result<Json<CreateAirdropResponse>, (StatusCode, Json<ErrorResponse>)> {
    if req.total_amount <= Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_total_amount".to_string(),
            }),
        ));
    }
    let airdrop_id = p_project_core::utils::generate_id();
    state
        .db
        .create_airdrop(&airdrop_id, req.total_amount.round_dp(8), None, None)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    if let Some(list) = req.recipients.clone() {
        let vec_pairs: Vec<(String, Decimal)> = list
            .into_iter()
            .map(|r| (r.user_id, r.amount.round_dp(8)))
            .collect();
        state
            .db
            .add_airdrop_recipients(&airdrop_id, &vec_pairs, Some("default"))
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: e.to_string(),
                    }),
                )
            })?;
    }

    Ok(Json(CreateAirdropResponse { airdrop_id }))
}

#[derive(Debug, Deserialize)]
pub struct ClaimAirdropRequest {
    pub airdrop_id: String,
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct AirdropClaimResponse {
    pub airdrop_id: String,
    pub user_id: String,
    pub amount: Decimal,
    pub message: String,
}

pub async fn claim_airdrop(
    State(state): State<AppState>,
    Json(req): Json<ClaimAirdropRequest>,
) -> Result<Json<AirdropClaimResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.claim_airdrop(&req.airdrop_id, &req.user_id).await {
        Ok(amount) => Ok(Json(AirdropClaimResponse {
            airdrop_id: req.airdrop_id,
            user_id: req.user_id,
            amount,
            message: "claimed".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct BatchClaimRequest {
    pub airdrop_id: String,
    pub user_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchClaimResponse {
    pub claimed: Vec<RecipientAmount>,
}

pub async fn batch_claim_airdrops(
    State(state): State<AppState>,
    Json(req): Json<BatchClaimRequest>,
) -> Result<Json<BatchClaimResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .db
        .batch_claim_airdrops(&req.airdrop_id, &req.user_ids)
        .await
    {
        Ok(items) => Ok(Json(BatchClaimResponse {
            claimed: items
                .into_iter()
                .map(|(user_id, amount)| RecipientAmount { user_id, amount })
                .collect(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

// Bridge
#[derive(Debug, Deserialize)]
pub struct BridgeRequest {
    pub user_id: String,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct BridgeResponse {
    pub transaction_id: String,
}

pub async fn bridge_tokens(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<BridgeRequest>,
) -> Result<Json<BridgeResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    if req.amount <= Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_amount".to_string(),
            }),
        ));
    }
    let svc = BridgeService::new(state.db.clone());
    match svc
        .bridge_tokens(
            &req.user_id,
            &req.from_chain,
            &req.to_chain,
            req.amount.round_dp(8).to_f64().unwrap_or(0.0),
        )
        .await
    {
        Ok(tx_id) => Ok(Json(BridgeResponse {
            transaction_id: tx_id,
        })),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e }))),
    }
}

#[derive(Debug, Deserialize)]
pub struct BridgeStatusRequest {
    pub transaction_id: String,
}

#[derive(Debug, Serialize)]
pub struct BridgeStatusResponse {
    pub transaction_id: String,
    pub status: String,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: Decimal,
}

pub async fn get_bridge_status(
    State(state): State<AppState>,
    Json(req): Json<BridgeStatusRequest>,
) -> Result<Json<BridgeStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    let svc = BridgeService::new(state.db.clone());
    match svc.get_bridge_status(&req.transaction_id).await {
        Ok(s) => Ok(Json(BridgeStatusResponse {
            transaction_id: s.tx_id,
            status: s.status,
            from_chain: s.from_chain,
            to_chain: s.to_chain,
            amount: Decimal::from_f64(s.amount).unwrap_or(Decimal::ZERO),
        })),
        Err(_e) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
            }),
        )),
    }
}

// Metrics and dashboards
pub async fn get_performance_metrics() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "uptime": "unknown", "ok": true }))
}

#[derive(Debug, Deserialize)]
pub struct StakingYieldRequest {
    pub amount: Decimal,
    pub duration_days: i64,
}

#[derive(Debug, Serialize)]
pub struct StakingYieldResponse {
    pub projected_rewards: Decimal,
    pub total_return: Decimal,
    pub apy_rate: f64,
}

pub async fn calculate_staking_yield(
    Json(req): Json<StakingYieldRequest>,
) -> Result<Json<StakingYieldResponse>, (StatusCode, Json<ErrorResponse>)> {
    if req.amount <= Decimal::ZERO || req.duration_days <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_input".to_string(),
            }),
        ));
    }
    // Simple linear APY example for calculator (placeholder)
    let apy_rate = if req.duration_days >= 365 {
        0.20
    } else if req.duration_days >= 90 {
        0.10
    } else {
        0.05
    };
    let projected_rewards = Decimal::from_f64(apy_rate).unwrap_or(Decimal::ZERO)
        * req.amount
        * Decimal::from_f64(req.duration_days as f64 / 365.0).unwrap_or(Decimal::ZERO);
    let total_return = req.amount + projected_rewards;
    Ok(Json(StakingYieldResponse {
        projected_rewards: projected_rewards.round_dp(8),
        total_return: total_return.round_dp(8),
        apy_rate,
    }))
}

#[derive(Debug, Serialize)]
pub struct StakingTierResponse {
    pub name: String,
    pub min_amount: f64,
    pub duration_days: i64,
    pub apy_rate: f64,
}

pub async fn get_staking_tiers() -> Json<Vec<StakingTierResponse>> {
    Json(vec![
        StakingTierResponse {
            name: "Basic".into(),
            min_amount: 100.0,
            duration_days: 30,
            apy_rate: 0.05,
        },
        StakingTierResponse {
            name: "Silver".into(),
            min_amount: 1000.0,
            duration_days: 90,
            apy_rate: 0.10,
        },
        StakingTierResponse {
            name: "Gold".into(),
            min_amount: 10000.0,
            duration_days: 365,
            apy_rate: 0.20,
        },
    ])
}

// Airdrop status for dashboard (mock/demo)
#[derive(Debug, Serialize)]
pub struct AirdropStatusResponse {
    pub total_airdrops: u32,
    pub total_recipients: u32,
    pub total_distributed: f64,
}

pub async fn get_airdrop_status() -> Json<AirdropStatusResponse> {
    Json(AirdropStatusResponse {
        total_airdrops: 0,
        total_recipients: 0,
        total_distributed: 0.0,
    })
}

#[derive(Debug, Serialize)]
pub struct AirdropRecipientResponse {
    pub user_id: String,
    pub amount: f64,
}

pub async fn get_airdrop_recipients() -> Json<Vec<AirdropRecipientResponse>> {
    Json(Vec::new())
}

// ---------------- Referrals ----------------
#[derive(Debug, Serialize)]
pub struct GenerateReferralCodeResponse {
    pub code: String,
}

pub async fn generate_referral_code(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
) -> Result<Json<GenerateReferralCodeResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = &claims.sub;
    match state.db.upsert_referral_code(user_id).await {
        Ok(code) => Ok(Json(GenerateReferralCodeResponse { code })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct AcceptReferralRequest {
    pub code: String,
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct AcceptReferralResponse {
    pub accepted: bool,
}

pub async fn accept_referral(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<AcceptReferralRequest>,
) -> Result<Json<AcceptReferralResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    if claims.role.as_deref() != Some("admin") && claims.sub != req.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "forbidden".to_string(),
            }),
        ));
    }
    match state.db.accept_referral(&req.code, &req.user_id).await {
        Ok(_) => Ok(Json(AcceptReferralResponse { accepted: true })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Serialize)]
pub struct ReferralStatsResponse {
    pub user_id: String,
    pub referred_count: i64,
}

pub async fn get_referral_stats(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<ReferralStatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.get_referral_stats(&user_id).await {
        Ok(cnt) => Ok(Json(ReferralStatsResponse {
            user_id,
            referred_count: cnt,
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

// ---------------- Community Events (AMAs & Events) ----------------
#[derive(Debug, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub description: String,
    pub event_type: String,      // "AMA" | "CommunityEvent" | "Other"
    pub scheduled_start: String, // ISO8601
    pub scheduled_end: Option<String>,
    pub link: Option<String>,
    pub created_by: String,
}

#[derive(Debug, Serialize)]
pub struct CreateEventResponse {
    pub id: String,
}

pub async fn create_event(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<CreateEventRequest>,
) -> Result<Json<CreateEventResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["admin"])?;
    let id = p_project_core::utils::generate_id();
    let start = chrono::DateTime::parse_from_rfc3339(&req.scheduled_start)
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "invalid_start".to_string(),
                }),
            )
        })?
        .naive_utc();
    let end = match &req.scheduled_end {
        Some(s) => Some(
            chrono::DateTime::parse_from_rfc3339(s)
                .map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "invalid_end".to_string(),
                        }),
                    )
                })?
                .naive_utc(),
        ),
        None => None,
    };
    state
        .db
        .create_event(
            &id,
            &req.title,
            &req.description,
            &req.event_type,
            start,
            end,
            req.link.as_deref(),
            &req.created_by,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;
    Ok(Json(CreateEventResponse { id }))
}

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub event_type: String,
    pub scheduled_start: String,
    pub scheduled_end: Option<String>,
    pub link: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

pub async fn list_events(
    State(state): State<AppState>,
    Query(query): Query<LearningContentListQuery>,
) -> Result<Json<Vec<EventResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let limit = query.limit.unwrap_or(50).max(1).min(200);
    match state.db.list_events(limit).await {
        Ok(rows) => Ok(Json(
            rows.into_iter()
                .map(
                    |(
                        id,
                        title,
                        description,
                        event_type,
                        scheduled_start,
                        scheduled_end,
                        link,
                        created_by,
                        created_at,
                    )| EventResponse {
                        id,
                        title,
                        description,
                        event_type,
                        scheduled_start: chrono::NaiveDateTime::and_utc(scheduled_start)
                            .to_rfc3339(),
                        scheduled_end: scheduled_end
                            .map(|v| chrono::NaiveDateTime::and_utc(v).to_rfc3339()),
                        link,
                        created_by,
                        created_at: chrono::NaiveDateTime::and_utc(created_at).to_rfc3339(),
                    },
                )
                .collect(),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<EventResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.get_event(&id).await {
        Ok(Some((
            id,
            title,
            description,
            event_type,
            scheduled_start,
            scheduled_end,
            link,
            created_by,
            created_at,
        ))) => Ok(Json(EventResponse {
            id,
            title,
            description,
            event_type,
            scheduled_start: chrono::NaiveDateTime::and_utc(scheduled_start).to_rfc3339(),
            scheduled_end: scheduled_end.map(|v| chrono::NaiveDateTime::and_utc(v).to_rfc3339()),
            link,
            created_by,
            created_at: chrono::NaiveDateTime::and_utc(created_at).to_rfc3339(),
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}
// ---------------- Budget Alternatives ----------------
#[derive(Debug, Deserialize)]
pub struct CreateBudgetAlternativeRequest {
    pub option_name: String,
    pub details: String,
    pub strategy: String,
    pub start_amount: Decimal,
    pub growth_rate: Decimal,
    pub duration_months: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct BudgetAlternativeResponse {
    pub id: String,
    pub option_name: String,
    pub details: String,
    pub strategy: String,
    pub start_amount: Decimal,
    pub growth_rate: Decimal,
    pub duration_months: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct BudgetSchedulePointResponse {
    pub month: i64,
    pub amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct BudgetSimulationRequest {
    pub months: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct BudgetSimulationResponse {
    pub id: String,
    pub option_name: String,
    pub schedule: Vec<BudgetSchedulePointResponse>,
}

impl From<BudgetAlternative> for BudgetAlternativeResponse {
    fn from(alt: BudgetAlternative) -> Self {
        BudgetAlternativeResponse {
            id: alt.id,
            option_name: alt.option_name,
            details: alt.details,
            strategy: alt.strategy.as_str().to_string(),
            start_amount: alt.start_amount,
            growth_rate: alt.growth_rate,
            duration_months: alt.duration_months,
            created_at: chrono::DateTime::<Utc>::from_utc(alt.created_at, Utc).to_rfc3339(),
        }
    }
}

impl From<BudgetSchedulePoint> for BudgetSchedulePointResponse {
    fn from(point: BudgetSchedulePoint) -> Self {
        BudgetSchedulePointResponse {
            month: point.month,
            amount: point.amount,
        }
    }
}

pub async fn create_budget_alternative(
    State(state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<CreateBudgetAlternativeRequest>,
) -> Result<Json<BudgetAlternativeResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["admin"])?;
    if req.start_amount <= Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_start_amount".to_string(),
            }),
        ));
    }
    if req.growth_rate < Decimal::ZERO {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_growth_rate".to_string(),
            }),
        ));
    }

    let duration = req.duration_months.unwrap_or(12).max(1);
    let id = p_project_core::utils::generate_id();
    match state
        .db
        .create_budget_alternative(
            &id,
            &req.option_name,
            &req.details,
            BudgetStrategy::from_str(&req.strategy),
            req.start_amount.round_dp(8),
            req.growth_rate.round_dp(8),
            duration,
        )
        .await
    {
        Ok(result) => Ok(Json(result.into())),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn list_budget_alternatives(
    State(state): State<AppState>,
    Query(query): Query<LearningContentListQuery>,
) -> Result<Json<Vec<BudgetAlternativeResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let limit = query.limit.unwrap_or(50).max(1).min(200);
    match state.db.list_budget_alternatives(limit).await {
        Ok(items) => Ok(Json(items.into_iter().map(Into::into).collect())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn get_budget_alternative(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<BudgetAlternativeResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.get_budget_alternative(&id).await {
        Ok(Some(item)) => Ok(Json(item.into())),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn simulate_budget_alternative(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<BudgetSimulationRequest>,
) -> Result<Json<BudgetSimulationResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.get_budget_alternative(&id).await {
        Ok(Some(item)) => {
            let months = req.months.filter(|m| *m > 0);
            let schedule = item.simulate_schedule(months);
            Ok(Json(BudgetSimulationResponse {
                id: item.id.clone(),
                option_name: item.option_name.clone(),
                schedule: schedule.into_iter().map(Into::into).collect(),
            }))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "budget_alternative_not_found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

// ---------------- DAO handlers ----------------
#[derive(Debug, Deserialize)]
pub struct CreateProposalRequest {
    pub title: String,
    pub description: String,
    pub creator_id: String,
}

#[derive(Debug, Serialize)]
pub struct CreateProposalResponse {
    pub proposal_id: String,
}

pub async fn create_proposal(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(_request): Json<CreateProposalRequest>,
) -> Result<Json<CreateProposalResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    let proposal_id = p_project_core::utils::generate_id();
    Ok(Json(CreateProposalResponse { proposal_id }))
}

#[derive(Debug, Deserialize)]
pub struct VoteProposalRequest {
    pub proposal_id: String,
    pub user_id: String,
    pub approve: bool,
}

#[derive(Debug, Serialize)]
pub struct VoteProposalResponse {
    pub message: String,
}

pub async fn vote_on_proposal(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(_request): Json<VoteProposalRequest>,
) -> Result<Json<VoteProposalResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    Ok(Json(VoteProposalResponse {
        message: "Vote recorded successfully".to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct TallyVotesRequest {
    pub proposal_id: String,
}

#[derive(Debug, Serialize)]
pub struct TallyVotesResponse {
    pub status: String,
    pub message: String,
}

pub async fn tally_votes(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(_request): Json<TallyVotesRequest>,
) -> Result<Json<TallyVotesResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["governance", "admin"])?;
    Ok(Json(TallyVotesResponse {
        status: "success".to_string(),
        message: "Votes tallied successfully".to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct ExecuteProposalRequest {
    pub proposal_id: String,
}

#[derive(Debug, Serialize)]
pub struct ExecuteProposalResponse {
    pub message: String,
}

pub async fn execute_proposal(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(_request): Json<ExecuteProposalRequest>,
) -> Result<Json<ExecuteProposalResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["governance", "admin"])?;
    Ok(Json(ExecuteProposalResponse {
        message: "Proposal executed successfully".to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct DelegateVoteRequest {
    pub from_user_id: String,
    pub to_user_id: String,
}

#[derive(Debug, Serialize)]
pub struct DelegateVoteResponse {
    pub message: String,
}

pub async fn delegate_vote(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(_request): Json<DelegateVoteRequest>,
) -> Result<Json<DelegateVoteResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    Ok(Json(DelegateVoteResponse {
        message: "Vote delegation set successfully".to_string(),
    }))
}

#[derive(Debug, Serialize)]
pub struct ProposalResponse {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
}

pub async fn get_proposals(
    State(_state): State<AppState>,
) -> Result<Json<Vec<ProposalResponse>>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(vec![]))
}

// ---------------- AI Service handlers ----------------
#[derive(Debug, Deserialize)]
pub struct ImpactVerificationRequest {
    pub user_id: String,
    pub ngo_id: String,
    pub activity_description: String,
    pub evidence_urls: Vec<String>,
    pub impact_metrics: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Serialize)]
pub struct ImpactVerificationResponse {
    pub verification_id: String,
    pub confidence_score: f32,
    pub verified: bool,
    pub feedback: String,
    pub recommendations: Vec<String>,
}

pub async fn verify_impact(
    State(_state): State<AppState>,
    Json(req): Json<ImpactVerificationRequest>,
) -> Result<Json<ImpactVerificationResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize AI service from env
    let ai_config = env_ai_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let ai_service = AIService::new(ai_config);

    // Convert handler request to core request
    let core_request = p_project_core::ImpactVerificationRequest {
        user_id: req.user_id,
        ngo_id: req.ngo_id,
        activity_description: req.activity_description,
        evidence_urls: req.evidence_urls,
        impact_metrics: req.impact_metrics,
    };

    match ai_service.verify_impact(core_request).await {
        Ok(response) => {
            // Convert core response to handler response
            let handler_response = ImpactVerificationResponse {
                verification_id: response.verification_id,
                confidence_score: response.confidence_score,
                verified: response.verified,
                feedback: response.feedback,
                recommendations: response.recommendations,
            };
            Ok(Json(handler_response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct AINFTArtRequest {
    pub prompt: String,
    pub style: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize)]
pub struct AINFTArtResponse {
    pub image_data: String,
    pub metadata_uri: String,
    pub generation_time_ms: u64,
}

pub async fn generate_peace_nft_art(
    State(_state): State<AppState>,
    Json(req): Json<AINFTArtRequest>,
) -> Result<Json<AINFTArtResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize AI service from env
    let ai_config = env_ai_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let ai_service = AIService::new(ai_config);

    // Convert handler request to core request
    let core_request = p_project_core::AINFTArtRequest {
        prompt: req.prompt,
        style: req.style,
        width: req.width,
        height: req.height,
    };

    match ai_service.generate_peace_nft_art(core_request).await {
        Ok(response) => {
            // Convert core response to handler response
            let handler_response = AINFTArtResponse {
                image_data: response.image_data,
                metadata_uri: response.metadata_uri,
                generation_time_ms: response.generation_time_ms,
            };
            Ok(Json(handler_response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct FraudDetectionRequest {
    pub ngo_id: String,
    pub transaction_data: Vec<p_project_core::TransactionData>,
    pub historical_patterns: Vec<p_project_core::HistoricalPattern>,
}

#[derive(Debug, Serialize)]
pub struct FraudDetectionResponse {
    pub analysis_id: String,
    pub risk_score: f32,
    pub suspicious_activities: Vec<p_project_core::SuspiciousActivity>,
    pub recommendations: Vec<String>,
}

pub async fn detect_fraud(
    State(_state): State<AppState>,
    Json(req): Json<FraudDetectionRequest>,
) -> Result<Json<FraudDetectionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize AI service from env
    let ai_config = env_ai_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let ai_service = AIService::new(ai_config);

    // Convert handler request to core request
    let core_request = p_project_core::FraudDetectionRequest {
        ngo_id: req.ngo_id,
        transaction_data: req.transaction_data,
        historical_patterns: req.historical_patterns,
    };

    match ai_service.detect_fraud(core_request).await {
        Ok(response) => {
            // Convert core response to handler response
            let handler_response = FraudDetectionResponse {
                analysis_id: response.analysis_id,
                risk_score: response.risk_score,
                suspicious_activities: response.suspicious_activities,
                recommendations: response.recommendations,
            };
            Ok(Json(handler_response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

/// AI-generated meme request
#[derive(Debug, Deserialize)]
pub struct AIMemeRequest {
    pub prompt: String,
    pub style: String,
    pub width: u32,
    pub height: u32,
    pub template: Option<String>,
}

/// AI-generated meme response
#[derive(Debug, Serialize)]
pub struct AIMemeResponse {
    pub image_data: String,
    pub metadata_uri: String,
    pub generation_time_ms: u64,
    pub meme_text: String,
}

pub async fn generate_ai_meme(
    State(_state): State<AppState>,
    Json(req): Json<AIMemeRequest>,
) -> Result<Json<AIMemeResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize AI service from env
    let ai_config = env_ai_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let ai_service = AIService::new(ai_config);

    // Convert handler request to core request
    let core_request = p_project_core::AIMemeRequest {
        prompt: req.prompt,
        style: req.style,
        width: req.width,
        height: req.height,
        template: req.template,
    };

    match ai_service.generate_meme(core_request).await {
        Ok(response) => {
            // Convert core response to handler response
            let handler_response = AIMemeResponse {
                image_data: response.image_data,
                metadata_uri: response.metadata_uri,
                generation_time_ms: response.generation_time_ms,
                meme_text: response.meme_text,
            };
            Ok(Json(handler_response))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

// ---------------- IoT Service handlers ----------------
#[derive(Debug, Deserialize)]
pub struct RegisterDonationBoxRequest {
    pub box_id: String,
    pub location: String,
    pub wallet_address: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterDonationBoxResponse {
    pub box_data: p_project_core::SmartDonationBox,
}

pub async fn register_donation_box(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<RegisterDonationBoxRequest>,
) -> Result<Json<RegisterDonationBoxResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["admin"])?;
    // Initialize IoT service from env
    let iot_config = env_iot_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut iot_service = IoTService::new(iot_config);

    match iot_service.register_donation_box(req.box_id, req.location, req.wallet_address) {
        Ok(box_data) => Ok(Json(RegisterDonationBoxResponse { box_data })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct RecordDonationRequest {
    pub box_id: String,
    pub amount: f64,
    pub donor_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RecordDonationResponse {
    pub transaction: p_project_core::DonationBoxTransaction,
}

pub async fn record_donation(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<RecordDonationRequest>,
) -> Result<Json<RecordDonationResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    // Initialize IoT service from env
    let iot_config = env_iot_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut iot_service = IoTService::new(iot_config);

    match iot_service.record_donation(&req.box_id, req.amount, req.donor_address) {
        Ok(transaction) => Ok(Json(RecordDonationResponse { transaction })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct GetDonationBoxStatusRequest {
    pub box_id: String,
}

#[derive(Debug, Serialize)]
pub struct GetDonationBoxStatusResponse {
    pub box_data: Option<p_project_core::SmartDonationBox>,
}

pub async fn get_donation_box_status(
    State(_state): State<AppState>,
    Extension(_claims): Extension<crate::middleware::Claims>,
    Json(req): Json<GetDonationBoxStatusRequest>,
) -> Result<Json<GetDonationBoxStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize IoT service from env
    let iot_config = env_iot_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let iot_service = IoTService::new(iot_config);

    let box_data = iot_service.get_donation_box_status(&req.box_id).cloned();
    Ok(Json(GetDonationBoxStatusResponse { box_data }))
}

// New IoT handlers for NFC wristbands
#[derive(Debug, Deserialize)]
pub struct RegisterWristbandRequest {
    pub wristband_id: String,
    pub refugee_id: String,
    pub camp_id: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterWristbandResponse {
    pub wristband_data: p_project_core::NFCWristband,
}

pub async fn register_wristband(
    State(_state): State<AppState>,
    Json(req): Json<RegisterWristbandRequest>,
) -> Result<Json<RegisterWristbandResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize IoT service from env
    let iot_config = env_iot_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut iot_service = IoTService::new(iot_config);

    match iot_service.register_wristband(req.wristband_id, req.refugee_id, req.camp_id) {
        Ok(wristband_data) => Ok(Json(RegisterWristbandResponse { wristband_data })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct AddFundsToWristbandRequest {
    pub wristband_id: String,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct AddFundsToWristbandResponse {
    pub success: bool,
    pub message: String,
}

pub async fn add_funds_to_wristband(
    State(_state): State<AppState>,
    Json(req): Json<AddFundsToWristbandRequest>,
) -> Result<Json<AddFundsToWristbandResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize IoT service from env
    let iot_config = env_iot_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut iot_service = IoTService::new(iot_config);

    match iot_service.add_funds_to_wristband(&req.wristband_id, req.amount) {
        Ok(()) => Ok(Json(AddFundsToWristbandResponse {
            success: true,
            message: "Funds added successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ProcessWristbandTransactionRequest {
    pub wristband_id: String,
    pub amount: f64,
    pub transaction_type: String,
    pub vendor_id: String,
}

#[derive(Debug, Serialize)]
pub struct ProcessWristbandTransactionResponse {
    pub transaction: p_project_core::WristbandTransaction,
}

pub async fn process_wristband_transaction(
    State(_state): State<AppState>,
    Json(req): Json<ProcessWristbandTransactionRequest>,
) -> Result<Json<ProcessWristbandTransactionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize IoT service from env
    let iot_config = env_iot_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut iot_service = IoTService::new(iot_config);

    match iot_service.process_wristband_transaction(
        &req.wristband_id,
        req.amount,
        req.transaction_type,
        req.vendor_id,
    ) {
        Ok(transaction) => Ok(Json(ProcessWristbandTransactionResponse { transaction })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct GetWristbandStatusRequest {
    pub wristband_id: String,
}

#[derive(Debug, Serialize)]
pub struct GetWristbandStatusResponse {
    pub wristband_data: Option<p_project_core::NFCWristband>,
}

pub async fn get_wristband_status(
    State(_state): State<AppState>,
    Extension(_claims): Extension<crate::middleware::Claims>,
    Json(req): Json<GetWristbandStatusRequest>,
) -> Result<Json<GetWristbandStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize IoT service from env
    let iot_config = env_iot_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let iot_service = IoTService::new(iot_config);

    let wristband_data = iot_service.get_wristband_status(&req.wristband_id).cloned();
    Ok(Json(GetWristbandStatusResponse { wristband_data }))
}

// New IoT handlers for QR-code food distribution
#[derive(Debug, Deserialize)]
pub struct CreateFoodQRRequest {
    pub distribution_point: String,
    pub food_type: String,
    pub quantity: u32,
    pub expiration_date: String, // ISO 8601 format
}

#[derive(Debug, Serialize)]
pub struct CreateFoodQRResponse {
    pub qr_data: p_project_core::FoodDistributionQR,
}

pub async fn create_food_qr(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<CreateFoodQRRequest>,
) -> Result<Json<CreateFoodQRResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["admin"])?;
    // Initialize IoT service from env
    let iot_config = env_iot_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut iot_service = IoTService::new(iot_config);

    // Parse the expiration date
    let expiration_date =
        match chrono::NaiveDateTime::parse_from_str(&req.expiration_date, "%Y-%m-%dT%H:%M:%S") {
            Ok(date) => date,
            Err(_) => match chrono::NaiveDateTime::parse_from_str(
                &req.expiration_date,
                "%Y-%m-%d %H:%M:%S",
            ) {
                Ok(date) => date,
                Err(e) => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: format!("Invalid date format: {}", e),
                        }),
                    ));
                }
            },
        };

    match iot_service.create_food_qr(
        req.distribution_point,
        req.food_type,
        req.quantity,
        expiration_date,
    ) {
        Ok(qr_data) => Ok(Json(CreateFoodQRResponse { qr_data })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ClaimFoodQRRequest {
    pub qr_id: String,
    pub recipient_id: String,
    pub recipient_nfc_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ClaimFoodQRResponse {
    pub response: p_project_core::QRClaimResponse,
}

pub async fn claim_food_qr(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<ClaimFoodQRRequest>,
) -> Result<Json<ClaimFoodQRResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    // Initialize IoT service from env
    let iot_config = env_iot_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut iot_service = IoTService::new(iot_config);

    let claim_request = p_project_core::QRClaimRequest {
        qr_id: req.qr_id,
        recipient_id: req.recipient_id,
        recipient_nfc_id: req.recipient_nfc_id,
    };

    match iot_service.claim_food_qr(claim_request) {
        Ok(response) => Ok(Json(ClaimFoodQRResponse { response })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct GetQRStatusRequest {
    pub qr_id: String,
}

#[derive(Debug, Serialize)]
pub struct GetQRStatusResponse {
    pub qr_data: Option<p_project_core::FoodDistributionQR>,
}

pub async fn get_qr_status(
    State(_state): State<AppState>,
    Json(req): Json<GetQRStatusRequest>,
) -> Result<Json<GetQRStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize IoT service from env
    let iot_config = env_iot_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let iot_service = IoTService::new(iot_config);

    let qr_data = iot_service.get_qr_status(&req.qr_id).cloned();
    Ok(Json(GetQRStatusResponse { qr_data }))
}

// ---------------- Web2 Service handlers ----------------
#[derive(Debug, Deserialize)]
pub struct CreateDonationWidgetRequest {
    pub config: p_project_core::SocialMediaWidgetConfig,
}

#[derive(Debug, Serialize)]
pub struct CreateDonationWidgetResponse {
    pub widget_data: p_project_core::DonationWidgetData,
}

pub async fn create_donation_widget(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<CreateDonationWidgetRequest>,
) -> Result<Json<CreateDonationWidgetResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["admin"])?;
    // Initialize Web2 service from env
    let web2_config = env_web2_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut web2_service = Web2Service::new(web2_config);

    match web2_service.create_donation_widget(req.config) {
        Ok(widget_data) => Ok(Json(CreateDonationWidgetResponse { widget_data })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ProcessSocialMediaDonationRequest {
    pub donation_data: p_project_core::SocialMediaDonationRequest,
}

#[derive(Debug, Serialize)]
pub struct ProcessSocialMediaDonationResponse {
    pub donation_response: p_project_core::DonationResponse,
}

pub async fn process_social_media_donation(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<ProcessSocialMediaDonationRequest>,
) -> Result<Json<ProcessSocialMediaDonationResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    // Initialize Web2 service from env
    let web2_config = env_web2_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let web2_service = Web2Service::new(web2_config);

    match web2_service
        .process_social_media_donation(req.donation_data)
        .await
    {
        Ok(donation_response) => Ok(Json(ProcessSocialMediaDonationResponse {
            donation_response,
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct GenerateWidgetHtmlRequest {
    pub widget_id: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateWidgetHtmlResponse {
    pub html: String,
}

pub async fn generate_widget_html(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<GenerateWidgetHtmlRequest>,
) -> Result<Json<GenerateWidgetHtmlResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    // Initialize Web2 service from env
    let web2_config = env_web2_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let web2_service = Web2Service::new(web2_config);

    match web2_service.generate_widget_html(&req.widget_id) {
        Ok(html) => Ok(Json(GenerateWidgetHtmlResponse { html })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

// New Web2 handlers for YouTube tips
#[derive(Debug, Deserialize)]
pub struct CreateYouTubeTipConfigRequest {
    pub config: p_project_core::YouTubeTipConfig,
}

#[derive(Debug, Serialize)]
pub struct CreateYouTubeTipConfigResponse {
    pub config_id: String,
    pub success: bool,
    pub message: String,
}

pub async fn create_youtube_tip_config(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<CreateYouTubeTipConfigRequest>,
) -> Result<Json<CreateYouTubeTipConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["admin"])?;
    // Initialize Web2 service from env
    let web2_config = env_web2_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut web2_service = Web2Service::new(web2_config);

    match web2_service.create_youtube_tip_config(req.config) {
        Ok(config_id) => Ok(Json(CreateYouTubeTipConfigResponse {
            config_id,
            success: true,
            message: "YouTube tip configuration created successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ProcessYouTubeTipRequest {
    pub tip_data: p_project_core::YouTubeTipRequest,
}

#[derive(Debug, Serialize)]
pub struct ProcessYouTubeTipResponse {
    pub donation_response: p_project_core::DonationResponse,
}

pub async fn process_youtube_tip(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<ProcessYouTubeTipRequest>,
) -> Result<Json<ProcessYouTubeTipResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    // Initialize Web2 service from env
    let web2_config = env_web2_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let web2_service = Web2Service::new(web2_config);

    match web2_service.process_youtube_tip(req.tip_data).await {
        Ok(donation_response) => Ok(Json(ProcessYouTubeTipResponse { donation_response })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

// New Web2 handlers for messaging bots
#[derive(Debug, Deserialize)]
pub struct RegisterMessagingBotRequest {
    pub config: p_project_core::MessagingBotConfig,
}

#[derive(Debug, Serialize)]
pub struct RegisterMessagingBotResponse {
    pub bot_id: String,
    pub success: bool,
    pub message: String,
}

pub async fn register_messaging_bot(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<RegisterMessagingBotRequest>,
) -> Result<Json<RegisterMessagingBotResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["admin"])?;
    // Initialize Web2 service from env
    let web2_config = env_web2_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut web2_service = Web2Service::new(web2_config);

    match web2_service.register_messaging_bot(req.config) {
        Ok(bot_id) => Ok(Json(RegisterMessagingBotResponse {
            bot_id,
            success: true,
            message: "Messaging bot registered successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ProcessBotCommandRequest {
    pub command_data: p_project_core::BotCommandRequest,
}

#[derive(Debug, Serialize)]
pub struct ProcessBotCommandResponse {
    pub command_response: p_project_core::BotCommandResponse,
}

pub async fn process_bot_command(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<ProcessBotCommandRequest>,
) -> Result<Json<ProcessBotCommandResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    // Initialize Web2 service from env
    let web2_config = env_web2_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let web2_service = Web2Service::new(web2_config);

    match web2_service.process_bot_command(req.command_data).await {
        Ok(command_response) => Ok(Json(ProcessBotCommandResponse { command_response })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterNgoRequest {
    pub config: p_project_core::NGORegistration,
}

#[derive(Debug, Serialize)]
pub struct RegisterNgoResponse {
    pub ngo: p_project_core::NGOProfile,
}

#[derive(Debug, Deserialize)]
pub struct AddImpactEventRequest {
    pub user_id: String,
    pub event: p_project_core::SocialImpactEvent,
}

#[derive(Debug, Serialize)]
pub struct AddImpactEventResponse {
    pub credit_score: f64,
}

#[derive(Debug, Deserialize)]
pub struct RequestMicroLoanRequest {
    pub borrower_id: String,
    pub amount: f64,
    pub collateral_amount: f64,
    pub ngo_id: String,
}

#[derive(Debug, Serialize)]
pub struct RequestMicroLoanResponse {
    pub loan: p_project_core::MicroLoan,
    pub credit_score: f64,
}

#[derive(Debug, Deserialize)]
pub struct RepayMicroLoanRequest {
    pub loan_id: String,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct RepayMicroLoanResponse {
    pub loan: p_project_core::MicroLoan,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct GetMicroLoanResponse {
    pub loan: p_project_core::MicroLoan,
}

#[derive(Debug, Serialize)]
pub struct CreditScoreResponse {
    pub user_id: String,
    pub credit_score: f64,
}

#[derive(Debug, Serialize)]
pub struct TokenomicsSummaryResponse {
    pub summary: p_project_core::TokenomicsSummary,
}

#[derive(Debug, Serialize)]
pub struct ExchangeListingsResponse {
    pub strategy: p_project_core::ExchangeListings,
}

#[derive(Debug, Deserialize)]
pub struct RegisterMissionRequest {
    pub mission: p_project_core::PeacefulMission,
}

#[derive(Debug, Serialize)]
pub struct RegisterMissionResponse {
    pub mission: p_project_core::PeacefulMission,
}

#[derive(Debug, Deserialize)]
pub struct CompleteMissionRequest {
    pub player_id: String,
    pub mission_id: String,
}

#[derive(Debug, Serialize)]
pub struct CompleteMissionResponse {
    pub receipt: p_project_core::RewardReceipt,
}

#[derive(Debug, Deserialize)]
pub struct RecordBehaviorRequest {
    pub player_id: String,
    pub behavior: p_project_core::PositiveBehavior,
}

#[derive(Debug, Serialize)]
pub struct RecordBehaviorResponse {
    pub receipt: p_project_core::RewardReceipt,
}

#[derive(Debug, Serialize)]
pub struct PlayerBalanceResponse {
    pub player_id: String,
    pub balance: f64,
}

pub async fn register_ngo(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<RegisterNgoRequest>,
) -> Result<Json<RegisterNgoResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["admin"])?;
    let config = env_credit_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut service = CreditService::new(config);

    let ngo = service.register_ngo(req.config);
    Ok(Json(RegisterNgoResponse { ngo }))
}

pub async fn add_social_impact_event(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<AddImpactEventRequest>,
) -> Result<Json<AddImpactEventResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    let config = env_credit_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut service = CreditService::new(config);

    match service.add_social_impact_event(&req.user_id, req.event) {
        Ok(score) => Ok(Json(AddImpactEventResponse {
            credit_score: score,
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn request_micro_loan(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<RequestMicroLoanRequest>,
) -> Result<Json<RequestMicroLoanResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    let config = env_credit_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut service = CreditService::new(config);

    match service.request_micro_loan(
        &req.borrower_id,
        req.amount,
        req.collateral_amount,
        &req.ngo_id,
    ) {
        Ok(loan) => {
            let score = service.get_credit_score(&req.borrower_id);
            Ok(Json(RequestMicroLoanResponse {
                loan,
                credit_score: score,
            }))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn repay_micro_loan(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<RepayMicroLoanRequest>,
) -> Result<Json<RepayMicroLoanResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    let config = env_credit_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut service = CreditService::new(config);

    match service.repay_micro_loan(&req.loan_id, req.amount) {
        Ok(loan) => Ok(Json(RepayMicroLoanResponse {
            loan: loan.clone(),
            status: format!("{:?}", loan.status),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn get_micro_loan(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Path(loan_id): Path<String>,
) -> Result<Json<GetMicroLoanResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    let config = env_credit_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let service = CreditService::new(config);

    match service.get_micro_loan(&loan_id) {
        Some(loan) => Ok(Json(GetMicroLoanResponse { loan: loan.clone() })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "loan_not_found".to_string(),
            }),
        )),
    }
}

pub async fn get_credit_score(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Path(user_id): Path<String>,
) -> Result<Json<CreditScoreResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    let config = env_credit_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut service = CreditService::new(config);

    let score = service.get_credit_score(&user_id);
    Ok(Json(CreditScoreResponse {
        user_id,
        credit_score: score,
    }))
}

pub async fn get_tokenomics_summary(
    State(_state): State<AppState>,
) -> Result<Json<TokenomicsSummaryResponse>, (StatusCode, Json<ErrorResponse>)> {
    let path = env_tokenomics_path().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    match TokenomicsService::from_path(path) {
        Ok(service) => Ok(Json(TokenomicsSummaryResponse {
            summary: service.summary().clone(),
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn get_exchange_listings_strategy(
    State(_state): State<AppState>,
) -> Result<Json<ExchangeListingsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let path = env_tokenomics_path().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    match TokenomicsService::from_path(path) {
        Ok(service) => {
            let strategy = p_project_core::ExchangeListings::from_tokenomics_service(&service);
            Ok(Json(ExchangeListingsResponse { strategy }))
        }
        Err(_e) => {
            // Fall back to defaults if CSV missing or malformed
            let strategy = p_project_core::ExchangeListings::default();
            Ok(Json(ExchangeListingsResponse { strategy }))
        }
    }
}

pub async fn register_mission(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<RegisterMissionRequest>,
) -> Result<Json<RegisterMissionResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["admin"])?;
    let config = env_game_currency_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut service = GameCurrencyService::new(config);

    match service.register_mission(req.mission) {
        Ok(mission) => Ok(Json(RegisterMissionResponse { mission })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn complete_mission(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<CompleteMissionRequest>,
) -> Result<Json<CompleteMissionResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    let config = env_game_currency_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut service = GameCurrencyService::new(config);

    match service.complete_mission(&req.player_id, &req.mission_id) {
        Ok(receipt) => Ok(Json(CompleteMissionResponse { receipt })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn record_behavior(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Json(req): Json<RecordBehaviorRequest>,
) -> Result<Json<RecordBehaviorResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    let config = env_game_currency_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut service = GameCurrencyService::new(config);

    let receipt = service.record_behavior(&req.player_id, req.behavior);
    Ok(Json(RecordBehaviorResponse { receipt }))
}

pub async fn get_player_balance(
    State(_state): State<AppState>,
    Extension(claims): Extension<crate::middleware::Claims>,
    Path(player_id): Path<String>,
) -> Result<Json<PlayerBalanceResponse>, (StatusCode, Json<ErrorResponse>)> {
    ensure_roles(&claims, &["user", "admin"])?;
    let config = env_game_currency_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let service = GameCurrencyService::new(config);

    let balance = service.get_balance(&player_id);
    Ok(Json(PlayerBalanceResponse { player_id, balance }))
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// --- Lightweight JWT helper for handlers that aren't on the auth-protected router ---
fn parse_jwt_from_headers(
    headers: &HeaderMap,
) -> Result<crate::middleware::Claims, (StatusCode, Json<ErrorResponse>)> {
    let secret = std::env::var("JWT_SECRET").map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
            }),
        )
    })?;

    let auth = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let token = auth.strip_prefix("Bearer ").unwrap_or("").trim();
    if token.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
            }),
        ));
    }

    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    let key = jsonwebtoken::DecodingKey::from_secret(secret.as_bytes());
    let data = jsonwebtoken::decode::<crate::middleware::Claims>(token, &key, &validation)
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "unauthorized".to_string(),
                }),
            )
        })?;
    Ok(data.claims)
}

#[derive(Debug, Serialize)]
pub struct MerchantPaymentResponse {
    pub transaction_id: String,
    pub merchant_id: String,
    pub customer_wallet: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub timestamp: NaiveDateTime,
    pub tx_hash: Option<String>,
    pub qr_code_used: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MerchantPaymentRequest {
    pub merchant_id: String,
    pub customer_wallet: String,
    pub amount: f64,
    pub currency: String,
    pub description: Option<String>,
    pub qr_code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateMerchantResponse {
    pub merchant_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMerchantRequest {
    pub name: String,
    pub category: String, // "coffee_shop", "restaurant", "bookstore", "clinic", "repair_shop"
    pub wallet_address: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub contact_info: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateQRResponse {
    pub qr_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateQRRequest {
    pub merchant_id: String,
    pub amount: f64,
    pub currency: String,
    pub description: Option<String>,
    pub expires_in_seconds: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GetMerchantResponse {
    pub id: String,
    pub name: String,
    pub category: String,
    pub wallet_address: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub contact_info: Option<String>,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
    pub verified_at: Option<NaiveDateTime>,
}

pub async fn process_merchant_payment(
    State(state): State<AppState>,
    Json(req): Json<MerchantPaymentRequest>,
) -> Result<Json<MerchantPaymentResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize merchant service
    let merchant_config = p_project_core::merchant_service::MerchantServiceConfig {
        fee_percentage: 0.01,            // 1% platform fee
        max_transaction_amount: 10000.0, // Max 10,000 P-Coin per transaction
    };
    let mut merchant_service =
        p_project_core::merchant_service::MerchantService::new(merchant_config);

    // In a real implementation, we would load existing merchants from the database
    // For this example, we'll just process the payment directly

    let core_request = p_project_core::merchant_service::MerchantPaymentRequest {
        merchant_id: req.merchant_id,
        customer_wallet: req.customer_wallet,
        amount: req.amount,
        currency: req.currency,
        description: req.description,
        qr_code: req.qr_code,
    };

    match merchant_service.process_payment(core_request).await {
        Ok(response) => {
            let handler_response = MerchantPaymentResponse {
                transaction_id: response.transaction_id,
                merchant_id: response.merchant_id,
                customer_wallet: response.customer_wallet,
                amount: response.amount,
                currency: response.currency,
                status: response.status,
                timestamp: response.timestamp,
                tx_hash: response.tx_hash,
                qr_code_used: response.qr_code_used,
            };
            Ok(Json(handler_response))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn create_merchant(
    State(_state): State<AppState>,
    Json(req): Json<CreateMerchantRequest>,
) -> Result<Json<CreateMerchantResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize merchant service
    let merchant_config = p_project_core::merchant_service::MerchantServiceConfig {
        fee_percentage: 0.01,
        max_transaction_amount: 10000.0,
    };
    let mut merchant_service =
        p_project_core::merchant_service::MerchantService::new(merchant_config);

    // Convert category string to enum
    let category = match req.category.as_str() {
        "coffee_shop" => p_project_core::merchant_service::MerchantCategory::CoffeeShop,
        "restaurant" => p_project_core::merchant_service::MerchantCategory::Restaurant,
        "bookstore" => p_project_core::merchant_service::MerchantCategory::Bookstore,
        "clinic" => p_project_core::merchant_service::MerchantCategory::Clinic,
        "repair_shop" => p_project_core::merchant_service::MerchantCategory::RepairShop,
        _ => p_project_core::merchant_service::MerchantCategory::Other(req.category.clone()),
    };

    match merchant_service.register_merchant(
        req.name,
        category,
        req.wallet_address,
        req.description,
        req.location,
        req.contact_info,
    ) {
        Ok(merchant_id) => {
            let response = CreateMerchantResponse { merchant_id };
            Ok(Json(response))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn create_payment_qr(
    State(_state): State<AppState>,
    Json(req): Json<CreateQRRequest>,
) -> Result<Json<CreateQRResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize merchant service
    let merchant_config = p_project_core::merchant_service::MerchantServiceConfig {
        fee_percentage: 0.01,
        max_transaction_amount: 10000.0,
    };
    let mut merchant_service =
        p_project_core::merchant_service::MerchantService::new(merchant_config);

    match merchant_service.create_payment_qr(
        req.merchant_id,
        req.amount,
        req.currency,
        req.description,
        req.expires_in_seconds,
    ) {
        Ok(qr_id) => {
            let response = CreateQRResponse { qr_id };
            Ok(Json(response))
        }
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub async fn get_merchant(
    State(_state): State<AppState>,
    Path(merchant_id): Path<String>,
) -> Result<Json<GetMerchantResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize merchant service
    let merchant_config = p_project_core::merchant_service::MerchantServiceConfig {
        fee_percentage: 0.01,
        max_transaction_amount: 10000.0,
    };
    let merchant_service = p_project_core::merchant_service::MerchantService::new(merchant_config);

    // In a real implementation, we would fetch the merchant from the database
    // For this example, we'll return a mock response

    // This is a simplified implementation - in reality, you would look up the merchant
    let response = GetMerchantResponse {
        id: merchant_id,
        name: "Sample Merchant".to_string(),
        category: "coffee_shop".to_string(),
        wallet_address: "0x123456789abcdef".to_string(),
        description: Some("A sample merchant for testing".to_string()),
        location: Some("123 Main St".to_string()),
        contact_info: Some("contact@samplemerchant.com".to_string()),
        is_verified: true,
        created_at: Utc::now().naive_utc(),
        verified_at: Some(Utc::now().naive_utc()),
    };

    Ok(Json(response))
}

// ---------------- Digital goods (merchant) ----------------
#[derive(Debug, Deserialize)]
pub struct AddDigitalGoodsProductRequest {
    pub merchant_id: String,
    pub product: p_project_core::merchant_service::DigitalGoodsProduct,
}

#[derive(Debug, Serialize)]
pub struct AddDigitalGoodsProductResponse {
    pub success: bool,
}

pub async fn add_digital_goods_product(
    headers: axum::http::HeaderMap,
    Json(req): Json<AddDigitalGoodsProductRequest>,
) -> Result<Json<AddDigitalGoodsProductResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Require JWT (admin typically)
    let _claims = parse_jwt_from_headers(&headers)?;

    let merchant_config = p_project_core::merchant_service::MerchantServiceConfig {
        fee_percentage: 0.01,
        max_transaction_amount: 10000.0,
    };
    let mut merchant_service =
        p_project_core::merchant_service::MerchantService::new(merchant_config);

    // For demo/tests, attempt to add; will error if merchant not present
    match merchant_service.add_digital_goods_product(&req.merchant_id, req.product) {
        Ok(()) => Ok(Json(AddDigitalGoodsProductResponse { success: true })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct PurchaseDigitalGoodsRequest {
    pub product_id: String,
    pub customer_id: String,
    pub customer_country: String,
    pub customer_language: String,
}

#[derive(Debug, Serialize)]
pub struct PurchaseDigitalGoodsResponse {
    pub transaction: p_project_core::merchant_service::DigitalGoodsTransaction,
}

pub async fn purchase_digital_goods(
    headers: axum::http::HeaderMap,
    Json(req): Json<PurchaseDigitalGoodsRequest>,
) -> Result<Json<PurchaseDigitalGoodsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Require JWT (user)
    let _claims = parse_jwt_from_headers(&headers)?;

    let merchant_config = p_project_core::merchant_service::MerchantServiceConfig {
        fee_percentage: 0.01,
        max_transaction_amount: 10000.0,
    };
    let mut merchant_service =
        p_project_core::merchant_service::MerchantService::new(merchant_config);

    match merchant_service
        .purchase_digital_good(
            req.product_id,
            req.customer_id,
            req.customer_country,
            req.customer_language,
        )
        .await
    {
        Ok(tx) => Ok(Json(PurchaseDigitalGoodsResponse { transaction: tx })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct GetDigitalGoodsTransactionRequest {
    pub transaction_id: String,
}

#[derive(Debug, Serialize)]
pub struct GetDigitalGoodsTransactionResponse {
    pub transaction: Option<p_project_core::merchant_service::DigitalGoodsTransaction>,
}

pub async fn get_digital_goods_transaction(
    headers: axum::http::HeaderMap,
    Json(req): Json<GetDigitalGoodsTransactionRequest>,
) -> Result<Json<GetDigitalGoodsTransactionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Require JWT (user)
    let _claims = parse_jwt_from_headers(&headers)?;

    let merchant_config = p_project_core::merchant_service::MerchantServiceConfig {
        fee_percentage: 0.01,
        max_transaction_amount: 10000.0,
    };
    let merchant_service = p_project_core::merchant_service::MerchantService::new(merchant_config);

    let tx = merchant_service.get_digital_goods_transaction(&req.transaction_id);
    Ok(Json(GetDigitalGoodsTransactionResponse {
        transaction: tx.cloned(),
    }))
}

// ---------------- E-commerce (Shopify/WooCommerce) ----------------
#[derive(Debug, Deserialize)]
pub struct CreateEcommerceIntegrationRequest {
    pub config: p_project_core::web2_service::EcommerceConfig,
}

#[derive(Debug, Serialize)]
pub struct CreateEcommerceIntegrationResponse {
    pub integration_id: String,
}

pub async fn create_ecommerce_integration(
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateEcommerceIntegrationRequest>,
) -> Result<Json<CreateEcommerceIntegrationResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Require JWT (admin)
    let _claims = parse_jwt_from_headers(&headers)?;

    let web2_config = env_web2_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let mut web2 = Web2Service::new(web2_config);
    match web2.create_ecommerce_integration(req.config) {
        Ok(integration_id) => Ok(Json(CreateEcommerceIntegrationResponse { integration_id })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ProcessEcommercePaymentRequest {
    pub payment_data: p_project_core::web2_service::EcommercePaymentRequest,
}

#[derive(Debug, Serialize)]
pub struct ProcessEcommercePaymentResponse {
    pub response: p_project_core::web2_service::EcommercePaymentResponse,
}

pub async fn process_ecommerce_payment(
    headers: axum::http::HeaderMap,
    Json(req): Json<ProcessEcommercePaymentRequest>,
) -> Result<Json<ProcessEcommercePaymentResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Require JWT (user)
    let _claims = parse_jwt_from_headers(&headers)?;

    let web2_config = env_web2_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let web2 = Web2Service::new(web2_config);
    match web2.process_ecommerce_payment(req.payment_data).await {
        Ok(resp) => Ok(Json(ProcessEcommercePaymentResponse { response: resp })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct VerifyWebhookSignatureRequest {
    pub platform: String, // "shopify" | "woocommerce"
    pub body: String,
    pub signature: String,
    pub secret: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyWebhookSignatureResponse {
    pub valid: bool,
}

pub async fn verify_webhook_signature(
    Json(req): Json<VerifyWebhookSignatureRequest>,
) -> Result<Json<VerifyWebhookSignatureResponse>, (StatusCode, Json<ErrorResponse>)> {
    let web2_config = env_web2_config().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let web2 = Web2Service::new(web2_config);
    let body_bytes = req.body.as_bytes();

    let valid = match req.platform.as_str() {
        "shopify" => web2.verify_shopify_webhook(body_bytes, &req.signature, &req.secret),
        "woocommerce" => web2.verify_woocommerce_webhook(body_bytes, &req.signature, &req.secret),
        _ => false,
    };

    Ok(Json(VerifyWebhookSignatureResponse { valid }))
}

// ---------------- Cashback & Loyalty ----------------
#[derive(Debug, Deserialize)]
pub struct ConfigureCashbackRequest {
    pub merchant_id: String,
    pub cashback_percentage: f64,
    pub min_purchase_amount: f64,
    pub max_cashback_amount: f64,
    pub loyalty_points_per_coin: f64,
}

#[derive(Debug, Serialize)]
pub struct ConfigureCashbackResponse {
    pub success: bool,
}

pub async fn configure_cashback(
    headers: axum::http::HeaderMap,
    Json(req): Json<ConfigureCashbackRequest>,
) -> Result<Json<ConfigureCashbackResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Require JWT (admin)
    let _claims = parse_jwt_from_headers(&headers)?;

    let merchant_config = p_project_core::merchant_service::MerchantServiceConfig {
        fee_percentage: 0.01,
        max_transaction_amount: 10000.0,
    };
    let mut svc = p_project_core::merchant_service::MerchantService::new(merchant_config);
    match svc.configure_cashback(
        req.merchant_id,
        req.cashback_percentage,
        req.min_purchase_amount,
        req.max_cashback_amount,
        req.loyalty_points_per_coin,
    ) {
        Ok(()) => Ok(Json(ConfigureCashbackResponse { success: true })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ProcessPurchaseWithCashbackRequest {
    pub merchant_id: String,
    pub customer_id: String,
    pub purchase_amount: f64,
    pub customer_wallet: String,
}

#[derive(Debug, Serialize)]
pub struct ProcessPurchaseWithCashbackResponse {
    pub transaction: p_project_core::merchant_service::CashbackTransaction,
}

pub async fn process_purchase_with_cashback(
    headers: axum::http::HeaderMap,
    Json(req): Json<ProcessPurchaseWithCashbackRequest>,
) -> Result<Json<ProcessPurchaseWithCashbackResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Require JWT (user)
    let _claims = parse_jwt_from_headers(&headers)?;

    let merchant_config = p_project_core::merchant_service::MerchantServiceConfig {
        fee_percentage: 0.01,
        max_transaction_amount: 10000.0,
    };
    let mut svc = p_project_core::merchant_service::MerchantService::new(merchant_config);
    match svc
        .process_purchase_with_cashback(
            req.merchant_id,
            req.customer_id,
            req.purchase_amount,
            req.customer_wallet,
        )
        .await
    {
        Ok(tx) => Ok(Json(ProcessPurchaseWithCashbackResponse {
            transaction: tx,
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct GetCustomerLoyaltyPointsRequest {
    pub customer_id: String,
    pub merchant_id: String,
}

#[derive(Debug, Serialize)]
pub struct GetCustomerLoyaltyPointsResponse {
    pub customer_id: String,
    pub merchant_id: String,
    pub points: f64,
    pub total_earned: f64,
    pub total_spent: f64,
}

pub async fn get_customer_loyalty_points(
    Json(req): Json<GetCustomerLoyaltyPointsRequest>,
) -> Result<Json<GetCustomerLoyaltyPointsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Public info endpoint
    let merchant_config = p_project_core::merchant_service::MerchantServiceConfig {
        fee_percentage: 0.01,
        max_transaction_amount: 10000.0,
    };
    let svc = p_project_core::merchant_service::MerchantService::new(merchant_config);
    let points = svc.get_customer_loyalty_points(&req.customer_id, &req.merchant_id);
    let (p, earned, spent) = match points {
        Some(lp) => (lp.points, lp.total_earned, lp.total_spent),
        None => (0.0, 0.0, 0.0),
    };
    Ok(Json(GetCustomerLoyaltyPointsResponse {
        customer_id: req.customer_id,
        merchant_id: req.merchant_id,
        points: p,
        total_earned: earned,
        total_spent: spent,
    }))
}

#[derive(Debug, Deserialize)]
pub struct RedeemLoyaltyPointsRequest {
    pub customer_id: String,
    pub merchant_id: String,
    pub points_to_redeem: f64,
}

#[derive(Debug, Serialize)]
pub struct RedeemLoyaltyPointsResponse {
    pub redeemed_value: f64,
}

pub async fn redeem_loyalty_points(
    headers: axum::http::HeaderMap,
    Json(req): Json<RedeemLoyaltyPointsRequest>,
) -> Result<Json<RedeemLoyaltyPointsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Require JWT (user)
    let _claims = parse_jwt_from_headers(&headers)?;

    let merchant_config = p_project_core::merchant_service::MerchantServiceConfig {
        fee_percentage: 0.01,
        max_transaction_amount: 10000.0,
    };
    let mut svc = p_project_core::merchant_service::MerchantService::new(merchant_config);
    match svc.redeem_loyalty_points(&req.customer_id, &req.merchant_id, req.points_to_redeem) {
        Ok(value) => Ok(Json(RedeemLoyaltyPointsResponse {
            redeemed_value: value,
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct GetCashbackTransactionRequest {
    pub transaction_id: String,
}

#[derive(Debug, Serialize)]
pub struct GetCashbackTransactionResponse {
    pub transaction: Option<p_project_core::merchant_service::CashbackTransaction>,
}

pub async fn get_cashback_transaction(
    Json(req): Json<GetCashbackTransactionRequest>,
) -> Result<Json<GetCashbackTransactionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Public lookup
    let merchant_config = p_project_core::merchant_service::MerchantServiceConfig {
        fee_percentage: 0.01,
        max_transaction_amount: 10000.0,
    };
    let svc = p_project_core::merchant_service::MerchantService::new(merchant_config);
    let tx = svc.get_cashback_transaction(&req.transaction_id);
    Ok(Json(GetCashbackTransactionResponse {
        transaction: tx.cloned(),
    }))
}

// ---------------- Webhooks for bots ----------------
#[derive(Debug, Deserialize)]
pub struct TelegramWebhookRequest {
    pub update_id: Option<i64>,
    #[serde(default)]
    pub message: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct WebhookAckResponse {
    pub ok: bool,
    pub received: bool,
}

pub async fn telegram_webhook(
    headers: HeaderMap,
    Query(q): Option<Query<HashMap<String, String>>>,
    Json(_req): Json<TelegramWebhookRequest>,
) -> Result<Json<WebhookAckResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !webhook_token_valid(&headers, q.as_ref().map(|x| &x.0), "TELEGRAM_WEBHOOK_TOKEN") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
            }),
        ));
    }
    Ok(Json(WebhookAckResponse { ok: true, received: true }))
}

#[derive(Debug, Deserialize)]
pub struct DiscordWebhookRequest {
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

pub async fn discord_webhook(
    headers: HeaderMap,
    Query(q): Option<Query<HashMap<String, String>>>,
    Json(_req): Json<DiscordWebhookRequest>,
) -> Result<Json<WebhookAckResponse>, (StatusCode, Json<ErrorResponse>)> {
    if !webhook_token_valid(&headers, q.as_ref().map(|x| &x.0), "DISCORD_WEBHOOK_TOKEN") {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
            }),
        ));
    }
    Ok(Json(WebhookAckResponse { ok: true, received: true }))
}

// ---------------- Validation & Blocklists ----------------
fn is_valid_username(s: &str) -> bool {
    let len = s.len();
    if len < 3 || len > 32 { return false; }
    s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' ))
}

fn is_valid_wallet(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower.len() == 42 && lower.starts_with("0x") && lower[2..].chars().all(|c| c.is_ascii_hexdigit())
}

fn parse_list_env(key: &str) -> Vec<String> {
    env::var(key)
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn is_wallet_blocked(addr: &str) -> bool {
    let list = parse_list_env("WALLET_BLOCKLIST");
    let a = addr.to_ascii_lowercase();
    list.iter().any(|x| x.to_ascii_lowercase() == a)
}

fn is_username_blocked(name: &str) -> bool {
    let list = parse_list_env("USERNAME_BLOCKLIST");
    let n = name.to_ascii_lowercase();
    list.iter().any(|x| x.to_ascii_lowercase() == n)
}

fn webhook_token_valid(
    headers: &HeaderMap,
    query: Option<&HashMap<String, String>>,
    specific_env_key: &str,
) -> bool {
    let hdr = headers
        .get("x-webhook-token")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let qry = query
        .and_then(|m| m.get("token").cloned());
    let provided = hdr.or(qry);
    let specific = env::var(specific_env_key).ok();
    let generic = env::var("WEBHOOK_TOKEN").ok();

    match (provided, specific, generic) {
        (Some(p), Some(s), _) => constant_time_eq(&p, &s),
        (Some(p), None, Some(g)) => constant_time_eq(&p, &g),
        // If no secret configured, deny by default
        _ => false,
    }
}

fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() { return false; }
    let mut acc = 0u8;
    for (x, y) in a.as_bytes().iter().zip(b.as_bytes()) { acc |= x ^ y; }
    acc == 0
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn username_validation() {
        assert!(super::is_valid_username("user_123"));
        assert!(!super::is_valid_username("ab")); // too short
        assert!(!super::is_valid_username("has space"));
        let long = "a".repeat(33);
        assert!(!super::is_valid_username(&long));
    }

    #[test]
    fn wallet_validation() {
        assert!(super::is_valid_wallet("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd"));
        assert!(super::is_valid_wallet("0xABCDEFABCDEFABCDEFABCDEFABCDEFABCDEFABCD"));
        assert!(!super::is_valid_wallet("0xabc"));
        assert!(!super::is_valid_wallet("0xzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"));
    }

    #[test]
    fn constant_time_eq_works() {
        assert!(super::constant_time_eq("secret", "secret"));
        assert!(!super::constant_time_eq("secret", "Secret"));
        assert!(!super::constant_time_eq("a", "ab"));
    }

    #[test]
    fn wallet_username_blocklist() {
        std::env::set_var("WALLET_BLOCKLIST", "0x0000000000000000000000000000000000000000,0xdeadbeef000000000000000000000000000000");
        std::env::set_var("USERNAME_BLOCKLIST", "admin,root,system");
        assert!(super::is_wallet_blocked("0x0000000000000000000000000000000000000000"));
        assert!(!super::is_wallet_blocked("0x1111111111111111111111111111111111111111"));
        assert!(super::is_username_blocked("Admin")); // case-insensitive
        assert!(!super::is_username_blocked("normaluser"));
    }

    #[test]
    fn webhook_token_validation() {
        // Prefer specific token over generic
        std::env::set_var("WEBHOOK_TOKEN", "generic");
        std::env::set_var("TELEGRAM_WEBHOOK_TOKEN", "specific");

        let mut headers = HeaderMap::new();
        headers.insert("x-webhook-token", http::HeaderValue::from_static("specific"));
        assert!(super::webhook_token_valid(&headers, None, "TELEGRAM_WEBHOOK_TOKEN"));

        // Query param path
        let headers2 = HeaderMap::new();
        let mut q = std::collections::HashMap::new();
        q.insert("token".to_string(), "generic".to_string());
        // If no specific env, falls back to generic
        std::env::remove_var("TELEGRAM_WEBHOOK_TOKEN");
        assert!(super::webhook_token_valid(&headers2, Some(&q), "TELEGRAM_WEBHOOK_TOKEN"));

        // Deny when no env configured
        std::env::remove_var("WEBHOOK_TOKEN");
        assert!(!super::webhook_token_valid(&headers2, Some(&q), "TELEGRAM_WEBHOOK_TOKEN"));
    }
}
