use crate::shared::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Json as AxumJson,
};
use p_project_core::{
    database::BalanceError,
    models::{StakingInfo, TransactionType, User},
    utils::generate_id,
};
use serde::{Deserialize, Serialize};

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
        let valid = trimmed.starts_with("0x")
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
        .update_user(&id, username_value.as_deref(), wallet_value.as_deref())
        .await
    {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(api_error(StatusCode::NOT_FOUND, "not_found")),
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

#[derive(Deserialize)]
pub struct TransferRequest {
    pub from_user_id: String,
    pub to_user_id: String,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct TransferResponse {
    pub transaction_id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub amount: f64,
}

pub async fn transfer_tokens(
    State(state): State<AppState>,
    AxumJson(request): AxumJson<TransferRequest>,
) -> Result<Json<TransferResponse>, (StatusCode, Json<ErrorResponse>)> {
    if request.from_user_id == request.to_user_id {
        return Err(api_error(StatusCode::BAD_REQUEST, "same_user"));
    }
    if request.amount <= 0.0 {
        return Err(api_error(StatusCode::BAD_REQUEST, "invalid_amount"));
    }

    let transaction_id = generate_id();
    match state
        .db
        .transfer_tokens(
            &transaction_id,
            &request.from_user_id,
            &request.to_user_id,
            request.amount,
            TransactionType::Transfer,
        )
        .await
    {
        Ok(_) => Ok(Json(TransferResponse {
            transaction_id,
            from_user_id: request.from_user_id,
            to_user_id: request.to_user_id,
            amount: request.amount,
        })),
        Err(BalanceError::InvalidAmount) => {
            Err(api_error(StatusCode::BAD_REQUEST, "invalid_amount"))
        }
        Err(BalanceError::InsufficientBalance) => {
            Err(api_error(StatusCode::BAD_REQUEST, "insufficient_balance"))
        }
        Err(BalanceError::UserNotFound) => Err(api_error(StatusCode::NOT_FOUND, "user_not_found")),
        Err(_) => Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
        )),
    }
}

#[derive(Deserialize)]
pub struct StakeRequest {
    pub user_id: String,
    pub amount: f64,
    pub duration_days: i64,
}

pub async fn stake_tokens(
    State(state): State<AppState>,
    AxumJson(request): AxumJson<StakeRequest>,
) -> Result<Json<StakingInfo>, (StatusCode, Json<ErrorResponse>)> {
    if request.amount <= 0.0 || request.duration_days <= 0 {
        return Err(api_error(StatusCode::BAD_REQUEST, "invalid_amount"));
    }

    match state
        .db
        .stake_tokens(
            &generate_id(),
            &request.user_id,
            request.amount,
            request.duration_days,
        )
        .await
    {
        Ok(info) => Ok(Json(info)),
        Err(BalanceError::InsufficientBalance) => {
            Err(api_error(StatusCode::BAD_REQUEST, "insufficient_balance"))
        }
        Err(BalanceError::InvalidAmount) => {
            Err(api_error(StatusCode::BAD_REQUEST, "invalid_amount"))
        }
        Err(_) => Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
        )),
    }
}

#[derive(Deserialize)]
pub struct UnstakeRequest {
    pub user_id: String,
    pub stake_id: Option<String>,
}

pub async fn unstake_tokens(
    State(state): State<AppState>,
    AxumJson(request): AxumJson<UnstakeRequest>,
) -> Result<Json<StakingInfo>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .db
        .unstake_tokens(&request.user_id, request.stake_id.as_deref())
        .await
    {
        Ok(info) => Ok(Json(info)),
        Err(BalanceError::StakeNotFound) => {
            Err(api_error(StatusCode::NOT_FOUND, "stake_not_found"))
        }
        Err(BalanceError::InsufficientBalance) => {
            Err(api_error(StatusCode::BAD_REQUEST, "insufficient_balance"))
        }
        Err(_) => Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
        )),
    }
}

