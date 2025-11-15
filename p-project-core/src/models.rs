use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub wallet_address: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Transfer,
    Burn,
    Reward,
    Staking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransaction {
    pub id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub amount: Decimal,
    pub transaction_type: TransactionType,
    pub timestamp: chrono::NaiveDateTime,
}

// Remittance domain models
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RemittanceStatus {
    Initiated,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remittance {
    pub id: String,
    pub from_user_id: String,
    pub to_user_id: String,
    pub amount: Decimal,
    pub fee: Decimal,
    pub net_amount: Decimal,
    pub status: RemittanceStatus,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

// Learning structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LearningContentType {
    Course,
    Quiz,
    Workshop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LearningActivityType {
    CourseCompletion,
    QuizCompletion,
    WorkshopParticipation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningContent {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content_type: LearningContentType,
    pub reward_tokens: Decimal,
    pub reward_points: i64,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningCompletion {
    pub id: String,
    pub user_id: String,
    pub content_id: String,
    pub activity_type: LearningActivityType,
    pub reward_tokens: Decimal,
    pub reward_points: i64,
    pub proof_reference: Option<String>,
    pub completed_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingInfo {
    pub user_id: String,
    pub amount: Decimal,
    pub start_time: chrono::NaiveDateTime,
    pub end_time: Option<chrono::NaiveDateTime>,
    pub rewards_earned: Decimal,
    pub tier_name: Option<String>, // Staking tier name
    pub is_compounding: bool,      // Whether rewards are compounded
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub creator_id: String,
    pub created_at: chrono::NaiveDateTime,
    pub voting_end_time: chrono::NaiveDateTime,
    pub status: ProposalStatus,
    // Execution details
    pub execution_type: Option<ProposalExecutionType>,
    pub execution_data: Option<String>, // JSON string with execution parameters
    pub executed_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalExecutionType {
    TokenTransfer,   // Transfer tokens from treasury
    ParameterChange, // Change system parameters
    ContractUpgrade, // Upgrade contract logic
    Airdrop,         // Distribute tokens to users
    StakingReward,   // Distribute staking rewards
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeTxStatus {
    Pending,
    Locked,
    Minted,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeTx {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: f64,
    pub lock_id: Option<String>,
    pub src_tx_hash: Option<String>,
    pub dst_tx_hash: Option<String>,
    pub status: BridgeTxStatus,
    pub error_msg: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
