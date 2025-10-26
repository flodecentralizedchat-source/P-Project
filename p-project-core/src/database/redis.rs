use redis::{Client, AsyncCommands};

pub struct RedisCache {
    client: Client,
}

impl RedisCache {
    pub async fn new(connection_string: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(connection_string)?;
        Ok(Self { client })
    }
    
    pub async fn cache_user_session(&self, user_id: &str, session_token: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        conn.set_ex::<&str, &str, ()>(session_token, user_id, 3600).await?; // 1 hour expiry
        Ok(())
    }
    
    pub async fn get_user_from_session(&self, session_token: &str) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        let user_id: Option<String> = conn.get(session_token).await?;
        Ok(user_id)
    }
    
    pub async fn invalidate_session(&self, session_token: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        conn.del::<&str, ()>(session_token).await?;
        Ok(())
    }
}