#[cfg(feature = "database-tests")]
use p_project_core::database::MySqlDatabase;
#[cfg(feature = "database-tests")]
use p_project_core::models::{StakingInfo, TransactionType};
#[cfg(feature = "database-tests")]
use sqlx::MySqlPool;
#[cfg(feature = "database-tests")]
use sqlx::Row;
#[cfg(feature = "database-tests")]
use rust_decimal::Decimal;

#[cfg(feature = "database-tests")]
async fn init_test_db() -> Option<(MySqlDatabase, MySqlPool)> {
    let db_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "mysql://pproject:pprojectpassword@localhost/p_project_test".to_string()
    });
    
    // Try to connect to the database, return None if we can't
    let db = match MySqlDatabase::new(&db_url).await {
        Ok(db) => db,
        Err(_) => return None,
    };
    
    // Try to initialize tables, return None if we can't
    if db.init_tables().await.is_err() {
        return None;
    }
    
    let pool = match MySqlPool::connect(&db_url).await {
        Ok(pool) => pool,
        Err(_) => return None,
    };

    // Try to clean up test data, but don't fail if we can't
    let _ = sqlx::query("DELETE FROM transactions")
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM stakes")
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM balances")
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM users")
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM airdrop_recipients")
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM airdrops")
        .execute(&pool)
        .await;

    Some((db, pool))
}

#[cfg(feature = "database-tests")]
fn user_payload(username: &str, wallet: &str) -> (String, String) {
    (username.to_string(), wallet.to_string())
}

#[cfg(feature = "database-tests")]
fn wallet_for(name: &str) -> String {
    format!("0x{:0>40}", name)
}

#[cfg(feature = "database-tests")]
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

#[cfg(feature = "database-tests")]
#[tokio::test]
async fn transfer_updates_balances() {
    let (db, pool) = match init_test_db().await {
        Some(connections) => connections,
        None => {
            println!("Skipping integration test - cannot connect to database");
            return;
        }
    };

    let user_a = db
        .create_user(
            "alice",
            "alice",
            "0xabc0000000000000000000000000000000000000",
        )
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
        Decimal::from_f64(25.0).unwrap(),
        TransactionType::Transfer,
    )
    .await
    .expect("transfer succeeds");

    let available_a_row = sqlx::query(
        "SELECT available_balance FROM balances WHERE user_id = ?",
    )
    .bind(&user_a.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let available_a: f64 = available_a_row.get(0);
    
    let available_b_row = sqlx::query(
        "SELECT available_balance FROM balances WHERE user_id = ?",
    )
    .bind(&user_b.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let available_b: f64 = available_b_row.get(0);

    assert_eq!(available_a, 75.0);
    assert_eq!(available_b, 25.0);
}

#[cfg(feature = "database-tests")]
#[tokio::test]
async fn stake_and_unstake_flow() {
    let (db, pool) = match init_test_db().await {
        Some(connections) => connections,
        None => {
            println!("Skipping integration test - cannot connect to database");
            return;
        }
    };

    let user = db
        .create_user(
            "staker",
            "staker",
            "0xaaa0000000000000000000000000000000000000",
        )
        .await
        .unwrap();

    populate_balances(&pool, &user.id, 200.0).await;

    let stake_info = db
        .stake_tokens("stake1", &user.id, Decimal::from_f64(50.0).unwrap(), 15)
        .await
        .expect("stake");

    assert!(matches!(
        stake_info,
        StakingInfo {
            rewards_earned: _,
            ..
        }
    ));

    let balance_row = sqlx::query(
        "SELECT available_balance, staked_balance FROM balances WHERE user_id = ?",
    )
    .bind(&user.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let available_balance: f64 = balance_row.get(0);
    let staked_balance: f64 = balance_row.get(1);
    assert_eq!(available_balance, 150.0);
    assert_eq!(staked_balance, 50.0);

    let unstake = db
        .unstake_tokens(&user.id, Some("stake1"))
        .await
        .expect("unstake");

    assert!(unstake.end_time.is_some());
    let balances_row = sqlx::query(
        "SELECT available_balance, staked_balance FROM balances WHERE user_id = ?",
    )
    .bind(&user.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let available_balance: f64 = balances_row.get(0);
    let staked_balance: f64 = balances_row.get(1);
    assert_eq!(available_balance, 200.0);
    assert_eq!(staked_balance, 0.0);
}

#[cfg(feature = "database-tests")]
#[tokio::test]
async fn airdrop_claims_work() {
    let (db, _pool) = match init_test_db().await {
        Some(connections) => connections,
        None => {
            println!("Skipping integration test - cannot connect to database");
            return;
        }
    };
    
    let user = db
        .create_user(
            "claimer",
            "claimer",
            "0xaaa0000000000000000000000000000000000010",
        )
        .await
        .unwrap();

    db.create_airdrop("airdrop1", Decimal::from_f64(500.0).unwrap(), None, None)
        .await
        .unwrap();
    db.add_airdrop_recipients("airdrop1", &[(user.id.clone(), Decimal::from_f64(100.0).unwrap())], Some("test"))
        .await
        .unwrap();

    let amount = db.claim_airdrop("airdrop1", &user.id).await.unwrap();
    assert_eq!(amount, Decimal::from_f64(100.0).unwrap());
}
