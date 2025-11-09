use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Json as AxumJson,
};
use p_project_core::{models::User, utils::generate_id};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub async fn root() -> &'static str {
    "P-Project API Server"
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub wallet_address: String,
}

// Create user now returns the full User JSON

pub async fn create_user(
    State(state): State<AppState>,
    AxumJson(request): AxumJson<CreateUserRequest>,
) -> Result<Json<User>, StatusCode> {
    // Basic input validation
    let username = request.username.trim();
    let wallet = request.wallet_address.trim();

    // Username: 3-32 chars, ASCII [A-Za-z0-9_-]
    let username_ok = !username.is_empty()
        && username.len() >= 3
        && username.len() <= 32
        && username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !username_ok {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Wallet: basic 0x-prefixed 42-char hex (Ethereum-style)
    let wallet_ok = wallet.starts_with("0x")
        && wallet.len() == 42
        && wallet
            .chars()
            .skip(2)
            .all(|c| c.is_ascii_hexdigit());
    if !wallet_ok {
        return Err(StatusCode::BAD_REQUEST);
    }

    let user_id = generate_id();
    match state
        .db
        .create_user(&user_id, username, wallet)
        .await
    {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            let es = e.to_string();
            if es.contains("1062") || es.contains("Duplicate entry") {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<User>, StatusCode> {
    match state.db.get_user(&id).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct TransferRequest {
    pub from_user_id: String,
    pub to_user_id: String,
    pub amount: f64,
}

pub async fn transfer_tokens(
    State(_state): State<AppState>,
    AxumJson(_request): AxumJson<TransferRequest>,
) -> Result<Json<Value>, StatusCode> {
    // In a real implementation, we would process the transfer
    // For now, we'll return a placeholder
    
    Err(StatusCode::NOT_IMPLEMENTED)
}

#[derive(Deserialize)]
pub struct StakeRequest {
    pub user_id: String,
    pub amount: f64,
    pub duration_days: i64,
}

pub async fn stake_tokens(
    State(_state): State<AppState>,
    AxumJson(_request): AxumJson<StakeRequest>,
) -> Result<Json<Value>, StatusCode> {
    // In a real implementation, we would process the staking
    // For now, we'll return a placeholder
    
    Err(StatusCode::NOT_IMPLEMENTED)
}

#[derive(Deserialize)]
pub struct UnstakeRequest {
    pub user_id: String,
}

pub async fn unstake_tokens(
    State(_state): State<AppState>,
    AxumJson(_request): AxumJson<UnstakeRequest>,
) -> Result<Json<Value>, StatusCode> {
    // In a real implementation, we would process the unstaking
    // For now, we'll return a placeholder
    
    Err(StatusCode::NOT_IMPLEMENTED)
}

#[derive(Deserialize)]
pub struct AirdropClaimRequest {
    pub user_id: String,
}

#[derive(Serialize)]
pub struct AirdropClaimResponse {
    pub success: bool,
    pub amount: Option<f64>,
    pub message: String,
}

pub async fn claim_airdrop(
    State(_state): State<AppState>,
    AxumJson(_request): AxumJson<AirdropClaimRequest>,
) -> Result<Json<AirdropClaimResponse>, StatusCode> {
    // In a real implementation, we would process the airdrop claim
    // For now, we'll return a placeholder response
    
    Ok(Json(AirdropClaimResponse {
        success: true,
        amount: Some(100.0),
        message: "Airdrop claimed successfully".to_string(),
    }))
}

#[derive(Deserialize)]
pub struct CreateAirdropRequest {
    pub total_amount: f64,
    pub recipients: Vec<(String, f64)>,
}

#[derive(Serialize)]
pub struct CreateAirdropResponse {
    pub airdrop_id: String,
    pub message: String,
}

pub async fn create_airdrop(
    State(_state): State<AppState>,
    AxumJson(_request): AxumJson<CreateAirdropRequest>,
) -> Result<Json<CreateAirdropResponse>, StatusCode> {
    // Create a new airdrop
    // In a real implementation, we would save this to the database
    // For now, we'll just return a placeholder response
    
    Ok(Json(CreateAirdropResponse {
        airdrop_id: "test_airdrop_id".to_string(),
        message: "Airdrop created successfully".to_string(),
    }))
}

#[derive(Deserialize)]
pub struct BatchClaimRequest {
    pub user_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct BatchClaimResponse {
    pub success: bool,
    pub claimed_amounts: Vec<(String, f64)>,
    pub message: String,
}

pub async fn batch_claim_airdrops(
    State(_state): State<AppState>,
    AxumJson(_request): AxumJson<BatchClaimRequest>,
) -> Result<Json<BatchClaimResponse>, StatusCode> {
    // In a real implementation, we would process batch airdrop claims
    // For now, we'll return a placeholder response
    
    Ok(Json(BatchClaimResponse {
        success: true,
        claimed_amounts: vec![],
        message: "Batch airdrop claims processed".to_string(),
    }))
}
