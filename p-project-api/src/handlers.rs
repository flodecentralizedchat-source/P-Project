use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use p_project_bridge::BridgeService;
use p_project_core::database::{BalanceError, MySqlDatabase};
use p_project_core::models::{Proposal, ProposalStatus, TransactionType, User};
use p_project_core::{AIService, AIServiceConfig, IoTService, IoTServiceConfig, Web2Service, Web2ServiceConfig};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::shared::AppState;

// Root/health
pub async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "p-project-api",
        "status": "ok"
    }))
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
    let id = p_project_core::utils::generate_id();
    match state
        .db
        .create_user(&id, &req.username, &req.wallet_address)
        .await
    {
        Ok(_user) => Ok(Json(CreateUserResponse { id })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub wallet_address: Option<String>,
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .db
        .update_user(
            &id,
            req.username.as_deref(),
            req.wallet_address.as_deref(),
        )
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
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

// Transfer
#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from_user_id: String,
    pub to_user_id: String,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct TransferResponse {
    pub transaction_id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub amount: f64,
}

pub async fn transfer_tokens(
    State(state): State<AppState>,
    Json(req): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, (StatusCode, Json<ErrorResponse>)> {
    if req.amount <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_amount".to_string(),
            }),
        ));
    }
    let tx_id = p_project_core::utils::generate_id();
    match state
        .db
        .transfer_tokens(
            &tx_id,
            &req.from_user_id,
            &req.to_user_id,
            req.amount,
            TransactionType::Transfer,
        )
        .await
    {
        Ok(()) => Ok(Json(TransferResponse {
            transaction_id: tx_id,
            from_user_id: req.from_user_id,
            to_user_id: req.to_user_id,
            amount: req.amount,
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
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

// Staking
#[derive(Debug, Deserialize)]
pub struct StakeRequest {
    pub user_id: String,
    pub amount: f64,
    pub duration_days: i64,
}

#[derive(Debug, Serialize)]
pub struct StakingInfoResponse {
    pub user_id: String,
    pub amount: f64,
    pub start_time: String,
    pub end_time: Option<String>,
    pub rewards_earned: f64,
}

pub async fn stake_tokens(
    State(state): State<AppState>,
    Json(req): Json<StakeRequest>,
) -> Result<Json<StakingInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    if req.amount <= 0.0 {
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
        .stake_tokens(&stake_id, &req.user_id, req.amount, req.duration_days)
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
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct UnstakeRequest {
    pub user_id: String,
}

pub async fn unstake_tokens(
    State(state): State<AppState>,
    Json(req): Json<UnstakeRequest>,
) -> Result<Json<StakingInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
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
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

// Airdrop
#[derive(Debug, Deserialize)]
pub struct CreateAirdropRequest {
    pub total_amount: f64,
    pub recipients: Option<Vec<RecipientAmount>>, // optional batch insert
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RecipientAmount {
    pub user_id: String,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct CreateAirdropResponse {
    pub airdrop_id: String,
}

pub async fn create_airdrop(
    State(state): State<AppState>,
    Json(req): Json<CreateAirdropRequest>,
) -> Result<Json<CreateAirdropResponse>, (StatusCode, Json<ErrorResponse>)> {
    if req.total_amount <= 0.0 {
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
        .create_airdrop(&airdrop_id, req.total_amount, None, None)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e.to_string() })))?;

    if let Some(list) = req.recipients.clone() {
        let vec_pairs: Vec<(String, f64)> = list.into_iter().map(|r| (r.user_id, r.amount)).collect();
        state
            .db
            .add_airdrop_recipients(&airdrop_id, &vec_pairs, Some("default"))
            .await
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse { error: e.to_string() }),
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
    pub amount: f64,
    pub message: String,
}

pub async fn claim_airdrop(
    State(state): State<AppState>,
    Json(req): Json<ClaimAirdropRequest>,
) -> Result<Json<AirdropClaimResponse>, (StatusCode, Json<ErrorResponse>)> {
    match state
        .db
        .claim_airdrop(&req.airdrop_id, &req.user_id)
        .await
    {
        Ok(amount) => Ok(Json(AirdropClaimResponse {
            airdrop_id: req.airdrop_id,
            user_id: req.user_id,
            amount,
            message: "claimed".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

// Bridge
#[derive(Debug, Deserialize)]
pub struct BridgeRequest {
    pub user_id: String,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
pub struct BridgeResponse {
    pub transaction_id: String,
}

pub async fn bridge_tokens(
    State(state): State<AppState>,
    Json(req): Json<BridgeRequest>,
) -> Result<Json<BridgeResponse>, (StatusCode, Json<ErrorResponse>)> {
    if req.amount <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid_amount".to_string(),
            }),
        ));
    }
    let svc = BridgeService::new(state.db.clone());
    match svc
        .bridge_tokens(&req.user_id, &req.from_chain, &req.to_chain, req.amount)
        .await
    {
        Ok(tx_id) => Ok(Json(BridgeResponse { transaction_id: tx_id })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e }),
        )),
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
    pub amount: f64,
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
            amount: s.amount,
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
    pub amount: f64,
    pub duration_days: i64,
}

#[derive(Debug, Serialize)]
pub struct StakingYieldResponse {
    pub projected_rewards: f64,
    pub total_return: f64,
    pub apy_rate: f64,
}

pub async fn calculate_staking_yield(
    Json(req): Json<StakingYieldRequest>,
) -> Result<Json<StakingYieldResponse>, (StatusCode, Json<ErrorResponse>)> {
    if req.amount <= 0.0 || req.duration_days <= 0 {
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
    let projected_rewards = req.amount * apy_rate * (req.duration_days as f64 / 365.0);
    let total_return = req.amount + projected_rewards;
    Ok(Json(StakingYieldResponse {
        projected_rewards,
        total_return,
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
    Json(_request): Json<CreateProposalRequest>,
) -> Result<Json<CreateProposalResponse>, (StatusCode, Json<ErrorResponse>)> {
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
    Json(_request): Json<VoteProposalRequest>,
) -> Result<Json<VoteProposalResponse>, (StatusCode, Json<ErrorResponse>)> {
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
    Json(_request): Json<TallyVotesRequest>,
) -> Result<Json<TallyVotesResponse>, (StatusCode, Json<ErrorResponse>)> {
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
    Json(_request): Json<ExecuteProposalRequest>,
) -> Result<Json<ExecuteProposalResponse>, (StatusCode, Json<ErrorResponse>)> {
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
    Json(_request): Json<DelegateVoteRequest>,
) -> Result<Json<DelegateVoteResponse>, (StatusCode, Json<ErrorResponse>)> {
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
    // Initialize AI service
    let ai_config = AIServiceConfig {
        api_key: "test_key".to_string(), // In production, get from environment variables
        model: "gpt-4".to_string(),
        temperature: 0.7,
    };
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
        },
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
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
    // Initialize AI service
    let ai_config = AIServiceConfig {
        api_key: "test_key".to_string(), // In production, get from environment variables
        model: "dall-e".to_string(),
        temperature: 0.7,
    };
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
        },
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
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
    // Initialize AI service
    let ai_config = AIServiceConfig {
        api_key: "test_key".to_string(), // In production, get from environment variables
        model: "fraud-detection-model".to_string(),
        temperature: 0.7,
    };
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
        },
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e.to_string() }),
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
    Json(req): Json<RegisterDonationBoxRequest>,
) -> Result<Json<RegisterDonationBoxResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize IoT service
    let iot_config = IoTServiceConfig {
        api_endpoint: "https://api.example.com".to_string(), // In production, get from environment variables
        auth_token: "test_token".to_string(),
    };
    let mut iot_service = IoTService::new(iot_config);
    
    match iot_service.register_donation_box(req.box_id, req.location, req.wallet_address) {
        Ok(box_data) => Ok(Json(RegisterDonationBoxResponse { box_data })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
    Json(req): Json<RecordDonationRequest>,
) -> Result<Json<RecordDonationResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize IoT service
    let iot_config = IoTServiceConfig {
        api_endpoint: "https://api.example.com".to_string(), // In production, get from environment variables
        auth_token: "test_token".to_string(),
    };
    let mut iot_service = IoTService::new(iot_config);
    
    match iot_service.record_donation(&req.box_id, req.amount, req.donor_address) {
        Ok(transaction) => Ok(Json(RecordDonationResponse { transaction })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
    Json(req): Json<GetDonationBoxStatusRequest>,
) -> Result<Json<GetDonationBoxStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize IoT service
    let iot_config = IoTServiceConfig {
        api_endpoint: "https://api.example.com".to_string(), // In production, get from environment variables
        auth_token: "test_token".to_string(),
    };
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
    // Initialize IoT service
    let iot_config = IoTServiceConfig {
        api_endpoint: "https://api.example.com".to_string(), // In production, get from environment variables
        auth_token: "test_token".to_string(),
    };
    let mut iot_service = IoTService::new(iot_config);
    
    match iot_service.register_wristband(req.wristband_id, req.refugee_id, req.camp_id) {
        Ok(wristband_data) => Ok(Json(RegisterWristbandResponse { wristband_data })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
    // Initialize IoT service
    let iot_config = IoTServiceConfig {
        api_endpoint: "https://api.example.com".to_string(), // In production, get from environment variables
        auth_token: "test_token".to_string(),
    };
    let mut iot_service = IoTService::new(iot_config);
    
    match iot_service.add_funds_to_wristband(&req.wristband_id, req.amount) {
        Ok(()) => Ok(Json(AddFundsToWristbandResponse {
            success: true,
            message: "Funds added successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
    // Initialize IoT service
    let iot_config = IoTServiceConfig {
        api_endpoint: "https://api.example.com".to_string(), // In production, get from environment variables
        auth_token: "test_token".to_string(),
    };
    let mut iot_service = IoTService::new(iot_config);
    
    match iot_service.process_wristband_transaction(&req.wristband_id, req.amount, req.transaction_type, req.vendor_id) {
        Ok(transaction) => Ok(Json(ProcessWristbandTransactionResponse { transaction })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
    Json(req): Json<GetWristbandStatusRequest>,
) -> Result<Json<GetWristbandStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize IoT service
    let iot_config = IoTServiceConfig {
        api_endpoint: "https://api.example.com".to_string(), // In production, get from environment variables
        auth_token: "test_token".to_string(),
    };
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
    Json(req): Json<CreateFoodQRRequest>,
) -> Result<Json<CreateFoodQRResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize IoT service
    let iot_config = IoTServiceConfig {
        api_endpoint: "https://api.example.com".to_string(), // In production, get from environment variables
        auth_token: "test_token".to_string(),
    };
    let mut iot_service = IoTService::new(iot_config);
    
    // Parse the expiration date
    let expiration_date = match chrono::NaiveDateTime::parse_from_str(&req.expiration_date, "%Y-%m-%dT%H:%M:%S") {
        Ok(date) => date,
        Err(_) => match chrono::NaiveDateTime::parse_from_str(&req.expiration_date, "%Y-%m-%d %H:%M:%S") {
            Ok(date) => date,
            Err(e) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse { error: format!("Invalid date format: {}", e) }),
                ));
            }
        }
    };
    
    match iot_service.create_food_qr(req.distribution_point, req.food_type, req.quantity, expiration_date) {
        Ok(qr_data) => Ok(Json(CreateFoodQRResponse { qr_data })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
    Json(req): Json<ClaimFoodQRRequest>,
) -> Result<Json<ClaimFoodQRResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize IoT service
    let iot_config = IoTServiceConfig {
        api_endpoint: "https://api.example.com".to_string(), // In production, get from environment variables
        auth_token: "test_token".to_string(),
    };
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
            Json(ErrorResponse { error: e.to_string() }),
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
    // Initialize IoT service
    let iot_config = IoTServiceConfig {
        api_endpoint: "https://api.example.com".to_string(), // In production, get from environment variables
        auth_token: "test_token".to_string(),
    };
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
    Json(req): Json<CreateDonationWidgetRequest>,
) -> Result<Json<CreateDonationWidgetResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize Web2 service
    let mut api_keys = std::collections::HashMap::new();
    api_keys.insert("facebook".to_string(), "fb_key".to_string());
    api_keys.insert("youtube".to_string(), "yt_key".to_string());
    api_keys.insert("telegram".to_string(), "tg_key".to_string());
    api_keys.insert("discord".to_string(), "dc_key".to_string());
    
    let web2_config = Web2ServiceConfig {
        api_keys,
        webhook_url: "https://api.example.com/webhook".to_string(), // In production, get from environment variables
    };
    let mut web2_service = Web2Service::new(web2_config);
    
    match web2_service.create_donation_widget(req.config) {
        Ok(widget_data) => Ok(Json(CreateDonationWidgetResponse { widget_data })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
    Json(req): Json<ProcessSocialMediaDonationRequest>,
) -> Result<Json<ProcessSocialMediaDonationResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize Web2 service
    let mut api_keys = std::collections::HashMap::new();
    api_keys.insert("facebook".to_string(), "fb_key".to_string());
    api_keys.insert("youtube".to_string(), "yt_key".to_string());
    api_keys.insert("telegram".to_string(), "tg_key".to_string());
    api_keys.insert("discord".to_string(), "dc_key".to_string());
    
    let web2_config = Web2ServiceConfig {
        api_keys,
        webhook_url: "https://api.example.com/webhook".to_string(), // In production, get from environment variables
    };
    let web2_service = Web2Service::new(web2_config);
    
    match web2_service.process_social_media_donation(req.donation_data).await {
        Ok(donation_response) => Ok(Json(ProcessSocialMediaDonationResponse { donation_response })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
    Json(req): Json<GenerateWidgetHtmlRequest>,
) -> Result<Json<GenerateWidgetHtmlResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize Web2 service
    let mut api_keys = std::collections::HashMap::new();
    api_keys.insert("facebook".to_string(), "fb_key".to_string());
    api_keys.insert("youtube".to_string(), "yt_key".to_string());
    api_keys.insert("telegram".to_string(), "tg_key".to_string());
    api_keys.insert("discord".to_string(), "dc_key".to_string());
    
    let web2_config = Web2ServiceConfig {
        api_keys,
        webhook_url: "https://api.example.com/webhook".to_string(), // In production, get from environment variables
    };
    let web2_service = Web2Service::new(web2_config);
    
    match web2_service.generate_widget_html(&req.widget_id) {
        Ok(html) => Ok(Json(GenerateWidgetHtmlResponse { html })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
    Json(req): Json<CreateYouTubeTipConfigRequest>,
) -> Result<Json<CreateYouTubeTipConfigResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize Web2 service
    let mut api_keys = std::collections::HashMap::new();
    api_keys.insert("facebook".to_string(), "fb_key".to_string());
    api_keys.insert("youtube".to_string(), "yt_key".to_string());
    api_keys.insert("telegram".to_string(), "tg_key".to_string());
    api_keys.insert("discord".to_string(), "dc_key".to_string());
    
    let web2_config = Web2ServiceConfig {
        api_keys,
        webhook_url: "https://api.example.com/webhook".to_string(), // In production, get from environment variables
    };
    let mut web2_service = Web2Service::new(web2_config);
    
    match web2_service.create_youtube_tip_config(req.config) {
        Ok(config_id) => Ok(Json(CreateYouTubeTipConfigResponse {
            config_id,
            success: true,
            message: "YouTube tip configuration created successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
    Json(req): Json<ProcessYouTubeTipRequest>,
) -> Result<Json<ProcessYouTubeTipResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize Web2 service
    let mut api_keys = std::collections::HashMap::new();
    api_keys.insert("facebook".to_string(), "fb_key".to_string());
    api_keys.insert("youtube".to_string(), "yt_key".to_string());
    api_keys.insert("telegram".to_string(), "tg_key".to_string());
    api_keys.insert("discord".to_string(), "dc_key".to_string());
    
    let web2_config = Web2ServiceConfig {
        api_keys,
        webhook_url: "https://api.example.com/webhook".to_string(), // In production, get from environment variables
    };
    let web2_service = Web2Service::new(web2_config);
    
    match web2_service.process_youtube_tip(req.tip_data).await {
        Ok(donation_response) => Ok(Json(ProcessYouTubeTipResponse { donation_response })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
    Json(req): Json<RegisterMessagingBotRequest>,
) -> Result<Json<RegisterMessagingBotResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize Web2 service
    let mut api_keys = std::collections::HashMap::new();
    api_keys.insert("facebook".to_string(), "fb_key".to_string());
    api_keys.insert("youtube".to_string(), "yt_key".to_string());
    api_keys.insert("telegram".to_string(), "tg_key".to_string());
    api_keys.insert("discord".to_string(), "dc_key".to_string());
    
    let web2_config = Web2ServiceConfig {
        api_keys,
        webhook_url: "https://api.example.com/webhook".to_string(), // In production, get from environment variables
    };
    let mut web2_service = Web2Service::new(web2_config);
    
    match web2_service.register_messaging_bot(req.config) {
        Ok(bot_id) => Ok(Json(RegisterMessagingBotResponse {
            bot_id,
            success: true,
            message: "Messaging bot registered successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
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
    Json(req): Json<ProcessBotCommandRequest>,
) -> Result<Json<ProcessBotCommandResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Initialize Web2 service
    let mut api_keys = std::collections::HashMap::new();
    api_keys.insert("facebook".to_string(), "fb_key".to_string());
    api_keys.insert("youtube".to_string(), "yt_key".to_string());
    api_keys.insert("telegram".to_string(), "tg_key".to_string());
    api_keys.insert("discord".to_string(), "dc_key".to_string());
    
    let web2_config = Web2ServiceConfig {
        api_keys,
        webhook_url: "https://api.example.com/webhook".to_string(), // In production, get from environment variables
    };
    let web2_service = Web2Service::new(web2_config);
    
    match web2_service.process_bot_command(req.command_data).await {
        Ok(command_response) => Ok(Json(ProcessBotCommandResponse { command_response })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse { error: e.to_string() }),
        )),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}