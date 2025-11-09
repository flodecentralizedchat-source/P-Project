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

pub use adapter::{AdapterTxStatus, ChainAdapter};
use config::BridgeConfig;
pub use error::BridgeError;
use eth::EthereumAdapter;
use relayer::Relayer;
use solana::SolanaAdapter;
pub use store::{BoxedBridgeError, BridgeStore};
use sui::SuiAdapter;

pub struct BridgeService {
    db: Arc<dyn BridgeStore + Send + Sync>,
    supported_chains: Vec<String>,
    adapters: HashMap<String, Box<dyn adapter::ChainAdapter + Send + Sync>>,
}

impl BridgeService {
    pub fn new(db: MySqlDatabase) -> Self {
        let cfg = BridgeConfig::from_env();
        let adapters = Self::build_default_adapters(&cfg);
        Self::with_adapters(Arc::new(db), adapters)
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
        let mut adapters: HashMap<String, Box<dyn adapter::ChainAdapter + Send + Sync>> = HashMap::new();

        let eth = EthereumAdapter::new(cfg.eth.as_ref());
        adapters.insert("Ethereum".to_string(), Box::new(eth));

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

    pub fn relayer(&self) -> Relayer {
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
