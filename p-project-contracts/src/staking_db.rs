//! Database adapter for staking contract

use crate::staking::{StakingContract, StakingError};
use p_project_core::database::MySqlDatabase;
use p_project_core::models::StakingInfo;
use serde_json;
use sqlx::Row;
use std::sync::Arc;

pub struct StakingDbAdapter {
    mysql: Arc<MySqlDatabase>,
}

impl StakingDbAdapter {
    pub fn new(mysql: Arc<MySqlDatabase>) -> Self {
        Self { mysql }
    }

    /// Save staking contract state to database
    pub async fn save_staking_state(&self, staking: &StakingContract) -> Result<(), StakingError> {
        // Serialize staking state
        let staking_json = serde_json::to_string(staking).map_err(|e| {
            StakingError::SerializationError(format!("Failed to serialize staking: {}", e))
        })?;

        // Save to MySQL using the core database method
        self.mysql
            .as_ref()
            .save_staking_state(&staking_json)
            .await
            .map_err(|e| {
                StakingError::DatabaseError(format!("Failed to save staking state: {}", e))
            })?;

        Ok(())
    }

    /// Load staking contract state from database
    pub async fn load_staking_state(&self) -> Result<Option<StakingContract>, StakingError> {
        // Load from MySQL using the core database method
        let state_data = self
            .mysql
            .as_ref()
            .load_latest_staking_state()
            .await
            .map_err(|e| {
                StakingError::DatabaseError(format!("Failed to load staking state: {}", e))
            })?;

        if let Some(staking_json) = state_data {
            let staking: StakingContract = serde_json::from_str(&staking_json).map_err(|e| {
                StakingError::SerializationError(format!("Failed to deserialize staking: {}", e))
            })?;
            Ok(Some(staking))
        } else {
            Ok(None)
        }
    }

    /// Save staking info to database
    pub async fn save_staking_info(
        &self,
        user_id: &str,
        staking_info: &StakingInfo,
    ) -> Result<(), StakingError> {
        let staking_json = serde_json::to_string(staking_info).map_err(|e| {
            StakingError::SerializationError(format!("Failed to serialize staking info: {}", e))
        })?;

        sqlx::query("INSERT INTO staking_infos (user_id, staking_data) VALUES (?, ?) ON DUPLICATE KEY UPDATE staking_data = VALUES(staking_data)")
            .bind(user_id)
            .bind(staking_json)
            .execute(self.mysql.as_ref().get_pool())
            .await
            .map_err(|e| StakingError::DatabaseError(format!("Failed to save staking info: {}", e)))?;

        Ok(())
    }

    /// Load staking info from database
    pub async fn load_staking_info(
        &self,
        user_id: &str,
    ) -> Result<Option<StakingInfo>, StakingError> {
        let row = sqlx::query("SELECT staking_data FROM staking_infos WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(self.mysql.as_ref().get_pool())
            .await
            .map_err(|e| {
                StakingError::DatabaseError(format!("Failed to load staking info: {}", e))
            })?;

        if let Some(row) = row {
            let staking_json: String = row.get("staking_data");
            let staking_info: StakingInfo = serde_json::from_str(&staking_json).map_err(|e| {
                StakingError::SerializationError(format!(
                    "Failed to deserialize staking info: {}",
                    e
                ))
            })?;
            Ok(Some(staking_info))
        } else {
            Ok(None)
        }
    }
}
