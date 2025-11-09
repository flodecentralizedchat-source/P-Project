use async_trait::async_trait;
use p_project_contracts::token::PProjectToken;
use p_project_core::{
    database::mongodb::MongoDatabase,
    models::{Proposal, ProposalStatus},
};
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait ProposalRepository: Send + Sync {
    async fn save_proposal(&self, proposal: &Proposal) -> Result<(), String>;
    async fn get_active_proposals(&self) -> Result<Vec<Proposal>, String>;
}

pub trait TokenLedger: Send + Sync {
    fn get_balance(&self, user_id: &str) -> f64;
}

#[async_trait]
impl ProposalRepository for MongoDatabase {
    async fn save_proposal(&self, proposal: &Proposal) -> Result<(), String> {
        MongoDatabase::save_proposal(self, proposal)
            .await
            .map_err(|e| e.to_string())
    }

    async fn get_active_proposals(&self) -> Result<Vec<Proposal>, String> {
        MongoDatabase::get_active_proposals(self)
            .await
            .map_err(|e| e.to_string())
    }
}

impl TokenLedger for PProjectToken {
    fn get_balance(&self, user_id: &str) -> f64 {
        PProjectToken::get_balance(self, user_id)
    }
}

pub struct DaoGovernance {
    mongo_db: Arc<dyn ProposalRepository>,
    token_contract: Arc<dyn TokenLedger>,
    proposals: HashMap<String, Proposal>,
    votes: HashMap<String, HashMap<String, bool>>, // proposal_id -> (user_id -> approve)
}

impl DaoGovernance {
    pub fn new(mongo_db: MongoDatabase, token_contract: PProjectToken) -> Self {
        Self::with_dependencies(Arc::new(mongo_db), Arc::new(token_contract))
    }

    pub fn with_dependencies(
        mongo_db: Arc<dyn ProposalRepository>,
        token_contract: Arc<dyn TokenLedger>,
    ) -> Self {
        Self {
            mongo_db,
            token_contract,
            proposals: HashMap::new(),
            votes: HashMap::new(),
        }
    }

    /// Create a new proposal
    pub async fn create_proposal(
        &mut self,
        title: String,
        description: String,
        creator_id: String,
    ) -> Result<String, String> {
        let proposal_id = p_project_core::utils::generate_id();

        let proposal = Proposal {
            id: proposal_id.clone(),
            title,
            description,
            creator_id,
            created_at: chrono::Utc::now().naive_utc(),
            voting_end_time: chrono::Utc::now().naive_utc() + chrono::Duration::days(7), // 1 week voting
            status: ProposalStatus::Active,
        };

        // Save to repository
        self.mongo_db.save_proposal(&proposal).await?;

        self.proposals.insert(proposal_id.clone(), proposal);
        self.votes.insert(proposal_id.clone(), HashMap::new());

        Ok(proposal_id)
    }

    /// Vote on a proposal
    pub fn vote_on_proposal(
        &mut self,
        proposal_id: &str,
        user_id: &str,
        approve: bool,
    ) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if matches!(proposal.status, ProposalStatus::Active) == false {
            return Err("Voting is not active for this proposal".to_string());
        }

        // Record the vote
        if let Some(votes) = self.votes.get_mut(proposal_id) {
            votes.insert(user_id.to_string(), approve);
        }

