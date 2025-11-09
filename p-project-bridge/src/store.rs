use async_trait::async_trait;
use std::error::Error;

use p_project_core::database::MySqlDatabase;
use p_project_core::models::BridgeTx;

pub type BoxedBridgeError = Box<dyn Error + Send + Sync>;

#[async_trait]
pub trait BridgeStore: Send + Sync {
    async fn create_bridge_tx(
        &self,
        id: &str,
        user_id: &str,
        token: &str,
        from_chain: &str,
        to_chain: &str,
        amount: f64,
        status: &str,
    ) -> Result<(), BoxedBridgeError>;

    async fn set_bridge_src_tx(&self, id: &str, src_tx_hash: &str) -> Result<(), BoxedBridgeError>;
    async fn set_bridge_dst_tx(&self, id: &str, dst_tx_hash: &str) -> Result<(), BoxedBridgeError>;
    async fn set_bridge_lock_id(&self, id: &str, lock_id: &str) -> Result<(), BoxedBridgeError>;
    async fn update_bridge_status(
        &self,
        id: &str,
        status: &str,
        error_msg: Option<&str>,
    ) -> Result<(), BoxedBridgeError>;
    async fn get_bridge_tx(&self, id: &str) -> Result<BridgeTx, BoxedBridgeError>;
    async fn list_bridge_locked_without_dst(
        &self,
        from_chain: &str,
    ) -> Result<Vec<BridgeTx>, BoxedBridgeError>;
}

#[async_trait]
impl BridgeStore for MySqlDatabase {
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
        self.insert_bridge_tx(id, user_id, token, from_chain, to_chain, amount, status)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }

    async fn set_bridge_src_tx(&self, id: &str, src_tx_hash: &str) -> Result<(), BoxedBridgeError> {
        self.update_bridge_src_tx(id, src_tx_hash)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }

    async fn set_bridge_dst_tx(&self, id: &str, dst_tx_hash: &str) -> Result<(), BoxedBridgeError> {
        self.update_bridge_dst_tx(id, dst_tx_hash)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }

    async fn set_bridge_lock_id(&self, id: &str, lock_id: &str) -> Result<(), BoxedBridgeError> {
        self.update_bridge_lock_id(id, lock_id)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }

    async fn update_bridge_status(
        &self,
        id: &str,
        status: &str,
        error_msg: Option<&str>,
    ) -> Result<(), BoxedBridgeError> {
        self.update_bridge_status_row(id, status, error_msg)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }

    async fn get_bridge_tx(&self, id: &str) -> Result<BridgeTx, BoxedBridgeError> {
        self.fetch_bridge_tx(id)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }

    async fn list_bridge_locked_without_dst(
        &self,
        from_chain: &str,
    ) -> Result<Vec<BridgeTx>, BoxedBridgeError> {
        self.list_locked_without_dst(from_chain)
            .await
            .map_err(|e| Box::new(e) as BoxedBridgeError)
    }
}
