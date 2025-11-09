use chrono::{NaiveDateTime, Utc};
use sqlx::MySqlPool;
use sqlx::Row;
use std::fmt;

pub struct MySqlDatabase {
    pool: MySqlPool,
}

#[derive(Debug)]
pub enum BalanceError {
    Sql(sqlx::Error),
    InsufficientBalance,
    StakeNotFound,
    UserNotFound,
    InvalidAmount,
}

impl From<sqlx::Error> for BalanceError {
    fn from(error: sqlx::Error) -> Self {
        BalanceError::Sql(error)
    }
}

impl fmt::Display for BalanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BalanceError::Sql(err) => write!(f, "{}", err),
            BalanceError::InsufficientBalance => write!(f, "insufficient_balance"),
            BalanceError::StakeNotFound => write!(f, "stake_not_found"),
            BalanceError::UserNotFound => write!(f, "user_not_found"),
            BalanceError::InvalidAmount => write!(f, "invalid_amount"),
        }
    }
}

impl std::error::Error for BalanceError {}

impl MySqlDatabase {
    pub async fn new(connection_string: &str) -> Result<Self, sqlx::Error> {
        let pool = MySqlPool::connect(connection_string).await?;
        Ok(Self { pool })
    }

    /// Get a reference to the database pool
    pub fn get_pool(&self) -> &MySqlPool {
        &self.pool
    }

