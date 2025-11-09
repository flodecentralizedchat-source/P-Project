//! Database adapter for airdrop contract

use crate::airdrop::{AirdropContract, AirdropError};
use p_project_core::database::MySqlDatabase;
use serde_json;
use sqlx::Row;
use std::sync::Arc;

pub struct AirdropDbAdapter {
    mysql: Arc<MySqlDatabase>,
}

impl AirdropDbAdapter {
    pub fn new(mysql: Arc<MySqlDatabase>) -> Self {
        Self { mysql }
    }

    /// Save airdrop contract state to database
    pub async fn save_airdrop_state(&self, airdrop: &AirdropContract) -> Result<(), AirdropError> {
        // Serialize airdrop state
        let airdrop_json = serde_json::to_string(airdrop).map_err(|e| {
            AirdropError::SerializationError(format!("Failed to serialize airdrop: {}", e))
        })?;

        // Save to MySQL
        sqlx::query("INSERT INTO airdrop_states (state_data) VALUES (?)")
            .bind(airdrop_json)
            .execute(self.mysql.as_ref().get_pool())
            .await
            .map_err(|e| {
                AirdropError::DatabaseError(format!("Failed to save airdrop state: {}", e))
            })?;

        Ok(())
    }

    /// Load airdrop contract state from database
    pub async fn load_airdrop_state(&self) -> Result<Option<AirdropContract>, AirdropError> {
        let row =
            sqlx::query("SELECT state_data FROM airdrop_states ORDER BY created_at DESC LIMIT 1")
                .fetch_optional(self.mysql.as_ref().get_pool())
                .await
                .map_err(|e| {
                    AirdropError::DatabaseError(format!("Failed to load airdrop state: {}", e))
                })?;

        if let Some(row) = row {
            let airdrop_json: String = row.get("state_data");
            let airdrop: AirdropContract = serde_json::from_str(&airdrop_json).map_err(|e| {
                AirdropError::SerializationError(format!("Failed to deserialize airdrop: {}", e))
            })?;
            Ok(Some(airdrop))
        } else {
            Ok(None)
        }
    }
}