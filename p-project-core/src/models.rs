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
    pub amount: f64,
    pub transaction_type: TransactionType,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingInfo {
    pub user_id: String,
    pub amount: f64,
    pub start_time: chrono::NaiveDateTime,
    pub end_time: Option<chrono::NaiveDateTime>,
    pub rewards_earned: f64,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
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
