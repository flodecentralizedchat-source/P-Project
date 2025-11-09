use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use chrono::Utc;
use p_project_bridge::{AdapterTxStatus, BridgeService, BridgeStore, ChainAdapter, BoxedBridgeError};
use p_project_core::models::{BridgeTx, BridgeTxStatus};
use p_project_core::utils;

struct MockStore {
    inner: Mutex<HashMap<String, BridgeTx>>,
}

impl Default for MockStore {
    fn default() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl MockStore {
    fn now() -> chrono::NaiveDateTime {
        Utc::now().naive_utc()
    }

    fn map_status(status: &str) -> BridgeTxStatus {
        match status {
            "Locked" => BridgeTxStatus::Locked,
            "Minted" => BridgeTxStatus::Minted,
            "Failed" => BridgeTxStatus::Failed,
            _ => BridgeTxStatus::Pending,
        }
    }
}

#[async_trait]
impl BridgeStore for MockStore {
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
        let mut map = self.inner.lock().unwrap();
        map.insert(
            id.to_string(),
            BridgeTx {
                id: id.to_string(),
                user_id: user_id.to_string(),
                token: token.to_string(),
                from_chain: from_chain.to_string(),
                to_chain: to_chain.to_string(),
                amount,
                lock_id: None,
                src_tx_hash: None,
                dst_tx_hash: None,
                status: MockStore::map_status(status),
                error_msg: None,
                created_at: MockStore::now(),
                updated_at: MockStore::now(),
            },
        );
        Ok(())
    }

    async fn set_bridge_src_tx(&self, id: &str, src_tx_hash: &str) -> Result<(), BoxedBridgeError> {
        if let Some(mut tx) = self.inner.lock().unwrap().get_mut(id) {
            tx.src_tx_hash = Some(src_tx_hash.to_string());
            tx.updated_at = MockStore::now();
        }
        Ok(())
    }

    async fn set_bridge_dst_tx(&self, id: &str, dst_tx_hash: &str) -> Result<(), BoxedBridgeError> {
        if let Some(mut tx) = self.inner.lock().unwrap().get_mut(id) {
            tx.dst_tx_hash = Some(dst_tx_hash.to_string());
            tx.updated_at = MockStore::now();
        }
        Ok(())
    }

    async fn set_bridge_lock_id(&self, id: &str, lock_id: &str) -> Result<(), BoxedBridgeError> {
        if let Some(mut tx) = self.inner.lock().unwrap().get_mut(id) {
            tx.lock_id = Some(lock_id.to_string());
            tx.updated_at = MockStore::now();
        }
        Ok(())
    }

    async fn update_bridge_status(
        &self,
        id: &str,
        status: &str,
        error_msg: Option<&str>,
    ) -> Result<(), BoxedBridgeError> {
        if let Some(mut tx) = self.inner.lock().unwrap().get_mut(id) {
            tx.status = MockStore::map_status(status);
            tx.error_msg = error_msg.map(|s| s.to_string());
            tx.updated_at = MockStore::now();
        }
        Ok(())
    }

    async fn get_bridge_tx(&self, id: &str) -> Result<BridgeTx, BoxedBridgeError> {
        let map = self.inner.lock().unwrap();
        map.get(id)
            .cloned()
            .ok_or_else(|| -> BoxedBridgeError { format!("bridge tx {} not found", id).into() })
    }

    async fn list_bridge_locked_without_dst(
        &self,
        from_chain: &str,
    ) -> Result<Vec<BridgeTx>, BoxedBridgeError> {
        let map = self.inner.lock().unwrap();
        Ok(map
            .values()
            .filter(|tx| {
                tx.from_chain == from_chain
                    && matches!(tx.status, BridgeTxStatus::Locked)
                    && tx.dst_tx_hash.is_none()
            })
            .cloned()
            .collect::<Vec<_>>())
    }
}

struct MockAdapter {
    name: &'static str,
}

impl MockAdapter {
    fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[async_trait]
impl ChainAdapter for MockAdapter {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn lock(
        &self,
        _user: &str,
        _token: &str,
        _amount: f64,
        _to_chain: &str,
    ) -> Result<String, p_project_bridge::BridgeError> {
        Ok(format!("mock-{}-lock-{}", self.name, utils::generate_id()))
    }

    async fn mint_or_release(
        &self,
        _user: &str,
        _token: &str,
        _amount: f64,
        _from_chain: &str,
        _source_tx: &str,
        lock_id: Option<&str>,
    ) -> Result<String, p_project_bridge::BridgeError> {
        Ok(format!(
            "mock-{}-mint-{}",
            self.name,
            lock_id.unwrap_or("none")
        ))
    }

    async fn get_tx_status(
        &self,
        tx_id: &str,
    ) -> Result<AdapterTxStatus, p_project_bridge::BridgeError> {
        Ok(AdapterTxStatus {
            tx_id: tx_id.to_string(),
            status: "Success".to_string(),
            confirmations: 5,
        })
    }

    async fn extract_lock_id(
        &self,
        tx_hash: &str,
    ) -> Result<Option<String>, p_project_bridge::BridgeError> {
        Ok(Some(format!("lockid-{}", tx_hash)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let store: Arc<dyn BridgeStore + Send + Sync> = Arc::new(MockStore::default());
    let mut adapters: HashMap<String, Box<dyn ChainAdapter + Send + Sync>> = HashMap::new();
    adapters.insert(
        "Ethereum".to_string(),
        Box::new(MockAdapter::new("Ethereum")),
    );
    adapters.insert("Solana".to_string(), Box::new(MockAdapter::new("Solana")));

    let service = BridgeService::with_adapters(store.clone(), adapters);
    println!(
        "Simulating bridge: supported chains = {:?}",
        service.get_supported_chains()
    );

    let tx_id = service
        .bridge_tokens("user-1", "Ethereum", "Solana", 123.45)
        .await?;
    println!("Created bridge tx {}", tx_id);

    let relayer = service.relayer();
    relayer.run_once().await;

    let record = store.get_bridge_tx(&tx_id).await?;
    println!(
        "Bridge record after relayer: id={} status={:?} src={} dst={:?} lock_id={:?}",
        record.id, record.status, record.src_tx_hash, record.dst_tx_hash, record.lock_id
    );

    Ok(())
}
