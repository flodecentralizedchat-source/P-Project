use p_project_core::database::MySqlDatabase;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<MySqlDatabase>,
}