use p_project_contracts::token::PProjectToken;
use p_project_core::database::InMemoryDatabase;
use p_project_dao::DaoGovernance;

#[tokio::test]
async fn test_create_proposal() {
    let mut token = PProjectToken::new(1_000_000.0, 0.0, 0.0);
    token.initialize_distribution(vec![("test-user".to_string(), 1_000.0)]);

    // Using in-memory database for testing
    let db = InMemoryDatabase::new();
    let mut dao = DaoGovernance::new(db, token);

    let proposal_id = dao
        .create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            "test-user".to_string(),
        )
        .await
        .unwrap();

    assert!(!proposal_id.is_empty());
}

#[tokio::test]
async fn test_vote_on_proposal() {
    let mut token = PProjectToken::new(1_000_000.0, 0.0, 0.0);
    token.initialize_distribution(vec![
        ("creator".to_string(), 1_000.0),
        ("voter1".to_string(), 500.0),
        ("voter2".to_string(), 300.0),
    ]);

    // Using in-memory database for testing
    let db = InMemoryDatabase::new();
    let mut dao = DaoGovernance::new(db, token);

    let proposal_id = dao
        .create_proposal(
            "Test Proposal".to_string(),
            "This is a test proposal".to_string(),
            "creator".to_string(),
        )
        .await
        .unwrap();

    // Vote on the proposal
    assert!(dao.vote_on_proposal(&proposal_id, "voter1", true).is_ok());
    assert!(dao.vote_on_proposal(&proposal_id, "voter2", false).is_ok());

    // Check vote counts
    let (approve_votes, reject_votes) = dao.get_vote_count(&proposal_id).unwrap();
    assert_eq!(approve_votes, 500);
    assert_eq!(reject_votes, 300);
}

#[tokio::test]
async fn test_delegate_vote() {
    let mut token = PProjectToken::new(1_000_000.0, 0.0, 0.0);
    token.initialize_distribution(vec![
        ("user1".to_string(), 100.0),
        ("user2".to_string(), 200.0),
    ]);

    // Using in-memory database for testing
    let db = InMemoryDatabase::new();
    let mut dao = DaoGovernance::new(db, token);

    // Delegate user1's vote to user2
    assert!(dao.delegate_vote("user1", "user2").is_ok());

    // Check delegation
    assert_eq!(dao.get_delegate("user1"), Some(&"user2".to_string()));
}

#[tokio::test]
async fn test_quadratic_voting() {
    let mut token = PProjectToken::new(1_000_000.0, 0.0, 0.0);
    token.initialize_distribution(vec![("voter".to_string(), 100.0)]);

    // Using in-memory database for testing
    let db = InMemoryDatabase::new();
    let dao = DaoGovernance::new(db, token);

    // Calculate quadratic voting power (sqrt of token balance)
    let voting_power = dao.calculate_quadratic_voting_power("voter");
    assert_eq!(voting_power, 10.0); // sqrt(100) = 10
}

#[tokio::test]
async fn test_batch_delegate_votes() {
    let mut token = PProjectToken::new(1_000_000.0, 0.0, 0.0);
    token.initialize_distribution(vec![
        ("user1".to_string(), 100.0),
        ("user2".to_string(), 200.0),
        ("delegate".to_string(), 50.0),
    ]);

    // Using in-memory database for testing
    let db = InMemoryDatabase::new();
    let mut dao = DaoGovernance::new(db, token);

    // Batch delegate votes
    let delegations = vec![("user1", "delegate"), ("user2", "delegate")];
    assert!(dao.batch_delegate_votes(delegations).is_ok());

    // Check delegations
    assert_eq!(dao.get_delegate("user1"), Some(&"delegate".to_string()));
    assert_eq!(dao.get_delegate("user2"), Some(&"delegate".to_string()));
}
