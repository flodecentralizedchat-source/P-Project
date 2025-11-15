pub mod mongodb;
pub mod mysql;
pub mod redis;

pub use mysql::{BalanceError, MySqlDatabase};

use crate::models::Proposal;
use async_trait::async_trait;
use std::fmt;

#[derive(Debug)]
pub struct DatabaseError(String);

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DatabaseError {}

#[async_trait]
pub trait Database {
    type Error: std::error::Error + Send + Sync;

    async fn save_proposal(&self, proposal: &Proposal) -> Result<(), Self::Error>;
    async fn get_active_proposals(&self) -> Result<Vec<Proposal>, Self::Error>;
    async fn get_proposal(&self, proposal_id: &str) -> Result<Option<Proposal>, Self::Error>;
    async fn update_proposal(&self, proposal: &Proposal) -> Result<(), Self::Error>;
}

// In-memory database implementation for testing
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct InMemoryDatabase {
    proposals: Arc<RwLock<HashMap<String, Proposal>>>,
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        Self {
            proposals: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Database for InMemoryDatabase {
    type Error = DatabaseError;

    async fn save_proposal(&self, proposal: &Proposal) -> Result<(), Self::Error> {
        let mut proposals = self.proposals.write().await;
        proposals.insert(proposal.id.clone(), proposal.clone());
        Ok(())
    }

    async fn get_active_proposals(&self) -> Result<Vec<Proposal>, Self::Error> {
        let proposals = self.proposals.read().await;
        let active_proposals = proposals
            .values()
            .filter(|p| matches!(p.status, crate::models::ProposalStatus::Active))
            .cloned()
            .collect();
        Ok(active_proposals)
    }

    async fn get_proposal(&self, proposal_id: &str) -> Result<Option<Proposal>, Self::Error> {
        let proposals = self.proposals.read().await;
        Ok(proposals.get(proposal_id).cloned())
    }

    async fn update_proposal(&self, proposal: &Proposal) -> Result<(), Self::Error> {
        let mut proposals = self.proposals.write().await;
        proposals.insert(proposal.id.clone(), proposal.clone());
        Ok(())
    }
}