    pub async fn init_tables(&self) -> Result<(), sqlx::Error> {
        // Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id VARCHAR(255) PRIMARY KEY,
                username VARCHAR(255) UNIQUE NOT NULL,
                wallet_address VARCHAR(255) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create transactions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS transactions (
                id VARCHAR(255) PRIMARY KEY,
                from_user_id VARCHAR(255) NOT NULL,
                to_user_id VARCHAR(255) NOT NULL,
                amount DECIMAL(18, 8) NOT NULL,
                transaction_type VARCHAR(50) NOT NULL,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create airdrops table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS airdrops (
                id VARCHAR(255) PRIMARY KEY,
                total_amount DECIMAL(18, 8) NOT NULL,
                distributed_amount DECIMAL(18, 8) NOT NULL DEFAULT 0,
                start_time TIMESTAMP NULL,
                end_time TIMESTAMP NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create airdrop_recipients table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS airdrop_recipients (
                airdrop_id VARCHAR(255) NOT NULL,
                user_id VARCHAR(255) NOT NULL,
                amount DECIMAL(18, 8) NOT NULL,
                claimed BOOLEAN NOT NULL DEFAULT FALSE,
                claimed_at TIMESTAMP NULL,
                merkle_proof TEXT,
                category VARCHAR(100),
                PRIMARY KEY (airdrop_id, user_id),
                FOREIGN KEY (airdrop_id) REFERENCES airdrops(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create bridge_txs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bridge_txs (
                id VARCHAR(255) PRIMARY KEY,
                user_id VARCHAR(255) NOT NULL,
                token VARCHAR(64) NOT NULL,
                from_chain VARCHAR(64) NOT NULL,
                to_chain VARCHAR(64) NOT NULL,
                amount DECIMAL(18, 8) NOT NULL,
                lock_id VARCHAR(66) NULL,
                src_tx_hash VARCHAR(255) NULL,
                dst_tx_hash VARCHAR(255) NULL,
                status VARCHAR(32) NOT NULL,
                error_msg TEXT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create balances table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS balances (
                user_id VARCHAR(255) PRIMARY KEY,
                available_balance DECIMAL(18, 8) NOT NULL DEFAULT 0,
                staked_balance DECIMAL(18, 8) NOT NULL DEFAULT 0,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create stakes table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS stakes (
                id VARCHAR(255) PRIMARY KEY,
                user_id VARCHAR(255) NOT NULL,
                amount DECIMAL(18, 8) NOT NULL,
                duration_days INT NOT NULL,
                start_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                end_time TIMESTAMP NULL,
                status VARCHAR(32) NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create token_states table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS token_states (
                id INT AUTO_INCREMENT PRIMARY KEY,
                state_data JSON NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create token_transactions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS token_transactions (
                id VARCHAR(255) PRIMARY KEY,
                from_user_id VARCHAR(255) NOT NULL,
                to_user_id VARCHAR(255) NOT NULL,
                amount DECIMAL(18, 8) NOT NULL,
                transaction_type VARCHAR(50) NOT NULL,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                transaction_data JSON NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create token_events table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS token_events (
                id INT AUTO_INCREMENT PRIMARY KEY,
                event_type VARCHAR(100) NOT NULL,
                user_id VARCHAR(255) NOT NULL,
                amount DECIMAL(18, 8) NOT NULL,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                details TEXT,
                event_data JSON NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create staking_states table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS staking_states (
                id INT AUTO_INCREMENT PRIMARY KEY,
                state_data JSON NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create staking_infos table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS staking_infos (
                user_id VARCHAR(255) PRIMARY KEY,
                staking_data JSON NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create airdrop_states table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS airdrop_states (
                id INT AUTO_INCREMENT PRIMARY KEY,
                state_data JSON NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Airdrop related database operations
    pub async fn create_airdrop(
        &self,
        airdrop_id: &str,
        total_amount: f64,
        start_time: Option<NaiveDateTime>,
        end_time: Option<NaiveDateTime>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO airdrops (id, total_amount, start_time, end_time) VALUES (?, ?, ?, ?)",
        )
        .bind(airdrop_id)
        .bind(total_amount)
        .bind(start_time)
        .bind(end_time)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_airdrop_recipients(
        &self,
        airdrop_id: &str,
        recipients: &[(String, f64)],
        category: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        for (user_id, amount) in recipients {
            sqlx::query(
                "INSERT INTO airdrop_recipients (airdrop_id, user_id, amount, category) VALUES (?, ?, ?, ?)"
            )
            .bind(airdrop_id)
            .bind(user_id)
            .bind(amount)
            .bind(category)
            .execute(&self.pool)
            .await?;
        }

        // Update distributed amount in airdrop
        let total_new_amount: f64 = recipients.iter().map(|(_, amount)| amount).sum();
        sqlx::query("UPDATE airdrops SET distributed_amount = distributed_amount + ? WHERE id = ?")
            .bind(total_new_amount)
            .bind(airdrop_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn claim_airdrop(&self, airdrop_id: &str, user_id: &str) -> Result<f64, sqlx::Error> {
        // Get the amount to claim
        let row = sqlx::query(
            r#"SELECT amount FROM airdrop_recipients WHERE airdrop_id = ? AND user_id = ? AND claimed = FALSE"#
        )
        .bind(airdrop_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let amount: f64 = row.get("amount");

        // Update claim status
        sqlx::query(
            "UPDATE airdrop_recipients SET claimed = TRUE, claimed_at = ? WHERE airdrop_id = ? AND user_id = ?"
        )
        .bind(Utc::now().naive_utc())
        .bind(airdrop_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(amount)
    }

    pub async fn is_airdrop_claimed(
        &self,
        airdrop_id: &str,
        user_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query(
            r#"SELECT claimed FROM airdrop_recipients WHERE airdrop_id = ? AND user_id = ?"#,
        )
        .bind(airdrop_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let claimed: bool = row.get("claimed");
        Ok(claimed)
    }

    pub async fn get_airdrop_status(
        &self,
        airdrop_id: &str,
    ) -> Result<(f64, f64, usize, usize), sqlx::Error> {
        // Get airdrop info
        let airdrop_row =
            sqlx::query("SELECT total_amount, distributed_amount FROM airdrops WHERE id = ?")
                .bind(airdrop_id)
                .fetch_one(&self.pool)
                .await?;

        let total_amount: f64 = airdrop_row.get("total_amount");
        let distributed_amount: f64 = airdrop_row.get("distributed_amount");

        // Get recipient counts
        let recipient_row = sqlx::query(
            r#"SELECT COUNT(*) as total_recipients, SUM(claimed) as claimed_recipients FROM airdrop_recipients WHERE airdrop_id = ?"#
        )
        .bind(airdrop_id)
        .fetch_one(&self.pool)
        .await?;

        let total_recipients: i64 = recipient_row.get("total_recipients");
        let claimed_recipients: Option<i64> = recipient_row.get("claimed_recipients");

        Ok((
            total_amount,
            distributed_amount,
            total_recipients as usize,
            claimed_recipients.unwrap_or(0) as usize,
        ))
    }

    pub async fn batch_claim_airdrops(
        &self,
        airdrop_id: &str,
        user_ids: &[String],
    ) -> Result<Vec<(String, f64)>, sqlx::Error> {
        let mut claimed_amounts = Vec::new();

        for user_id in user_ids {
            match self.claim_airdrop(airdrop_id, user_id).await {
                Ok(amount) => claimed_amounts.push((user_id.clone(), amount)),
                Err(_) => continue, // Skip failed claims
            }
        }

        Ok(claimed_amounts)
    }
}

// User operations
impl MySqlDatabase {
    pub async fn create_user(
        &self,
        id: &str,
        username: &str,
        wallet_address: &str,
    ) -> Result<crate::models::User, sqlx::Error> {
        sqlx::query("INSERT INTO users (id, username, wallet_address) VALUES (?, ?, ?)")
            .bind(id)
            .bind(username)
            .bind(wallet_address)
            .execute(&self.pool)
            .await?;

        let row =
            sqlx::query("SELECT id, username, wallet_address, created_at FROM users WHERE id = ?")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        use crate::models::User;
        self.ensure_balance_row(id).await?;

        Ok(User {
            id: row.get("id"),
            username: row.get("username"),
            wallet_address: row.get("wallet_address"),
            created_at: row.get("created_at"),
        })
    }

    pub async fn ensure_balance_row(&self, user_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO balances (user_id) VALUES (?) ON DUPLICATE KEY UPDATE user_id = user_id",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_user(
        &self,
        id: &str,
        username: Option<&str>,
        wallet_address: Option<&str>,
    ) -> Result<Option<crate::models::User>, sqlx::Error> {
        let mut updates = Vec::new();
        let mut params = Vec::new();

        if let Some(username) = username {
            updates.push("username = ?");
            params.push(username);
        }

        if let Some(wallet) = wallet_address {
            updates.push("wallet_address = ?");
            params.push(wallet);
        }

        if updates.is_empty() {
            return Ok(None);
        }

        let query = format!("UPDATE users SET {} WHERE id = ?", updates.join(", "));
        params.push(id);

        let mut query_builder = sqlx::query(&query);
        for param in &params {
            query_builder = query_builder.bind(param);
        }
        let result = query_builder.execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        let row =
            sqlx::query("SELECT id, username, wallet_address, created_at FROM users WHERE id = ?")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        use crate::models::User;
        Ok(Some(User {
            id: row.get("id"),
            username: row.get("username"),
            wallet_address: row.get("wallet_address"),
            created_at: row.get("created_at"),
        }))
    }

    pub async fn get_user(&self, id: &str) -> Result<Option<crate::models::User>, sqlx::Error> {
        if let Some(row) =
            sqlx::query("SELECT id, username, wallet_address, created_at FROM users WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?
        {
            use crate::models::User;
            let user = User {
                id: row.get("id"),
                username: row.get("username"),
                wallet_address: row.get("wallet_address"),
                created_at: row.get("created_at"),
            };
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    pub async fn get_balances(&self, user_id: &str) -> Result<Option<(f64, f64)>, sqlx::Error> {
        if let Some(row) =
            sqlx::query("SELECT available_balance, staked_balance FROM balances WHERE user_id = ?")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?
        {
            let available: f64 = row.get("available_balance");
            let staked: f64 = row.get("staked_balance");
            Ok(Some((available, staked)))
        } else {
            Ok(None)
        }
    }
}

// Bridge transactions
impl MySqlDatabase {
    pub async fn insert_bridge_tx(
        &self,
        id: &str,
        user_id: &str,
        token: &str,
        from_chain: &str,
        to_chain: &str,
        amount: f64,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO bridge_txs (id, user_id, token, from_chain, to_chain, amount, status) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(id)
        .bind(user_id)
        .bind(token)
        .bind(from_chain)
        .bind(to_chain)
        .bind(amount)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_bridge_src_tx(
        &self,
        id: &str,
        src_tx_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE bridge_txs SET src_tx_hash = ? WHERE id = ?")
            .bind(src_tx_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_bridge_lock_id(&self, id: &str, lock_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE bridge_txs SET lock_id = ? WHERE id = ?")
            .bind(lock_id)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_bridge_dst_tx(
        &self,
        id: &str,
        dst_tx_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE bridge_txs SET dst_tx_hash = ? WHERE id = ?")
            .bind(dst_tx_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_bridge_status_row(
        &self,
        id: &str,
        status: &str,
        error_msg: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE bridge_txs SET status = ?, error_msg = ? WHERE id = ?")
            .bind(status)
            .bind(error_msg)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn fetch_bridge_tx(&self, id: &str) -> Result<crate::models::BridgeTx, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, user_id, token, from_chain, to_chain, amount, lock_id, src_tx_hash, dst_tx_hash, status, error_msg, created_at, updated_at FROM bridge_txs WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        use crate::models::{BridgeTx, BridgeTxStatus};

        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "Pending" => BridgeTxStatus::Pending,
            "Locked" => BridgeTxStatus::Locked,
            "Minted" => BridgeTxStatus::Minted,
            "Failed" => BridgeTxStatus::Failed,
            _ => BridgeTxStatus::Pending,
        };

        Ok(BridgeTx {
            id: row.get("id"),
            user_id: row.get("user_id"),
            token: row.get("token"),
            from_chain: row.get("from_chain"),
            to_chain: row.get("to_chain"),
            amount: row.get("amount"),
            lock_id: row.get("lock_id"),
            src_tx_hash: row.get("src_tx_hash"),
            dst_tx_hash: row.get("dst_tx_hash"),
            status,
            error_msg: row.get("error_msg"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn list_locked_without_dst(
        &self,
        from_chain: &str,
    ) -> Result<Vec<crate::models::BridgeTx>, sqlx::Error> {
        let rows = sqlx::query(
            r#"SELECT id, user_id, token, from_chain, to_chain, amount, lock_id, src_tx_hash, dst_tx_hash, status, error_msg, created_at, updated_at FROM bridge_txs WHERE status = 'Locked' AND from_chain = ? AND dst_tx_hash IS NULL"#
        )
        .bind(from_chain)
        .fetch_all(&self.pool)
        .await?;

        use crate::models::{BridgeTx, BridgeTxStatus};
        let mut out = Vec::new();
        for row in rows {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "Pending" => BridgeTxStatus::Pending,
                "Locked" => BridgeTxStatus::Locked,
                "Minted" => BridgeTxStatus::Minted,
                "Failed" => BridgeTxStatus::Failed,
                _ => BridgeTxStatus::Pending,
            };
            out.push(BridgeTx {
                id: row.get("id"),
                user_id: row.get("user_id"),
                token: row.get("token"),
                from_chain: row.get("from_chain"),
                to_chain: row.get("to_chain"),
                amount: row.get("amount"),
                lock_id: row.get("lock_id"),
                src_tx_hash: row.get("src_tx_hash"),
                dst_tx_hash: row.get("dst_tx_hash"),
                status,
                error_msg: row.get("error_msg"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        Ok(out)
    }
}

// Token contract state operations
impl MySqlDatabase {
    /// Save token contract state to database
    pub async fn save_token_state(&self, state_data: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO token_states (state_data) VALUES (?)")
            .bind(state_data)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Load latest token contract state from database
    pub async fn load_latest_token_state(&self) -> Result<Option<String>, sqlx::Error> {
        let row =
            sqlx::query("SELECT state_data FROM token_states ORDER BY created_at DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await?;

        if let Some(row) = row {
            let state_data: String = row.get("state_data");
            Ok(Some(state_data))
        } else {
            Ok(None)
        }
    }
}

// Staking contract state operations
impl MySqlDatabase {
    /// Save staking contract state to database
    pub async fn save_staking_state(&self, state_data: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO staking_states (state_data) VALUES (?)")
            .bind(state_data)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Load latest staking contract state from database
    pub async fn load_latest_staking_state(&self) -> Result<Option<String>, sqlx::Error> {
        let row =
            sqlx::query("SELECT state_data FROM staking_states ORDER BY created_at DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await?;

        if let Some(row) = row {
            let state_data: String = row.get("state_data");
            Ok(Some(state_data))
        } else {
            Ok(None)
        }
    }
}

// Airdrop contract state operations
impl MySqlDatabase {
    /// Save airdrop contract state to database
    pub async fn save_airdrop_state(&self, state_data: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO airdrop_states (state_data) VALUES (?)")
            .bind(state_data)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Load latest airdrop contract state from database
    pub async fn load_latest_airdrop_state(&self) -> Result<Option<String>, sqlx::Error> {
        let row =
            sqlx::query("SELECT state_data FROM airdrop_states ORDER BY created_at DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await?;

        if let Some(row) = row {
            let state_data: String = row.get("state_data");
            Ok(Some(state_data))
        } else {
            Ok(None)
        }
    }
}
