use p_project_core::database::MySqlDatabase;
use std::{collections::HashMap, sync::Arc};

mod adapter;
mod config;
mod error;
mod eth;
mod relayer;
mod solana;
mod store;
mod sui;

// New modules for relayer components
mod eth_listener;
mod solana_relayer;
mod sui_relayer;

pub use adapter::{AdapterTxStatus, ChainAdapter};
use config::{BridgeConfig, EvmConfig, EthConfig};
pub use error::BridgeError;
use eth::EthereumAdapter;
use relayer::Relayer;
use solana::SolanaAdapter;
pub use store::{BoxedBridgeError, BridgeStore};
use sui::SuiAdapter;

// Re-export the new modules
pub use eth_listener::EthEventListener;
pub use solana_relayer::SolanaRelayer;
pub use sui_relayer::SuiRelayer;

// Wrapper struct to implement BridgeStore for Arc<MySqlDatabase>
struct DatabaseWrapper {
    db: Arc<MySqlDatabase>,
}

#[async_trait::async_trait]
impl BridgeStore for DatabaseWrapper {
    async fn create_bridge_tx(
        &self,
        id: &str,
        user_id: &str,
        token: &str,
        from_chain: &str,
        to_chain: &str,
        amount: f64,
        status: &str,
    ) -> Result<(), BoxedBridgeError> {
        self.db
            .as_ref()
            .insert_bridge_tx(id, user_id, token, from_chain, to_chain, amount, status)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }

    async fn set_bridge_src_tx(&self, id: &str, src_tx_hash: &str) -> Result<(), BoxedBridgeError> {
        self.db
            .as_ref()
            .update_bridge_src_tx(id, src_tx_hash)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }

    async fn set_bridge_dst_tx(&self, id: &str, dst_tx_hash: &str) -> Result<(), BoxedBridgeError> {
        self.db
            .as_ref()
            .update_bridge_dst_tx(id, dst_tx_hash)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }

    async fn set_bridge_lock_id(&self, id: &str, lock_id: &str) -> Result<(), BoxedBridgeError> {
        self.db
            .as_ref()
            .update_bridge_lock_id(id, lock_id)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }

    async fn update_bridge_status(
        &self,
        id: &str,
        status: &str,
        error_msg: Option<&str>,
    ) -> Result<(), BoxedBridgeError> {
        self.db
            .as_ref()
            .update_bridge_status_row(id, status, error_msg)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }

    async fn get_bridge_tx(
        &self,
        id: &str,
    ) -> Result<p_project_core::models::BridgeTx, BoxedBridgeError> {
        self.db
            .as_ref()
            .fetch_bridge_tx(id)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }

    async fn list_bridge_locked_without_dst(
        &self,
        from_chain: &str,
    ) -> Result<Vec<p_project_core::models::BridgeTx>, BoxedBridgeError> {
        self.db
            .as_ref()
            .list_locked_without_dst(from_chain)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }
}

pub struct BridgeService {
    db: Arc<dyn BridgeStore + Send + Sync>,
    supported_chains: Vec<String>,
    adapters: HashMap<String, Box<dyn adapter::ChainAdapter + Send + Sync>>,
}

impl BridgeService {
    pub fn new(db: Arc<MySqlDatabase>) -> Self {
        let cfg = BridgeConfig::from_env();
        let adapters = Self::build_default_adapters(&cfg);
        let db_wrapper = DatabaseWrapper { db };
        Self::with_adapters(Arc::new(db_wrapper), adapters)
    }

    pub fn with_adapters(
        db: Arc<dyn BridgeStore + Send + Sync>,
        adapters: HashMap<String, Box<dyn adapter::ChainAdapter + Send + Sync>>,
    ) -> Self {
        let supported_chains = adapters.keys().cloned().collect::<Vec<_>>();
        Self {
            db,
            supported_chains,
            adapters,
        }
    }

    fn build_default_adapters(
        cfg: &BridgeConfig,
    ) -> HashMap<String, Box<dyn adapter::ChainAdapter + Send + Sync>> {
        let mut adapters: HashMap<String, Box<dyn adapter::ChainAdapter + Send + Sync>> =
            HashMap::new();

        // Register EVM adapters: prefer multi-EVM list, fallback to single ETH_*
        if !cfg.evm.is_empty() {
            for evm_cfg in &cfg.evm {
                let chain_name = evm_cfg.name.trim().to_string();
                let eth_cfg: EthConfig = evm_cfg.to_eth_config();
                let adapter = EthereumAdapter::new(Some(&eth_cfg));
                adapters.insert(chain_name, Box::new(adapter));
            }
        } else {
            // Backward-compat single Ethereum config
            let eth = EthereumAdapter::new(cfg.eth.as_ref());
            adapters.insert("Ethereum".to_string(), Box::new(eth));
        }

        let sol = SolanaAdapter::new(cfg.solana.as_ref());
        adapters.insert("Solana".to_string(), Box::new(sol));

        let sui = SuiAdapter::new(cfg.sui.as_ref());
        adapters.insert("Sui".to_string(), Box::new(sui));

        adapters
    }

