use chrono::{NaiveDateTime, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use sqlx::MySqlPool;
use sqlx::Row;
use std::fmt;
use uuid::Uuid;

use crate::budget_alternatives::{BudgetAlternative, BudgetStrategy};
use crate::models::{
    LearningActivityType, LearningCompletion, LearningContent, LearningContentType, TransactionType,
};
use crate::utils::generate_id;

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
    AlreadyCompleted,
    LearningContentNotFound,
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
            BalanceError::AlreadyCompleted => write!(f, "already_completed"),
            BalanceError::LearningContentNotFound => write!(f, "learning_content_not_found"),
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

        // Create learning_content table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS learning_content (
                id VARCHAR(255) PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                description TEXT NOT NULL,
                content_type VARCHAR(64) NOT NULL,
                reward_tokens DECIMAL(18, 8) NOT NULL,
                reward_points INT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create learning_completions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS learning_completions (
                id VARCHAR(255) PRIMARY KEY,
                user_id VARCHAR(255) NOT NULL,
                content_id VARCHAR(255) NOT NULL,
                activity_type VARCHAR(64) NOT NULL,
                reward_tokens DECIMAL(18, 8) NOT NULL,
                reward_points INT NOT NULL,
                proof_reference VARCHAR(255) NULL,
                completed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE KEY uniq_user_content (user_id, content_id),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (content_id) REFERENCES learning_content(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create referral_codes table (one code per user, code unique)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS referral_codes (
                code VARCHAR(64) PRIMARY KEY,
                user_id VARCHAR(255) UNIQUE NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create referrals table (each referred user may only be referred once)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS referrals (
                id VARCHAR(255) PRIMARY KEY,
                referrer_user_id VARCHAR(255) NOT NULL,
                referred_user_id VARCHAR(255) UNIQUE NOT NULL,
                code VARCHAR(64) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (referrer_user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (referred_user_id) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (code) REFERENCES referral_codes(code) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create community events table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id VARCHAR(255) PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                description TEXT NOT NULL,
                event_type VARCHAR(64) NOT NULL,
                scheduled_start TIMESTAMP NOT NULL,
                scheduled_end TIMESTAMP NULL,
                link VARCHAR(255) NULL,
                created_by VARCHAR(255) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create budget alternatives table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS budget_alternatives (
                id VARCHAR(255) PRIMARY KEY,
                option_name VARCHAR(255) NOT NULL,
                details TEXT NOT NULL,
                strategy VARCHAR(64) NOT NULL,
                start_amount DECIMAL(18, 8) NOT NULL,
                growth_rate DECIMAL(9, 8) NOT NULL,
                duration_months INT NOT NULL DEFAULT 12,
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
        total_amount: Decimal,
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
        recipients: &[(String, Decimal)],
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
        let total_new_amount: Decimal = recipients
            .iter()
            .fold(Decimal::ZERO, |acc, (_, a)| acc + a.clone());
        sqlx::query("UPDATE airdrops SET distributed_amount = distributed_amount + ? WHERE id = ?")
            .bind(total_new_amount)
            .bind(airdrop_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn claim_airdrop(
        &self,
        airdrop_id: &str,
        user_id: &str,
    ) -> Result<Decimal, sqlx::Error> {
        // Get the amount to claim
        let row = sqlx::query(
            r#"SELECT amount FROM airdrop_recipients WHERE airdrop_id = ? AND user_id = ? AND claimed = FALSE"#
        )
        .bind(airdrop_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let amount: Decimal = row.get("amount");

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
    ) -> Result<(Decimal, Decimal, usize, usize), sqlx::Error> {
        // Get airdrop info
        let airdrop_row =
            sqlx::query("SELECT total_amount, distributed_amount FROM airdrops WHERE id = ?")
                .bind(airdrop_id)
                .fetch_one(&self.pool)
                .await?;

        let total_amount: Decimal = airdrop_row.get("total_amount");
        let distributed_amount: Decimal = airdrop_row.get("distributed_amount");

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
    ) -> Result<Vec<(String, Decimal)>, sqlx::Error> {
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

    pub async fn get_balances(
        &self,
        user_id: &str,
    ) -> Result<Option<(Decimal, Decimal)>, sqlx::Error> {
        if let Some(row) =
            sqlx::query("SELECT available_balance, staked_balance FROM balances WHERE user_id = ?")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?
        {
            let available: Decimal = row.get("available_balance");
            let staked: Decimal = row.get("staked_balance");
            Ok(Some((available, staked)))
        } else {
            Ok(None)
        }
    }

    // ---------------- Learning (Quests) ----------------
    pub async fn register_learning_content(
        &self,
        content_id: &str,
        title: &str,
        description: &str,
        content_type: LearningContentType,
        reward_tokens: Decimal,
        reward_points: i64,
    ) -> Result<LearningContent, sqlx::Error> {
        let content_type_str = match content_type {
            LearningContentType::Course => "Course",
            LearningContentType::Quiz => "Quiz",
            LearningContentType::Workshop => "Workshop",
        };
        sqlx::query(
            r#"INSERT INTO learning_content (id, title, description, content_type, reward_tokens, reward_points)
               VALUES (?, ?, ?, ?, ?, ?)"#,
        )
        .bind(content_id)
        .bind(title)
        .bind(description)
        .bind(content_type_str)
        .bind(reward_tokens)
        .bind(reward_points)
        .execute(&self.pool)
        .await?;

        let row = sqlx::query(
            r#"SELECT id, title, description, content_type, reward_tokens, reward_points, created_at
               FROM learning_content WHERE id = ?"#,
        )
        .bind(content_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(LearningContent {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            content_type: match row.get::<String, _>("content_type").as_str() {
                "Course" => LearningContentType::Course,
                "Quiz" => LearningContentType::Quiz,
                _ => LearningContentType::Workshop,
            },
            reward_tokens: row.get("reward_tokens"),
            reward_points: row.get("reward_points"),
            created_at: row.get("created_at"),
        })
    }

    pub async fn list_learning_content(
        &self,
        limit: i64,
    ) -> Result<Vec<LearningContent>, sqlx::Error> {
        let rows = sqlx::query(
            r#"SELECT id, title, description, content_type, reward_tokens, reward_points, created_at
               FROM learning_content ORDER BY created_at DESC LIMIT ?"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::new();
        for row in rows {
            out.push(LearningContent {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                content_type: match row.get::<String, _>("content_type").as_str() {
                    "Course" => LearningContentType::Course,
                    "Quiz" => LearningContentType::Quiz,
                    _ => LearningContentType::Workshop,
                },
                reward_tokens: row.get("reward_tokens"),
                reward_points: row.get("reward_points"),
                created_at: row.get("created_at"),
            });
        }
        Ok(out)
    }

    pub async fn get_learning_content(
        &self,
        content_id: &str,
    ) -> Result<Option<LearningContent>, sqlx::Error> {
        if let Some(row) = sqlx::query(
            r#"SELECT id, title, description, content_type, reward_tokens, reward_points, created_at
               FROM learning_content WHERE id = ?"#,
        )
        .bind(content_id)
        .fetch_optional(&self.pool)
        .await?
        {
            Ok(Some(LearningContent {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                content_type: match row.get::<String, _>("content_type").as_str() {
                    "Course" => LearningContentType::Course,
                    "Quiz" => LearningContentType::Quiz,
                    _ => LearningContentType::Workshop,
                },
                reward_tokens: row.get("reward_tokens"),
                reward_points: row.get("reward_points"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn record_learning_completion(
        &self,
        completion_id: &str,
        user_id: &str,
        content_id: &str,
        proof_reference: Option<&str>,
        reward_source: Option<&str>,
    ) -> Result<LearningCompletion, BalanceError> {
        // Ensure content exists
        let content_row = sqlx::query(
            r#"SELECT id, reward_tokens, reward_points FROM learning_content WHERE id = ?"#,
        )
        .bind(content_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(BalanceError::Sql)?;

        let Some(content_row) = content_row else {
            return Err(BalanceError::LearningContentNotFound);
        };

        // Check if already completed
        let exists = sqlx::query(
            r#"SELECT 1 FROM learning_completions WHERE user_id = ? AND content_id = ?"#,
        )
        .bind(user_id)
        .bind(content_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(BalanceError::Sql)?
        .is_some();

        if exists {
            return Err(BalanceError::AlreadyCompleted);
        }

        let reward_tokens: Decimal = content_row.get("reward_tokens");
        let reward_points: i64 = content_row.get("reward_points");

        // Start a transaction for atomicity
        let mut tx = self.pool.begin().await.map_err(BalanceError::Sql)?;

        // Insert completion record
        sqlx::query(
            r#"INSERT INTO learning_completions
                (id, user_id, content_id, activity_type, reward_tokens, reward_points, proof_reference)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(completion_id)
        .bind(user_id)
        .bind(content_id)
        .bind("CourseCompletion")
        .bind(reward_tokens)
        .bind(reward_points)
        .bind(proof_reference)
        .execute(&mut *tx)
        .await
        .map_err(BalanceError::Sql)?;

        // Ensure balance row exists
        sqlx::query(
            "INSERT INTO balances (user_id) VALUES (?) ON DUPLICATE KEY UPDATE user_id = user_id",
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(BalanceError::Sql)?;

        // Credit reward tokens to user
        sqlx::query(
            "UPDATE balances SET available_balance = available_balance + ? WHERE user_id = ?",
        )
        .bind(reward_tokens)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(BalanceError::Sql)?;

        // Record transaction for reward
        let tx_id = generate_id();
        let from_user = reward_source.unwrap_or("system");
        let tx_type = "reward";
        sqlx::query(
            r#"INSERT INTO transactions (id, from_user_id, to_user_id, amount, transaction_type)
               VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(&tx_id)
        .bind(from_user)
        .bind(user_id)
        .bind(reward_tokens)
        .bind(tx_type)
        .execute(&mut *tx)
        .await
        .map_err(BalanceError::Sql)?;

        // Referral bonus: credit 5% to referrer if exists
        let referrer_row =
            sqlx::query(r#"SELECT referrer_user_id FROM referrals WHERE referred_user_id = ?"#)
                .bind(user_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(BalanceError::Sql)?;

        if let Some(row) = referrer_row {
            let referrer_id: String = row.get("referrer_user_id");
            // Ensure balance row exists
            sqlx::query(
                "INSERT INTO balances (user_id) VALUES (?) ON DUPLICATE KEY UPDATE user_id = user_id",
            )
            .bind(&referrer_id)
            .execute(&mut *tx)
            .await
            .map_err(BalanceError::Sql)?;

            // 5% bonus
            let bonus =
                reward_tokens * rust_decimal::Decimal::from_f64(0.05).unwrap_or(Decimal::ZERO);
            if bonus > Decimal::ZERO {
                sqlx::query(
                    "UPDATE balances SET available_balance = available_balance + ? WHERE user_id = ?",
                )
                .bind(bonus)
                .bind(&referrer_id)
                .execute(&mut *tx)
                .await
                .map_err(BalanceError::Sql)?;

                let bonus_tx_id = generate_id();
                sqlx::query(
                    r#"INSERT INTO transactions (id, from_user_id, to_user_id, amount, transaction_type)
                       VALUES (?, ?, ?, ?, ?)"#,
                )
                .bind(&bonus_tx_id)
                .bind("referral_bonus")
                .bind(&referrer_id)
                .bind(bonus)
                .bind("reward")
                .execute(&mut *tx)
                .await
                .map_err(BalanceError::Sql)?;
            }
        }

        // Commit transaction
        tx.commit().await.map_err(BalanceError::Sql)?;

        Ok(LearningCompletion {
            id: completion_id.to_string(),
            user_id: user_id.to_string(),
            content_id: content_id.to_string(),
            activity_type: LearningActivityType::CourseCompletion,
            reward_tokens,
            reward_points,
            proof_reference: proof_reference.map(|s| s.to_string()),
            completed_at: chrono::Utc::now().naive_utc(),
        })
    }

    pub async fn list_user_learning_completions(
        &self,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<LearningCompletion>, sqlx::Error> {
        let rows = sqlx::query(
            r#"SELECT id, user_id, content_id, activity_type, reward_tokens, reward_points, proof_reference, completed_at
               FROM learning_completions WHERE user_id = ? ORDER BY completed_at DESC LIMIT ?"#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        let mut out = Vec::new();
        for row in rows {
            out.push(LearningCompletion {
                id: row.get("id"),
                user_id: row.get("user_id"),
                content_id: row.get("content_id"),
                activity_type: match row.get::<String, _>("activity_type").as_str() {
                    "CourseCompletion" => LearningActivityType::CourseCompletion,
                    "QuizCompletion" => LearningActivityType::QuizCompletion,
                    _ => LearningActivityType::WorkshopParticipation,
                },
                reward_tokens: row.get("reward_tokens"),
                reward_points: row.get("reward_points"),
                proof_reference: row.get("proof_reference"),
                completed_at: row.get("completed_at"),
            });
        }
        Ok(out)
    }

    // ---------------- Referrals ----------------
    pub async fn upsert_referral_code(&self, user_id: &str) -> Result<String, sqlx::Error> {
        // If already exists, return it
        if let Some(row) = sqlx::query("SELECT code FROM referral_codes WHERE user_id = ?")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
        {
            let code: String = row.get("code");
            return Ok(code);
        }

        // Generate unique code and insert
        for _ in 0..5 {
            let code = Uuid::new_v4()
                .to_string()
                .chars()
                .take(8)
                .collect::<String>()
                .to_uppercase();
            let res = sqlx::query("INSERT INTO referral_codes (code, user_id) VALUES (?, ?)")
                .bind(&code)
                .bind(user_id)
                .execute(&self.pool)
                .await;
            match res {
                Ok(_) => return Ok(code),
                Err(e) => {
                    // On duplicate, retry
                    if let sqlx::Error::Database(db_err) = &e {
                        if db_err.code().map(|c| c.to_string()) == Some("23000".into()) {
                            continue;
                        }
                    }
                    return Err(e);
                }
            }
        }
        // Fallback one more time without checking
        let code = Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>()
            .to_uppercase();
        sqlx::query("INSERT INTO referral_codes (code, user_id) VALUES (?, ?)")
            .bind(&code)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(code)
    }

    pub async fn get_referral_code(&self, user_id: &str) -> Result<Option<String>, sqlx::Error> {
        Ok(
            sqlx::query("SELECT code FROM referral_codes WHERE user_id = ?")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?
                .map(|row| row.get("code")),
        )
    }

    pub async fn accept_referral(
        &self,
        code: &str,
        referred_user_id: &str,
    ) -> Result<(), BalanceError> {
        // Resolve referrer by code
        let row = sqlx::query("SELECT user_id FROM referral_codes WHERE code = ?")
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(BalanceError::Sql)?;

        let Some(row) = row else {
            return Err(BalanceError::UserNotFound);
        };
        let referrer_user_id: String = row.get("user_id");
        if referrer_user_id == referred_user_id {
            return Err(BalanceError::InvalidAmount);
        }

        // Insert if not already referred
        let exists = sqlx::query("SELECT 1 FROM referrals WHERE referred_user_id = ?")
            .bind(referred_user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(BalanceError::Sql)?
            .is_some();
        if exists {
            return Ok(());
        }

        let id = generate_id();
        sqlx::query(
            "INSERT INTO referrals (id, referrer_user_id, referred_user_id, code) VALUES (?, ?, ?, ?)",
        )
        .bind(id)
        .bind(referrer_user_id)
        .bind(referred_user_id)
        .bind(code)
        .execute(&self.pool)
        .await
        .map_err(BalanceError::Sql)?;
        Ok(())
    }

    pub async fn get_referral_stats(&self, user_id: &str) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT COUNT(*) AS cnt FROM referrals WHERE referrer_user_id = ?")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;
        let cnt: i64 = row.get("cnt");
        Ok(cnt)
    }

    // ---------------- Community Events (AMAs & Events) ----------------
    pub async fn create_event(
        &self,
        id: &str,
        title: &str,
        description: &str,
        event_type: &str,
        scheduled_start: chrono::NaiveDateTime,
        scheduled_end: Option<chrono::NaiveDateTime>,
        link: Option<&str>,
        created_by: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO events (id, title, description, event_type, scheduled_start, scheduled_end, link, created_by)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(id)
        .bind(title)
        .bind(description)
        .bind(event_type)
        .bind(scheduled_start)
        .bind(scheduled_end)
        .bind(link)
        .bind(created_by)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_events(
        &self,
        limit: i64,
    ) -> Result<
        Vec<(
            String,
            String,
            String,
            String,
            chrono::NaiveDateTime,
            Option<chrono::NaiveDateTime>,
            Option<String>,
            String,
            chrono::NaiveDateTime,
        )>,
        sqlx::Error,
    > {
        let rows = sqlx::query(
            r#"SELECT id, title, description, event_type, scheduled_start, scheduled_end, link, created_by, created_at
               FROM events ORDER BY scheduled_start DESC LIMIT ?"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        let mut out = Vec::new();
        for row in rows {
            out.push((
                row.get("id"),
                row.get("title"),
                row.get("description"),
                row.get("event_type"),
                row.get("scheduled_start"),
                row.get("scheduled_end"),
                row.get("link"),
                row.get("created_by"),
                row.get("created_at"),
            ));
        }
        Ok(out)
    }

    pub async fn get_event(
        &self,
        id: &str,
    ) -> Result<
        Option<(
            String,
            String,
            String,
            String,
            chrono::NaiveDateTime,
            Option<chrono::NaiveDateTime>,
            Option<String>,
            String,
            chrono::NaiveDateTime,
        )>,
        sqlx::Error,
    > {
        if let Some(row) = sqlx::query(
            r#"SELECT id, title, description, event_type, scheduled_start, scheduled_end, link, created_by, created_at
               FROM events WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        {
            Ok(Some((
                row.get("id"),
                row.get("title"),
                row.get("description"),
                row.get("event_type"),
                row.get("scheduled_start"),
                row.get("scheduled_end"),
                row.get("link"),
                row.get("created_by"),
                row.get("created_at"),
            )))
        } else {
            Ok(None)
        }
    }

    // ---------------- Budget Alternatives ----------------
    pub async fn create_budget_alternative(
        &self,
        id: &str,
        option_name: &str,
        details: &str,
        strategy: BudgetStrategy,
        start_amount: Decimal,
        growth_rate: Decimal,
        duration_months: i64,
    ) -> Result<BudgetAlternative, sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO budget_alternatives (id, option_name, details, strategy, start_amount, growth_rate, duration_months)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(id)
        .bind(option_name)
        .bind(details)
        .bind(strategy.as_str())
        .bind(start_amount)
        .bind(growth_rate)
        .bind(duration_months)
        .execute(&self.pool)
        .await?;

        let alternative = self.get_budget_alternative(id).await?;
        alternative.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn list_budget_alternatives(
        &self,
        limit: i64,
    ) -> Result<Vec<BudgetAlternative>, sqlx::Error> {
        let rows = sqlx::query(
            r#"SELECT id, option_name, details, strategy, start_amount, growth_rate, duration_months, created_at
               FROM budget_alternatives ORDER BY created_at DESC LIMIT ?"#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::new();
        for row in rows {
            out.push(BudgetAlternative {
                id: row.get("id"),
                option_name: row.get("option_name"),
                details: row.get("details"),
                strategy: BudgetStrategy::from_str(&row.get::<String, _>("strategy")),
                start_amount: row.get("start_amount"),
                growth_rate: row.get("growth_rate"),
                duration_months: row.get("duration_months"),
                created_at: row.get("created_at"),
            });
        }
        Ok(out)
    }

    pub async fn get_budget_alternative(
        &self,
        id: &str,
    ) -> Result<Option<BudgetAlternative>, sqlx::Error> {
        if let Some(row) = sqlx::query(
            r#"SELECT id, option_name, details, strategy, start_amount, growth_rate, duration_months, created_at
               FROM budget_alternatives WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        {
            Ok(Some(BudgetAlternative {
                id: row.get("id"),
                option_name: row.get("option_name"),
                details: row.get("details"),
                strategy: BudgetStrategy::from_str(&row.get::<String, _>("strategy")),
                start_amount: row.get("start_amount"),
                growth_rate: row.get("growth_rate"),
                duration_months: row.get("duration_months"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Transfer tokens between users
    pub async fn transfer_tokens(
        &self,
        transaction_id: &str,
        from_user_id: &str,
        to_user_id: &str,
        amount: Decimal,
        transaction_type: crate::models::TransactionType,
    ) -> Result<(), BalanceError> {
        if amount <= Decimal::ZERO {
            return Err(BalanceError::InvalidAmount);
        }

        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(BalanceError::Sql)?;

        // Check if from_user exists and has sufficient balance
        let from_balance_row =
            sqlx::query("SELECT available_balance FROM balances WHERE user_id = ? FOR UPDATE")
                .bind(from_user_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(BalanceError::Sql)?;

        let from_balance: Decimal = match from_balance_row {
            Some(row) => row.get::<Decimal, _>("available_balance"),
            None => return Err(BalanceError::UserNotFound),
        };

        if from_balance < amount {
            return Err(BalanceError::InsufficientBalance);
        }

        // Check if to_user exists
        let to_user_exists = sqlx::query("SELECT 1 FROM users WHERE id = ?")
            .bind(to_user_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(BalanceError::Sql)?
            .is_some();

        if !to_user_exists {
            return Err(BalanceError::UserNotFound);
        }

        // Update balances
        sqlx::query(
            "UPDATE balances SET available_balance = available_balance - ? WHERE user_id = ?",
        )
        .bind(amount)
        .bind(from_user_id)
        .execute(&mut *tx)
        .await
        .map_err(BalanceError::Sql)?;

        sqlx::query(
            "UPDATE balances SET available_balance = available_balance + ? WHERE user_id = ?",
        )
        .bind(amount)
        .bind(to_user_id)
        .execute(&mut *tx)
        .await
        .map_err(BalanceError::Sql)?;

        // Record transaction
        let transaction_type_str = match transaction_type {
            crate::models::TransactionType::Transfer => "transfer",
            crate::models::TransactionType::Burn => "burn",
            crate::models::TransactionType::Reward => "reward",
            crate::models::TransactionType::Staking => "staking",
        };

        sqlx::query(
            "INSERT INTO transactions (id, from_user_id, to_user_id, amount, transaction_type) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(transaction_id)
        .bind(from_user_id)
        .bind(to_user_id)
        .bind(amount)
        .bind(transaction_type_str)
        .execute(&mut *tx)
        .await
        .map_err(BalanceError::Sql)?;

        // Commit transaction
        tx.commit().await.map_err(BalanceError::Sql)?;

        Ok(())
    }

    /// Stake tokens for a user
    pub async fn stake_tokens(
        &self,
        stake_id: &str,
        user_id: &str,
        amount: Decimal,
        duration_days: i64,
    ) -> Result<crate::models::StakingInfo, BalanceError> {
        if amount <= Decimal::ZERO {
            return Err(BalanceError::InvalidAmount);
        }

        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(BalanceError::Sql)?;

        // Check if user exists and has sufficient balance
        let user_balance_row =
            sqlx::query("SELECT available_balance FROM balances WHERE user_id = ? FOR UPDATE")
                .bind(user_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(BalanceError::Sql)?;

        let available_balance: Decimal = match user_balance_row {
            Some(row) => row.get::<Decimal, _>("available_balance"),
            None => return Err(BalanceError::UserNotFound),
        };

        if available_balance < amount {
            return Err(BalanceError::InsufficientBalance);
        }

        // Update balances
        sqlx::query(
            "UPDATE balances SET available_balance = available_balance - ?, staked_balance = staked_balance + ? WHERE user_id = ?"
        )
        .bind(amount)
        .bind(amount)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(BalanceError::Sql)?;

        // Insert stake record
        let start_time = Utc::now().naive_utc();
        let end_time = start_time + chrono::Duration::days(duration_days);

        sqlx::query(
            "INSERT INTO stakes (id, user_id, amount, duration_days, start_time, end_time, status) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(stake_id)
        .bind(user_id)
        .bind(amount)
        .bind(duration_days)
        .bind(start_time)
        .bind(end_time)
        .bind("active") // status
        .execute(&mut *tx)
        .await
        .map_err(BalanceError::Sql)?;

        // Commit transaction
        tx.commit().await.map_err(BalanceError::Sql)?;

        // Return staking info
        Ok(crate::models::StakingInfo {
            user_id: user_id.to_string(),
            amount,
            start_time,
            end_time: Some(end_time),
            rewards_earned: Decimal::ZERO,
            tier_name: None,
            is_compounding: false,
        })
    }

    /// Unstake tokens for a user
    pub async fn unstake_tokens(
        &self,
        user_id: &str,
        stake_id: Option<&str>,
    ) -> Result<crate::models::StakingInfo, BalanceError> {
        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(BalanceError::Sql)?;

        // Get stake record
        let stake_query = if stake_id.is_some() {
            "SELECT id, amount, start_time, end_time FROM stakes WHERE user_id = ? AND id = ? AND status = 'active' FOR UPDATE"
        } else {
            "SELECT id, amount, start_time, end_time FROM stakes WHERE user_id = ? AND status = 'active' FOR UPDATE"
        };

        let stake_row = if let Some(stake_id) = stake_id {
            sqlx::query(stake_query)
                .bind(user_id)
                .bind(stake_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(BalanceError::Sql)?
        } else {
            sqlx::query(stake_query)
                .bind(user_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(BalanceError::Sql)?
        };

        let stake_data = match stake_row {
            Some(row) => {
                let id: String = row.get("id");
                let amount: Decimal = row.get("amount");
                let start_time: NaiveDateTime = row.get("start_time");
                let end_time: Option<NaiveDateTime> = row.get("end_time");
                (id, amount, start_time, end_time)
            }
            None => return Err(BalanceError::StakeNotFound),
        };

        let (stake_id_val, stake_amount, start_time, _end_time) = stake_data;

        // Calculate rewards (simplified - in a real implementation, this would be more complex)
        let rewards = Decimal::ZERO; // For now, no rewards

        // Update balances
        let total_return = stake_amount + rewards;
        sqlx::query(
            "UPDATE balances SET available_balance = available_balance + ?, staked_balance = staked_balance - ? WHERE user_id = ?"
        )
        .bind(total_return)
        .bind(stake_amount)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(BalanceError::Sql)?;

        // Update stake record
        let end_time_now = Utc::now().naive_utc();
        sqlx::query("UPDATE stakes SET status = 'completed', end_time = ? WHERE id = ?")
            .bind(end_time_now)
            .bind(stake_id_val)
            .execute(&mut *tx)
            .await
            .map_err(BalanceError::Sql)?;

        // Commit transaction
        tx.commit().await.map_err(BalanceError::Sql)?;

        // Return staking info
        Ok(crate::models::StakingInfo {
            user_id: user_id.to_string(),
            amount: total_return,
            start_time,
            end_time: Some(end_time_now),
            rewards_earned: rewards,
            tier_name: None,
            is_compounding: false,
        })
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
