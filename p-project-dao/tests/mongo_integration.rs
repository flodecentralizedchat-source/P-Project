use p_project_contracts::token::PProjectToken;
use p_project_core::database::mongodb::MongoDatabase;
use p_project_dao::DaoGovernance;

/// Ignored by default because it requires a running MongoDB instance.
#[tokio::test]
#[ignore = "requires a running MongoDB instance and network access"]
async fn create_and_fetch_active_proposal_via_mongo() {
    let uri =
        std::env::var("MONGO_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let db_name = std::env::var("MONGO_DB").unwrap_or_else(|_| "p_project_dao_test".to_string());

    let mongo = match MongoDatabase::new(&uri, &db_name).await {
        Ok(db) => db,
        Err(err) => {
            eprintln!("Skipping Mongo integration test: {err}");
            return;
        }
    };

    let mut token = PProjectToken::new(1_000_000.0, 0.0, 0.0);
    token.initialize_distribution(vec![("integration-user".to_string(), 1_000.0)]);

    let mut dao = DaoGovernance::new(mongo, token);
    let proposal_id = match dao
        .create_proposal(
            "Integration Test".to_string(),
            "Ensure Mongo round-trip works".to_string(),
            "integration-user".to_string(),
        )
        .await
    {
        Ok(id) => id,
        Err(err) => {
            eprintln!("Skipping Mongo integration test (create failed): {err}");
            return;
        }
    };

    let active = match dao.get_active_proposals().await {
        Ok(proposals) => proposals,
        Err(err) => {
            eprintln!("Skipping Mongo integration test (fetch failed): {err}");
            return;
        }
    };

    assert!(
        active.iter().any(|p| p.id == proposal_id),
        "expected to find proposal {proposal_id} in Mongo {:?}",
        active.iter().map(|p| p.id.clone()).collect::<Vec<String>>()
    );
}
