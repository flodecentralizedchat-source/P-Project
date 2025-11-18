use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;
use p_project_bridge::{AdapterTxStatus, BoxedBridgeError, BridgeService, BridgeStore, ChainAdapter};
use p_project_bridge::BridgeError;
use p_project_core::models::{BridgeTx, BridgeTxStatus};

// ------------------------------
// Test helpers (mocks)
// ------------------------------

#[derive(Default)]
struct MockStore {
    inner: Mutex<HashMap<String, BridgeTx>>,
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
        if let Some(tx) = self.inner.lock().unwrap().get_mut(id) {
            tx.src_tx_hash = Some(src_tx_hash.to_string());
            tx.updated_at = MockStore::now();
        }
        Ok(())
    }

    async fn set_bridge_dst_tx(&self, id: &str, dst_tx_hash: &str) -> Result<(), BoxedBridgeError> {
        if let Some(tx) = self.inner.lock().unwrap().get_mut(id) {
            tx.dst_tx_hash = Some(dst_tx_hash.to_string());
            tx.updated_at = MockStore::now();
        }
        Ok(())
    }

    async fn set_bridge_lock_id(&self, id: &str, lock_id: &str) -> Result<(), BoxedBridgeError> {
        if let Some(tx) = self.inner.lock().unwrap().get_mut(id) {
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
        if let Some(tx) = self.inner.lock().unwrap().get_mut(id) {
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
            .ok_or_else(|| -> BoxedBridgeError { format!("bridge tx {id} not found").into() })
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
    // Controls get_tx_status output
    status: &'static str,
    confirmations: u32,
    // Controls mint_or_release
    mint_result: Result<String, BridgeError>,
}

impl MockAdapter {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            status: "Success",
            confirmations: 5,
            mint_result: Ok(format!("mock-{name}-mint")),
        }
    }

    fn with_status(mut self, status: &'static str, confirmations: u32) -> Self {
        self.status = status;
        self.confirmations = confirmations;
        self
    }

    fn with_mint_error(mut self, msg: &'static str) -> Self {
        self.mint_result = Err(BridgeError::TxFailed(msg.to_string()));
        self
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
    ) -> Result<String, BridgeError> {
        Ok(format!("mock-{}-lock", self.name))
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
        self.mint_result.clone()
    }

    async fn get_tx_status(&self, tx_id: &str) -> Result<AdapterTxStatus, BridgeError> {
        Ok(AdapterTxStatus {
            tx_id: tx_id.to_string(),
            status: self.status.to_string(),
            confirmations: self.confirmations,
        })
    }

    async fn extract_lock_id(&self, tx_hash: &str) -> Result<Option<String>, BridgeError> {
        Ok(Some(format!("lockid-{tx_hash}")))
    }
}

// ------------------------------
// Tests
// ------------------------------

#[tokio::test]
async fn relayer_mints_on_successful_confirmation() {
    let store: Arc<dyn BridgeStore + Send + Sync> = Arc::new(MockStore::default());

    // Seed a locked tx with src tx hash and lock id
    let id = "tx-1";
    store
        .create_bridge_tx(id, "user-1", "P", "Ethereum", "Solana", 100.0, "Locked")
        .await
        .unwrap();
    store.set_bridge_src_tx(id, "0xabc").await.unwrap();
    store.set_bridge_lock_id(id, "0xlock").await.unwrap();

    let mut adapters: HashMap<String, Box<dyn ChainAdapter + Send + Sync>> = HashMap::new();
    adapters.insert(
        "Ethereum".to_string(),
        Box::new(MockAdapter::new("Ethereum").with_status("Success", 2)),
    );
    adapters.insert(
        "Solana".to_string(),
        Box::new(MockAdapter::new("Solana")),
    );

    let service = BridgeService::with_adapters(store.clone(), adapters);
    let relayer = service.relayer();
    relayer.run_once().await;

    let rec = store.get_bridge_tx(id).await.unwrap();
    assert!(matches!(rec.status, BridgeTxStatus::Minted));
    assert!(rec.dst_tx_hash.is_some());
}

