//! Database adapter for token contract

use crate::token::{PProjectToken, TokenError, TokenEvent};
use p_project_core::database::MySqlDatabase;
use p_project_core::models::TokenTransaction;
use serde_json;
use std::sync::Arc;
use sqlx::Row;

pub struct TokenDbAdapter {
    mysql: Arc<MySqlDatabase>,
}

impl TokenDbAdapter {
    pub fn new(mysql: Arc<MySqlDatabase>) -> Self {
        Self { mysql }
    }

    /// Save token state to database
    pub async fn save_token_state(&self, token: &PProjectToken) -> Result<(), TokenError> {
        // Serialize token state
        let token_json = serde_json::to_string(token)
            .map_err(|e| TokenError::SerializationError(format!("Failed to serialize token: {}", e)))?;

        // Save to MySQL
        sqlx::query("INSERT INTO token_states (state_data) VALUES (?)")
            .bind(token_json)
            .execute(self.mysql.as_ref().get_pool())
            .await
            .map_err(|e| TokenError::DatabaseError(format!("Failed to save token state: {}", e)))?;

        Ok(())
    }

    /// Load token state from database
    pub async fn load_token_state(&self) -> Result<Option<PProjectToken>, TokenError> {
        let row = sqlx::query("SELECT state_data FROM token_states ORDER BY created_at DESC LIMIT 1")
            .fetch_optional(self.mysql.as_ref().get_pool())
            .await
            .map_err(|e| TokenError::DatabaseError(format!("Failed to load token state: {}", e)))?;

        if let Some(row) = row {
            let token_json: String = row.get("state_data");
            let token: PProjectToken = serde_json::from_str(&token_json)
                .map_err(|e| TokenError::SerializationError(format!("Failed to deserialize token: {}", e)))?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    /// Save transaction log to database
    pub async fn save_transaction(&self, transaction: &TokenTransaction) -> Result<(), TokenError> {
        let transaction_json = serde_json::to_string(transaction)
            .map_err(|e| TokenError::SerializationError(format!("Failed to serialize transaction: {}", e)))?;

        sqlx::query("INSERT INTO token_transactions (id, from_user_id, to_user_id, amount, transaction_type, timestamp, transaction_data) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(&transaction.id)
            .bind(&transaction.from_user_id)
            .bind(&transaction.to_user_id)
            .bind(transaction.amount)
            .bind(format!("{:?}", transaction.transaction_type))
            .bind(transaction.timestamp)
            .bind(transaction_json)
            .execute(self.mysql.as_ref().get_pool())
            .await
            .map_err(|e| TokenError::DatabaseError(format!("Failed to save transaction: {}", e)))?;

        Ok(())
    }

    /// Save event log to database
    pub async fn save_event(&self, event: &TokenEvent) -> Result<(), TokenError> {
        let event_json = serde_json::to_string(event)
            .map_err(|e| TokenError::SerializationError(format!("Failed to serialize event: {}", e)))?;

        sqlx::query("INSERT INTO token_events (event_type, user_id, amount, timestamp, details, event_data) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(&event.event_type)
            .bind(&event.user_id)
            .bind(event.amount)
            .bind(event.timestamp)
            .bind(&event.details)
            .bind(event_json)
            .execute(self.mysql.as_ref().get_pool())
            .await
            .map_err(|e| TokenError::DatabaseError(format!("Failed to save event: {}", e)))?;

        Ok(())
    }
}