        Ok(())
    }

    /// Tally votes for a proposal and update status
    pub fn tally_votes(&mut self, proposal_id: &str) -> Result<(), String> {
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or("Proposal not found")?;

        if matches!(proposal.status, ProposalStatus::Active) == false {
            return Err("Proposal is not active".to_string());
        }

        let votes = self
            .votes
            .get(proposal_id)
            .ok_or("No votes found for proposal")?;

        let mut approve_votes = 0.0;
        let mut reject_votes = 0.0;

        // Count weighted votes based on token holdings
        for (user_id, &approve) in votes {
            let balance = self.token_contract.get_balance(user_id);
            if approve {
                approve_votes += balance;
            } else {
                reject_votes += balance;
            }
        }

        // Update proposal status based on vote results
        if approve_votes > reject_votes {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        Ok(())
    }

    /// Get all active proposals
    pub async fn get_active_proposals(&self) -> Result<Vec<Proposal>, String> {
        self.mongo_db.get_active_proposals().await
    }

    /// Inspect a proposal by id (useful for tests and monitoring)
    pub fn get_proposal(&self, proposal_id: &str) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as ChronoDuration;
    use chrono::Utc;
    use p_project_core::models::ProposalStatus;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct MockProposalRepo {
        saved: Mutex<Vec<Proposal>>,
        active: Mutex<Vec<Proposal>>,
    }

    impl MockProposalRepo {
        fn saved(&self) -> Vec<Proposal> {
            self.saved.lock().unwrap().clone()
        }

        fn set_active(&self, proposals: Vec<Proposal>) {
            *self.active.lock().unwrap() = proposals;
        }
    }

    #[async_trait]
    impl ProposalRepository for MockProposalRepo {
        async fn save_proposal(&self, proposal: &Proposal) -> Result<(), String> {
            self.saved.lock().unwrap().push(proposal.clone());
            Ok(())
        }

        async fn get_active_proposals(&self) -> Result<Vec<Proposal>, String> {
            Ok(self.active.lock().unwrap().clone())
        }
    }

    #[derive(Default)]
    struct MockTokenLedger {
        balances: Mutex<HashMap<String, f64>>,
    }

    impl MockTokenLedger {
        fn with_balances(entries: &[(&str, f64)]) -> Self {
            let mut balances = HashMap::new();
            for (user, amount) in entries {
                balances.insert((*user).to_string(), *amount);
            }
            Self {
                balances: Mutex::new(balances),
            }
        }
    }

    impl TokenLedger for MockTokenLedger {
        fn get_balance(&self, user_id: &str) -> f64 {
            self.balances
                .lock()
                .unwrap()
                .get(user_id)
                .copied()
                .unwrap_or(0.0)
        }
    }

    fn sample_proposal(id: &str) -> Proposal {
        let now = Utc::now().naive_utc();
        Proposal {
            id: id.to_string(),
            title: format!("Proposal {}", id),
            description: "desc".to_string(),
            creator_id: "creator".to_string(),
            created_at: now,
            voting_end_time: now + ChronoDuration::days(7),
            status: ProposalStatus::Active,
        }
    }

    #[tokio::test]
    async fn create_proposal_persists_and_initializes_votes() {
        let repo = Arc::new(MockProposalRepo::default());
        let token = Arc::new(MockTokenLedger::default());
        let mut dao = DaoGovernance::with_dependencies(repo.clone(), token);

        let proposal_id = dao
            .create_proposal(
                "Title".to_string(),
                "Description".to_string(),
                "creator".to_string(),
            )
            .await
            .expect("create_proposal should succeed");

        assert!(!proposal_id.is_empty());
        assert_eq!(repo.saved().len(), 1);
        dao.vote_on_proposal(&proposal_id, "user-1", true)
            .expect("vote should succeed");
    }

    #[tokio::test]
    async fn tally_votes_respects_token_weights() {
        let repo = Arc::new(MockProposalRepo::default());
        let token = Arc::new(MockTokenLedger::with_balances(&[
            ("alice", 100.0),
            ("bob", 40.0),
        ]));
        let mut dao = DaoGovernance::with_dependencies(repo, token);

        let proposal_id = dao
            .create_proposal(
                "Upgrade".to_string(),
                "Ship new feature".to_string(),
                "alice".to_string(),
            )
            .await
            .unwrap();

        dao.vote_on_proposal(&proposal_id, "alice", true).unwrap();
        dao.vote_on_proposal(&proposal_id, "bob", false).unwrap();

        dao.tally_votes(&proposal_id).unwrap();
        let proposal = dao.get_proposal(&proposal_id).unwrap();
        assert!(matches!(proposal.status, ProposalStatus::Passed));
    }

    #[tokio::test]
    async fn get_active_proposals_proxies_to_repository() {
        let repo = Arc::new(MockProposalRepo::default());
        repo.set_active(vec![sample_proposal("p1"), sample_proposal("p2")]);
        let token = Arc::new(MockTokenLedger::default());
        let dao = DaoGovernance::with_dependencies(repo, token);

        let proposals = dao.get_active_proposals().await.unwrap();
        assert_eq!(proposals.len(), 2);
        assert_eq!(proposals[0].id, "p1");
    }

    #[tokio::test]
    async fn voting_rejected_for_inactive_proposals() {
        let repo = Arc::new(MockProposalRepo::default());
        let token = Arc::new(MockTokenLedger::default());
        let mut dao = DaoGovernance::with_dependencies(repo, token);

        let proposal_id = dao
            .create_proposal(
                "Inactive".to_string(),
                "Should block votes".to_string(),
                "creator".to_string(),
            )
            .await
            .unwrap();

        dao.proposals.get_mut(&proposal_id).unwrap().status = ProposalStatus::Rejected;

        let err = dao
            .vote_on_proposal(&proposal_id, "user-1", true)
            .expect_err("vote should fail for inactive proposals");
        assert!(
            err.contains("Voting is not active"),
            "unexpected error message: {err}"
        );
    }

    #[tokio::test]
    async fn duplicate_votes_overwrite_previous_choice() {
        let repo = Arc::new(MockProposalRepo::default());
        let token = Arc::new(MockTokenLedger::with_balances(&[("alice", 100.0)]));
        let mut dao = DaoGovernance::with_dependencies(repo, token);

        let proposal_id = dao
            .create_proposal(
                "Change".to_string(),
                "Duplicate vote scenario".to_string(),
                "creator".to_string(),
            )
            .await
            .unwrap();

        dao.vote_on_proposal(&proposal_id, "alice", true)
            .expect("first vote should succeed");
        dao.vote_on_proposal(&proposal_id, "alice", false)
            .expect("second vote should overwrite");

        let ballot = dao
            .votes
            .get(&proposal_id)
            .and_then(|v| v.get("alice"))
            .copied();
        assert_eq!(ballot, Some(false));

        dao.tally_votes(&proposal_id).unwrap();
        let proposal = dao.get_proposal(&proposal_id).unwrap();
        assert!(matches!(proposal.status, ProposalStatus::Rejected));
    }
}
