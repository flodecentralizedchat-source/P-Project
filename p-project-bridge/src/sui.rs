use crate::adapter::{AdapterTxStatus, ChainAdapter};
use crate::config::SuiConfig;
use crate::error::BridgeError;
use async_trait::async_trait;
use p_project_core::utils::generate_id;

pub struct SuiAdapter {
    rpc_url: Option<String>,
    bridge_package: Option<String>,
    confirmations: u32,
}

impl SuiAdapter {
    pub fn new(cfg: Option<&SuiConfig>) -> Self {
        match cfg {
            Some(c) => Self {
                rpc_url: Some(c.rpc_url.clone()),
                bridge_package: Some(c.bridge_package.clone()),
                confirmations: c.confirmations,
            },
            None => Self {
                rpc_url: None,
                bridge_package: None,
                confirmations: 0,
            },
        }
    }
}

#[async_trait]
impl ChainAdapter for SuiAdapter {
    fn name(&self) -> &'static str {
        "Sui"
    }

    async fn lock(
        &self,
        _user: &str,
        _token: &str,
        _amount: f64,
        _to_chain: &str,
    ) -> Result<String, BridgeError> {
        if self.rpc_url.is_none() {
            return Err(BridgeError::ConfigMissing("SUI_RPC_URL"));
        }
        if self.bridge_package.is_none() {
            return Err(BridgeError::ConfigMissing("SUI_BRIDGE_PACKAGE"));
        }
        Ok(format!("sui_{}", generate_id()))
    }

    async fn mint_or_release(
        &self,
        _user: &str,
        _token: &str,
        _amount: f64,
        _from_chain: &str,
        _source_tx: &str,
        _lock_id: Option<&str>,
    ) -> Result<String, BridgeError> {
        if self.rpc_url.is_none() {
            return Err(BridgeError::ConfigMissing("SUI_RPC_URL"));
        }
        if self.bridge_package.is_none() {
            return Err(BridgeError::ConfigMissing("SUI_BRIDGE_PACKAGE"));
        }
        Ok(format!("sui_{}", generate_id()))
    }

    async fn get_tx_status(&self, tx_id: &str) -> Result<AdapterTxStatus, BridgeError> {
        Ok(AdapterTxStatus {
            tx_id: tx_id.to_string(),
            status: "Unknown".to_string(),
            confirmations: self.confirmations,
        })
    }
}