    /// Get supported chains
    pub fn get_supported_chains(&self) -> &[String] {
        &self.supported_chains
    }

    pub fn relayer(&self) -> Relayer<'_> {
        Relayer::new(&self.adapters, self.db.as_ref())
    }

    /// Bridge tokens from one chain to another
    pub async fn bridge_tokens(
        &self,
        user_id: &str,
        from_chain: &str,
        to_chain: &str,
        amount: f64,
    ) -> Result<String, String> {
        if !self.supported_chains.contains(&from_chain.to_string()) {
            return Err(format!("Unsupported source chain: {}", from_chain));
        }

        if !self.supported_chains.contains(&to_chain.to_string()) {
            return Err(format!("Unsupported destination chain: {}", to_chain));
        }

        let src = self
            .adapters
            .get(from_chain)
            .ok_or_else(|| format!("No adapter for {}", from_chain))?;
        let dst = self
            .adapters
            .get(to_chain)
            .ok_or_else(|| format!("No adapter for {}", to_chain))?;

        let token = "P";

        let tx_id = p_project_core::utils::generate_id();

        if let Err(e) = self
            .db
            .create_bridge_tx(
                &tx_id, user_id, token, from_chain, to_chain, amount, "Pending",
            )
            .await
        {
            return Err(format!("DB error: {}", e));
        }

        let src_tx = match src.lock(user_id, token, amount, to_chain).await {
            Ok(h) => h,
            Err(e) => {
                let _ = self
                    .db
                    .update_bridge_status(&tx_id, "Failed", Some(&format!("{:?}", e)))
                    .await;
                return Err(match e {
                    BridgeError::Other(s) => s,
                    _ => format!("Bridge error: {:?}", e),
                });
            }
        };
        let _ = self.db.set_bridge_src_tx(&tx_id, &src_tx).await;
        let _ = self.db.update_bridge_status(&tx_id, "Locked", None).await;

        if let Ok(Some(lock_id)) = src.extract_lock_id(&src_tx).await {
            let _ = self.db.set_bridge_lock_id(&tx_id, &lock_id).await;
        }

        let lock_id_opt = match self.db.get_bridge_tx(&tx_id).await {
            Ok(rec) => rec.lock_id,
            Err(_) => None,
        };
        let dst_tx = match dst
            .mint_or_release(
                user_id,
                token,
                amount,
                from_chain,
                &src_tx,
                lock_id_opt.as_deref(),
            )
            .await
        {
            Ok(h) => h,
            Err(e) => {
                let _ = self
                    .db
                    .update_bridge_status(&tx_id, "Failed", Some(&format!("{:?}", e)))
                    .await;
                return Err(match e {
                    BridgeError::Other(s) => s,
                    _ => format!("Bridge error: {:?}", e),
                });
            }
        };
        let _ = self.db.set_bridge_dst_tx(&tx_id, &dst_tx).await;
        let _ = self.db.update_bridge_status(&tx_id, "Minted", None).await;

        Ok(tx_id)
    }

    /// Get bridge transaction status
    pub async fn get_bridge_status(&self, tx_id: &str) -> Result<BridgeStatus, String> {
        match self.db.get_bridge_tx(tx_id).await {
            Ok(rec) => Ok(BridgeStatus {
                tx_id: rec.id,
                status: match rec.status {
                    p_project_core::models::BridgeTxStatus::Pending => "Pending".to_string(),
                    p_project_core::models::BridgeTxStatus::Locked => "Locked".to_string(),
                    p_project_core::models::BridgeTxStatus::Minted => "Minted".to_string(),
                    p_project_core::models::BridgeTxStatus::Failed => "Failed".to_string(),
                },
                from_chain: rec.from_chain,
                to_chain: rec.to_chain,
                amount: rec.amount,
            }),
            Err(e) => Err(format!("DB error: {}", e)),
        }
    }
}

pub struct BridgeStatus {
    pub tx_id: String,
    pub status: String,
    pub from_chain: String,
    pub to_chain: String,
    pub amount: f64,
}