#[derive(Deserialize)]
pub struct AirdropClaimRequest {
    pub airdrop_id: String,
    pub user_id: String,
}

#[derive(Serialize)]
pub struct AirdropClaimResponse {
    pub airdrop_id: String,
    pub user_id: String,
    pub amount: f64,
    pub message: String,
}

pub async fn claim_airdrop(
    State(state): State<AppState>,
    AxumJson(request): AxumJson<AirdropClaimRequest>,
) -> Result<Json<AirdropClaimResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .db
        .claim_airdrop(&request.airdrop_id, &request.user_id)
        .await
    {
        Ok(amount) => Ok(Json(AirdropClaimResponse {
            airdrop_id: request.airdrop_id,
            user_id: request.user_id,
            amount,
            message: "Airdrop claimed successfully".to_string(),
        })),
        Err(sqlx::Error::RowNotFound) => Err(api_error(StatusCode::NOT_FOUND, "claim_not_found")),
        Err(_) => Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
        )),
    }
}

#[derive(Deserialize)]
pub struct AirdropRecipient {
    pub user_id: String,
    pub amount: f64,
}

#[derive(Deserialize)]
pub struct CreateAirdropRequest {
    pub total_amount: f64,
    pub recipients: Vec<AirdropRecipient>,
    pub category: Option<String>,
}

#[derive(Serialize)]
pub struct CreateAirdropResponse {
    pub airdrop_id: String,
    pub total_amount: f64,
    pub recipients: usize,
}

pub async fn create_airdrop(
    State(state): State<AppState>,
    AxumJson(request): AxumJson<CreateAirdropRequest>,
) -> Result<Json<CreateAirdropResponse>, (StatusCode, Json<ErrorResponse>)> {
    if request.total_amount <= 0.0 || request.recipients.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "invalid_airdrop"));
    }

    let total_recipients_amount: f64 = request.recipients.iter().map(|r| r.amount).sum();
    if total_recipients_amount > request.total_amount {
        return Err(api_error(StatusCode::BAD_REQUEST, "amount_mismatch"));
    }

    let airdrop_id = generate_id();
    let recipient_pairs: Vec<(String, f64)> = request
        .recipients
        .iter()
        .map(|r| (r.user_id.clone(), r.amount))
        .collect();

    if recipient_pairs.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "no_recipients"));
    }

    match state
        .db
        .create_airdrop(&airdrop_id, request.total_amount, None, None)
        .await
    {
        Ok(_) => (),
        Err(_) => {
            return Err(api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
            ))
        }
    }

    match state
        .db
        .add_airdrop_recipients(&airdrop_id, &recipient_pairs, request.category.as_deref())
        .await
    {
        Ok(_) => Ok(Json(CreateAirdropResponse {
            airdrop_id,
            total_amount: request.total_amount,
            recipients: recipient_pairs.len(),
        })),
        Err(_) => Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
        )),
    }
}

#[derive(Deserialize)]
pub struct BatchClaimRequest {
    pub airdrop_id: String,
    pub user_ids: Vec<String>,
}

#[derive(Serialize)]
pub struct BatchClaimResult {
    pub user_id: String,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct BatchClaimResponse {
    pub claimed: Vec<BatchClaimResult>,
}

pub async fn batch_claim_airdrops(
    State(state): State<AppState>,
    AxumJson(request): AxumJson<BatchClaimRequest>,
) -> Result<Json<BatchClaimResponse>, (StatusCode, Json<ErrorResponse>)> {
    if request.user_ids.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "no_user_ids"));
    }

    match state
        .db
        .batch_claim_airdrops(&request.airdrop_id, &request.user_ids)
        .await
    {
        Ok(claimed) => Ok(Json(BatchClaimResponse {
            claimed: claimed
                .into_iter()
                .map(|(user_id, amount)| BatchClaimResult { user_id, amount })
                .collect(),
        })),
        Err(_) => Err(api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
        )),
    }
}
