use sqlx::MySqlPool;
use sqlx::Row;
use chrono::{NaiveDateTime, Utc};

pub struct MySqlDatabase {
    pool: MySqlPool,
}

impl MySqlDatabase {
    pub async fn new(connection_string: &str) -> Result<Self, sqlx::Error> {
        let pool = MySqlPool::connect(connection_string).await?;
        Ok(Self { pool })
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
            "#
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
            "#
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
            "#
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
            "#
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
                src_tx_hash VARCHAR(255) NULL,
                dst_tx_hash VARCHAR(255) NULL,
                status VARCHAR(32) NOT NULL,
                error_msg TEXT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    
    // Airdrop related database operations
    pub async fn create_airdrop(&self, airdrop_id: &str, total_amount: f64, start_time: Option<NaiveDateTime>, end_time: Option<NaiveDateTime>) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO airdrops (id, total_amount, start_time, end_time) VALUES (?, ?, ?, ?)"
        )
        .bind(airdrop_id)
        .bind(total_amount)
        .bind(start_time)
        .bind(end_time)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn add_airdrop_recipients(&self, airdrop_id: &str, recipients: &[(String, f64)], category: Option<&str>) -> Result<(), sqlx::Error> {
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
        sqlx::query(
            "UPDATE airdrops SET distributed_amount = distributed_amount + ? WHERE id = ?"
        )
        .bind(total_new_amount)
        .bind(airdrop_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn claim_airdrop(&self, airdrop_id: &str, user_id: &str) -> Result<f64, sqlx::Error> {
        // Get the amount to claim
        let row = sqlx::query(
            "SELECT amount FROM airdrop_recipients WHERE airdrop_id = ? AND user_id = ? AND claimed = FALSE"
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
    
    pub async fn is_airdrop_claimed(&self, airdrop_id: &str, user_id: &str) -> Result<bool, sqlx::Error> {
        let row = sqlx::query(
            "SELECT claimed FROM airdrop_recipients WHERE airdrop_id = ? AND user_id = ?"
        )
        .bind(airdrop_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        
        let claimed: bool = row.get("claimed");
        Ok(claimed)
    }
    
    pub async fn get_airdrop_status(&self, airdrop_id: &str) -> Result<(f64, f64, usize, usize), sqlx::Error> {
        // Get airdrop info
        let airdrop_row = sqlx::query(
            "SELECT total_amount, distributed_amount FROM airdrops WHERE id = ?"
        )
        .bind(airdrop_id)
        .fetch_one(&self.pool)
        .await?;
        
        let total_amount: f64 = airdrop_row.get("total_amount");
        let distributed_amount: f64 = airdrop_row.get("distributed_amount");
        
        // Get recipient counts
        let recipient_row = sqlx::query(
            "SELECT COUNT(*) as total_recipients, SUM(claimed) as claimed_recipients FROM airdrop_recipients WHERE airdrop_id = ?"
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
            claimed_recipients.unwrap_or(0) as usize
        ))
    }
    
    pub async fn batch_claim_airdrops(&self, airdrop_id: &str, user_ids: &[String]) -> Result<Vec<(String, f64)>, sqlx::Error> {
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
    pub async fn create_user(&self, id: &str, username: &str, wallet_address: &str) -> Result<crate::models::User, sqlx::Error> {
        sqlx::query(
            "INSERT INTO users (id, username, wallet_address) VALUES (?, ?, ?)"
        )
        .bind(id)
        .bind(username)
        .bind(wallet_address)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query(
            "SELECT id, username, wallet_address, created_at FROM users WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        use crate::models::User;
        Ok(User {
            id: row.get("id"),
            username: row.get("username"),
            wallet_address: row.get("wallet_address"),
            created_at: row.get("created_at"),
        })
    }

    pub async fn get_user(&self, id: &str) -> Result<Option<crate::models::User>, sqlx::Error> {
        if let Some(row) = sqlx::query(
            "SELECT id, username, wallet_address, created_at FROM users WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await? {
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
}

// Bridge transactions
impl MySqlDatabase {
    pub async fn create_bridge_tx(
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

    pub async fn set_bridge_src_tx(&self, id: &str, src_tx_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE bridge_txs SET src_tx_hash = ? WHERE id = ?")
            .bind(src_tx_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_bridge_dst_tx(&self, id: &str, dst_tx_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE bridge_txs SET dst_tx_hash = ? WHERE id = ?")
            .bind(dst_tx_hash)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_bridge_status(&self, id: &str, status: &str, error_msg: Option<&str>) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE bridge_txs SET status = ?, error_msg = ? WHERE id = ?")
            .bind(status)
            .bind(error_msg)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_bridge_tx(&self, id: &str) -> Result<crate::models::BridgeTx, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, user_id, token, from_chain, to_chain, amount, src_tx_hash, dst_tx_hash, status, error_msg, created_at, updated_at FROM bridge_txs WHERE id = ?"
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
            amount: row.get::<f64, _>("amount"),
            src_tx_hash: row.get::<Option<String>, _>("src_tx_hash"),
            dst_tx_hash: row.get::<Option<String>, _>("dst_tx_hash"),
            status,
            error_msg: row.get::<Option<String>, _>("error_msg"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    pub async fn list_bridge_locked_without_dst(&self, from_chain: &str) -> Result<Vec<crate::models::BridgeTx>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, user_id, token, from_chain, to_chain, amount, src_tx_hash, dst_tx_hash, status, error_msg, created_at, updated_at FROM bridge_txs WHERE status = 'Locked' AND from_chain = ? AND (dst_tx_hash IS NULL OR dst_tx_hash = '')"
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
                amount: row.get::<f64, _>("amount"),
                src_tx_hash: row.get::<Option<String>, _>("src_tx_hash"),
                dst_tx_hash: row.get::<Option<String>, _>("dst_tx_hash"),
                status,
                error_msg: row.get::<Option<String>, _>("error_msg"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        Ok(out)
    }
}
