use crate::error::BridgeError;
use async_trait::async_trait;

pub struct AdapterTxStatus {
    pub tx_id: String,
    pub status: String,
    pub confirmations: u32,
}

#[async_trait]
pub trait ChainAdapter {
    fn name(&self) -> &'static str;
    async fn lock(&self, user: &str, token: &str, amount: f64, to_chain: &str) -> Result<String, BridgeError>;
    async fn mint_or_release(&self, user: &str, token: &str, amount: f64, from_chain: &str, source_tx: &str) -> Result<String, BridgeError>;
    async fn get_tx_status(&self, tx_id: &str) -> Result<AdapterTxStatus, BridgeError>;
    fn supports_token(&self, _token: &str) -> bool { true }
}
