use p_project_core::database::MySqlDatabase;
use p_project_core::models::{TransactionType, StakingInfo};
use sqlx::MySqlPool;

async fn init_test_db() -> (MySqlDatabase, MySqlPool) {
    let db_url =
        std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "mysql://pproject:pprojectpassword@localhost/p_project_test".to_string());
    let db = MySqlDatabase::new(&db_url).await.expect("connect to test DB");
    db.init_tables().await.expect("init tables");
    let pool = MySqlPool::connect(&db_url).await.expect("connect pool");

    sqlx::query("DELETE FROM transactions").execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM stakes").execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM balances").execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM users").execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM airdrop_recipients").execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM airdrops").execute(&pool).await.unwrap();

    (db, pool)
}

fn user_payload(username: &str, wallet: &str) -> (String, String) {
    (username.to_string(), wallet.to_string())
}

fn wallet_for(name: &str) -> String {
    format!("0x{:0>40}", name)
}

async fn populate_balances(pool: &MySqlPool, user_id: &str, amount: f64) {
    sqlx::query(
        "INSERT INTO balances (user_id, available_balance) VALUES (?, ?) ON DUPLICATE KEY UPDATE available_balance = ?",
    )
    .bind(user_id)
    .bind(amount)
    .bind(amount)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn transfer_updates_balances() {
    let (db, pool) = init_test_db().await;

    let user_a = db
        .create_user("alice", "alice", "0xabc0000000000000000000000000000000000000")
        .await
        .unwrap();
    let user_b = db
        .create_user("bob", "bob", "0xdef0000000000000000000000000000000000000")
        .await
        .unwrap();

    populate_balances(&pool, &user_a.id, 100.0).await;

    db.transfer_tokens(
        "tx1",
        &user_a.id,
        &user_b.id,
        25.0,
        TransactionType::Transfer,
    )
    .await
    .expect("transfer succeeds");

    let available_a = sqlx::query!("SELECT available_balance FROM balances WHERE user_id = ?", user_a.id)
        .fetch_one(&pool)
        .await
        .unwrap()
        .available_balance;
    let available_b = sqlx::query!("SELECT available_balance FROM balances WHERE user_id = ?", user_b.id)
        .fetch_one(&pool)
        .await
        .unwrap()
        .available_balance;

    assert_eq!(available_a, 75.0);
    assert_eq!(available_b, 25.0);
}

#[tokio::test]
async fn stake_and_unstake_flow() {
    let (db, pool) = init_test_db().await;

    let user = db
        .create_user("staker", "staker", "0xaaa0000000000000000000000000000000000000")
        .await
        .unwrap();

    populate_balances(&pool, &user.id, 200.0).await;

    let stake_info = db
        .stake_tokens("stake1", &user.id, 50.0, 15)
        .await
        .expect("stake");

    assert!(matches!(stake_info, StakingInfo { rewards_earned: _, .. }));

    let balance_row =
        sqlx::query!("SELECT available_balance, staked_balance FROM balances WHERE user_id = ?", user.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(balance_row.available_balance, 150.0);
    assert_eq!(balance_row.staked_balance, 50.0);

    let unstake = db
        .unstake_tokens(&user.id, Some("stake1"))
        .await
        .expect("unstake");

    assert!(unstake.end_time.is_some());
    let balances =
        sqlx::query!("SELECT available_balance, staked_balance FROM balances WHERE user_id = ?", user.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(balances.available_balance, 200.0);
    assert_eq!(balances.staked_balance, 0.0);
}

#[tokio::test]
async fn airdrop_claims_work() {
    let (db, _pool) = init_test_db().await;
    let user = db
        .create_user("claimer", "claimer", "0xaaa0000000000000000000000000000000000010")
        .await
        .unwrap();

    db.create_airdrop("airdrop1", 500.0, None, None)
        .await
        .unwrap();
    db.add_airdrop_recipients(
        "airdrop1",
        &[(user.id.clone(), 100.0)],
        Some("test"),
    )
    .await
    .unwrap();

    let amount = db.claim_airdrop("airdrop1", &user.id).await.unwrap();
    assert_eq!(amount, 100.0);
}