#[tokio::test]
async fn relayer_waits_for_confirmations() {
    let store: Arc<dyn BridgeStore + Send + Sync> = Arc::new(MockStore::default());
    let id = "tx-2";
    store
        .create_bridge_tx(id, "user-1", "P", "Ethereum", "Solana", 5.0, "Locked")
        .await
        .unwrap();
    store.set_bridge_src_tx(id, "0xdef").await.unwrap();
    store.set_bridge_lock_id(id, "0xlock2").await.unwrap();

    let mut adapters: HashMap<String, Box<dyn ChainAdapter + Send + Sync>> = HashMap::new();
    adapters.insert(
        "Ethereum".to_string(),
        Box::new(MockAdapter::new("Ethereum").with_status("Success", 0)),
    );
    adapters.insert(
        "Solana".to_string(),
        Box::new(MockAdapter::new("Solana")),
    );

    let service = BridgeService::with_adapters(store.clone(), adapters);
    let relayer = service.relayer();
    relayer.run_once().await;

    let rec = store.get_bridge_tx(id).await.unwrap();
    assert!(matches!(rec.status, BridgeTxStatus::Locked));
    assert!(rec.dst_tx_hash.is_none());
}

#[tokio::test]
async fn relayer_handles_mint_error() {
    let store: Arc<dyn BridgeStore + Send + Sync> = Arc::new(MockStore::default());
    let id = "tx-3";
    store
        .create_bridge_tx(id, "user-1", "P", "Ethereum", "Solana", 42.0, "Locked")
        .await
        .unwrap();
    store.set_bridge_src_tx(id, "0xaaa").await.unwrap();
    store.set_bridge_lock_id(id, "0xlock3").await.unwrap();

    let mut adapters: HashMap<String, Box<dyn ChainAdapter + Send + Sync>> = HashMap::new();
    adapters.insert(
        "Ethereum".to_string(),
        Box::new(MockAdapter::new("Ethereum").with_status("Success", 10)),
    );
    adapters.insert(
        "Solana".to_string(),
        Box::new(MockAdapter::new("Solana").with_mint_error("boom")),
    );

    let service = BridgeService::with_adapters(store.clone(), adapters);
    let relayer = service.relayer();
    relayer.run_once().await;

    let rec = store.get_bridge_tx(id).await.unwrap();
    assert!(matches!(rec.status, BridgeTxStatus::Failed));
    assert!(rec.error_msg.as_deref().unwrap_or_default().contains("boom"));
}

#[tokio::test]
async fn bridge_service_sets_lock_id_and_mints() {
    // Build adapters for a full bridge call path
    let store: Arc<dyn BridgeStore + Send + Sync> = Arc::new(MockStore::default());
    let mut adapters: HashMap<String, Box<dyn ChainAdapter + Send + Sync>> = HashMap::new();
    adapters.insert("Ethereum".to_string(), Box::new(MockAdapter::new("Ethereum")));
    adapters.insert("Solana".to_string(), Box::new(MockAdapter::new("Solana")));

    let service = BridgeService::with_adapters(store.clone(), adapters);
    let tx_id = service
        .bridge_tokens("user-42", "Ethereum", "Solana", 7.5)
        .await
        .expect("bridge_tokens should succeed");

    let rec = store.get_bridge_tx(&tx_id).await.unwrap();
    assert!(matches!(rec.status, BridgeTxStatus::Minted));
    assert!(rec.lock_id.is_some());
    assert!(rec.src_tx_hash.is_some());
    assert!(rec.dst_tx_hash.is_some());
}

// ------------------------------
// Small unit checks for relayer components
// ------------------------------

#[cfg(feature = "solana-relayer")]
#[test]
fn solana_relayer_smoke() {
    // Just verify construction and API surface compile/return
    let payer = solana_sdk::signature::Keypair::new();
    let relayer = p_project_bridge::SolanaRelayer::new(
        "http://localhost:8899",
        payer,
        "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
        "BridgeAuthority11111111111111111111111111111111",
    )
    .expect("construct relayer");
    let _ = relayer; // no runtime calls
}

#[tokio::test]
async fn sui_relayer_smoke() {
    let relayer = p_project_bridge::SuiRelayer::new(
        "https://fullnode.devnet.sui.io:443",
        "0xbridge_authority",
    );
    let res = relayer
        .mint_tokens("0xrecipient", 1, "lockid")
        .await
        .expect("mint placeholder");
    assert!(res.success);
}

#[test]
fn eth_listener_smoke() {
    // Construct without hitting the network (does not perform requests on new())
    let transport = web3::transports::Http::new("http://localhost:8545").unwrap();
    let web3 = web3::Web3::new(transport);
    let addr: web3::types::Address = "0x0000000000000000000000000000000000000000".parse().unwrap();
    let mut listener = p_project_bridge::EthEventListener::new(web3, addr);
    listener.set_last_processed_block(web3::types::U256::from(123u64));
}
