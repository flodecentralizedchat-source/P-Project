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

// Community Events (AMAs & Events)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    AMA,
    CommunityEvent,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub description: String,
    pub event_type: EventType,
    pub scheduled_start: chrono::NaiveDateTime,
    pub scheduled_end: Option<chrono::NaiveDateTime>,
    pub link: Option<String>,
    pub created_by: String,
    pub created_at: chrono::NaiveDateTime,
}

// Referrals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralCode {
    pub code: String,
    pub user_id: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralRelation {
    pub id: String,
    pub referrer_user_id: String,
    pub referred_user_id: String,
    pub code: String,
    pub created_at: chrono::NaiveDateTime,
}

// Innovation Models: Partners, Ecosystem, UVPs

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PartnerIntegrationType {
    Payment,
    Oracle,
    Identity,
    Messaging,
    ECommerce,
    CarbonCredits,
    DeFi,
    NFT,
    Analytics,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partner {
    pub id: String,
    pub name: String,
    pub integration_type: PartnerIntegrationType,
    pub metadata: serde_json::Value,
    pub webhook_secret: Option<String>,
    pub active: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EcosystemComponentType {
    Service,
    Contract,
    API,
    UI,
    Bot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComponentStatus {
    Healthy,
    Degraded,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemComponent {
    pub id: String,
    pub name: String,
    pub component_type: EcosystemComponentType,
    pub version: String,
    pub status: ComponentStatus,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemLink {
    pub from_id: String,
    pub to_id: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueValueProposition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub metric_key: String,
    pub multiplier: f64,
}
