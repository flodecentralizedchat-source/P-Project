//! Database adapters for connecting contracts with database layers

use p_project_core::database::MySqlDatabase;
use p_project_core::mongodb::MongoDatabase;
use p_project_core::redis::RedisCache;
use std::sync::Arc;

/// Database manager that holds connections to all three database types
pub struct DatabaseManager {
    pub mysql: Arc<MySqlDatabase>,
    pub redis: Arc<RedisCache>,
    pub mongodb: Arc<MongoDatabase>,
}

impl DatabaseManager {
    pub fn new(
        mysql: Arc<MySqlDatabase>,
        redis: Arc<RedisCache>,
        mongodb: Arc<MongoDatabase>,
    ) -> Self {
        Self {
            mysql,
            redis,
            mongodb,
        }
    }
}
