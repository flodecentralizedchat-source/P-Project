#[cfg(feature = "database-tests")]
use p_project_core::database::MySqlDatabase;
#[cfg(feature = "database-tests")]
use rust_decimal::prelude::FromPrimitive;
#[cfg(feature = "database-tests")]
use rust_decimal::Decimal;
#[cfg(feature = "database-tests")]
use sqlx::Row;

#[cfg(feature = "database-tests")]
async fn init_test_db() -> Option<(MySqlDatabase, sqlx::MySqlPool)> {
    let db_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "mysql://pproject:pprojectpassword@localhost/p_project_test".to_string()
    });

    let db = match MySqlDatabase::new(&db_url).await {
        Ok(db) => db,
        Err(_) => return None,
    };
    if db.init_tables().await.is_err() {
        return None;
    }
    let pool = match sqlx::MySqlPool::connect(&db_url).await {
        Ok(p) => p,
        Err(_) => return None,
    };
    // cleanup
    let _ = sqlx::query("DELETE FROM transactions").execute(&pool).await;
    let _ = sqlx::query("DELETE FROM remittances").execute(&pool).await;
    let _ = sqlx::query("DELETE FROM stakes").execute(&pool).await;
    let _ = sqlx::query("DELETE FROM balances").execute(&pool).await;
    let _ = sqlx::query("DELETE FROM users").execute(&pool).await;
    Some((db, pool))
}

#[cfg(feature = "database-tests")]
async fn set_balance(pool: &sqlx::MySqlPool, user_id: &str, amount: f64) {
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
async fn remittance_processes_and_records() {
    let (db, pool) = match init_test_db().await {
        Some(v) => v,
        None => {
            println!("Skipping remittance test - DB unavailable");
            return;
        }
    };

    let sender = db
        .create_user("u_sender", "sender", "0x0001")
        .await
        .unwrap();
    let recipient = db
        .create_user("u_recipient", "recipient", "0x0002")
        .await
        .unwrap();
    let fee_user = db.create_user("u_fee", "fee", "0x0003").await.unwrap();

    set_balance(&pool, &sender.id, 1000.0).await;
    set_balance(&pool, &recipient.id, 0.0).await;
    set_balance(&pool, &fee_user.id, 0.0).await;

    let remittance_id = "remit_001";
    let amount = Decimal::from_f64(250.0).unwrap();
    let fee = Decimal::from_f64(0.25).unwrap(); // 0.1% of 250 = 0.25

    let record = db
        .process_remittance(
            remittance_id,
            &sender.id,
            &recipient.id,
            amount,
            fee,
            Some(&fee_user.id),
        )
        .await
        .expect("remittance processed");

    assert_eq!(record.id, remittance_id);
    assert_eq!(record.amount, amount);
    assert_eq!(record.fee, fee);

    // Check balances
    let srow = sqlx::query("SELECT available_balance FROM balances WHERE user_id = ?")
        .bind(&sender.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let sender_bal: f64 = srow.get(0);
    assert_eq!(sender_bal, 749.75); // 1000 - 250 - 0.25

    let rrow = sqlx::query("SELECT available_balance FROM balances WHERE user_id = ?")
        .bind(&recipient.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let recipient_bal: f64 = rrow.get(0);
    assert_eq!(recipient_bal, 250.0);

    let frow = sqlx::query("SELECT available_balance FROM balances WHERE user_id = ?")
        .bind(&fee_user.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let fee_bal: f64 = frow.get(0);
    assert_eq!(fee_bal, 0.25);
}
