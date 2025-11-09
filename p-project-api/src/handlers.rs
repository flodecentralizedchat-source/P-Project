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

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub wallet_address: Option<String>,
}

fn api_error(status: StatusCode, code: &'static str) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: code.to_string(),
        }),
    )
}

// Create user now returns the full User JSON

pub async fn create_user(
    State(state): State<AppState>,
    AxumJson(request): AxumJson<CreateUserRequest>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
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
        return Err(api_error(StatusCode::BAD_REQUEST, "invalid_username"));
    }

    // Wallet: basic 0x-prefixed 42-char hex (Ethereum-style)
    let wallet_ok = wallet.starts_with("0x")
        && wallet.len() == 42
        && wallet.chars().skip(2).all(|c| c.is_ascii_hexdigit());
    if !wallet_ok {
        return Err(api_error(StatusCode::BAD_REQUEST, "invalid_wallet_address"));
    }

    let user_id = generate_id();
    match state.db.create_user(&user_id, username, wallet).await {
        Ok(user) => Ok(Json(user)),
        Err(e) => {
            let es = e.to_string();
            if es.contains("1062") || es.contains("Duplicate entry") {
                Err(api_error(StatusCode::CONFLICT, "username_taken"))
            } else {
                Err(api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                ))
            }
        }
    }
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    match state.db.get_user(&id).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(api_error(StatusCode::NOT_FOUND, "not_found")),
        Err(_) => Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
        )),
    }
}


pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    AxumJson(request): AxumJson<UpdateUserRequest>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    let mut username_value = None;
    if let Some(raw_username) = request.username {
        let trimmed = raw_username.trim();
        let valid = !trimmed.is_empty()
            && trimmed.len() >= 3
            && trimmed.len() <= 32
            && trimmed
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if !valid {
            return Err(api_error(StatusCode::BAD_REQUEST, "invalid_username"));
        }
        username_value = Some(trimmed.to_string());
    }

    let mut wallet_value = None;
    if let Some(raw_wallet) = request.wallet_address {
        let trimmed = raw_wallet.trim();
        let valid = trimmed.startswith("0x")
            && trimmed.len() == 42
            && trimmed.chars().skip(2).all(|c| c.is_ascii_hexdigit());
        if !valid {
            return Err(api_error(StatusCode::BAD_REQUEST, "invalid_wallet_address"));
        }
        wallet_value = Some(trimmed.to_string());
    }

    if username_value.is_none() && wallet_value.is_none() {
        return Err(api_error(StatusCode::BAD_REQUEST, "missing_update_fields"));
    }

    match state
        .db
        .update_user(
            &id,
            username_value.as_deref(),
            wallet_value.as_deref(),
        )
        .await
    {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(api_error(StatusCode::NOT_FOUND, "not_found")),
        Err(e) => {
            let es = e.to_string();
            if es.contains("1062") || es.contains("Duplicate entry") {
                Err(api_error(StatusCode::CONFLICT, "username_taken"))
            } else {
                Err(api_error(StatusCode::INTERNAL_SERVER_ERROR, "internal_error"))
            }
        }
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
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    Err(api_error(StatusCode::NOT_IMPLEMENTED, "not_implemented"))
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
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    Err(api_error(StatusCode::NOT_IMPLEMENTED, "not_implemented"))
}

#[derive(Deserialize)]
pub struct UnstakeRequest {
    pub user_id: String,
}

pub async fn unstake_tokens(
    State(_state): State<AppState>,
    AxumJson(_request): AxumJson<UnstakeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    Err(api_error(StatusCode::NOT_IMPLEMENTED, "not_implemented"))
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
) -> Result<Json<AirdropClaimResponse>, (StatusCode, Json<ErrorResponse>)> {
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
) -> Result<Json<CreateAirdropResponse>, (StatusCode, Json<ErrorResponse>)> {
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
) -> Result<Json<BatchClaimResponse>, (StatusCode, Json<ErrorResponse>)> {
    // In a real implementation, we would process batch airdrop claims
    // For now, we'll return a placeholder response

    Ok(Json(BatchClaimResponse {
        success: true,
        claimed_amounts: vec![],
        message: "Batch airdrop claims processed".to_string(),
    }))
}
