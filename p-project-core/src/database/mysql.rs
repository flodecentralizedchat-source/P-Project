use sqlx::MySqlPool;

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
        
        Ok(())
    }
}