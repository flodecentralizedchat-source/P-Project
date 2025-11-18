#[cfg(feature = "database-tests")]
use p_project_core::database::MySqlDatabase;
#[cfg(feature = "database-tests")]
use rust_decimal::Decimal;
#[cfg(feature = "database-tests")]
use sqlx::{MySqlPool, Row};

#[cfg(feature = "database-tests")]
async fn init_test_db() -> Option<(MySqlDatabase, MySqlPool)> {
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
    let pool = match MySqlPool::connect(&db_url).await {
        Ok(pool) => pool,
        Err(_) => return None,
    };

    let _ = sqlx::query("DELETE FROM referrals").execute(&pool).await;
    let _ = sqlx::query("DELETE FROM referral_codes")
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM learning_completions")
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM learning_content")
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM transactions").execute(&pool).await;
    let _ = sqlx::query("DELETE FROM balances").execute(&pool).await;
    let _ = sqlx::query("DELETE FROM users").execute(&pool).await;

    Some((db, pool))
}

#[cfg(feature = "database-tests")]
#[tokio::test]
async fn referral_bonus_on_learning_completion() {
    let (db, pool) = match init_test_db().await {
        Some(v) => v,
        None => {
            println!("Skipping referral integration test - DB unavailable");
            return;
        }
    };

    // Create users
    let referrer = db
        .create_user(
            "referrer",
            "referrer",
            "0xabc2000000000000000000000000000000000000",
        )
        .await
        .unwrap();
    let referred = db
        .create_user(
            "referred",
            "referred",
            "0xabc3000000000000000000000000000000000000",
        )
        .await
        .unwrap();

    // Generate referral code for referrer
    let code = db.upsert_referral_code(&referrer.id).await.unwrap();

    // Accept referral
    db.accept_referral(&code, &referred.id)
        .await
        .expect("accept ok");

    // Register a learning content with 20 tokens reward
    let content = db
        .register_learning_content(
            "course_ref_bonus",
            "Referral Course",
            "Test",
            p_project_core::models::LearningContentType::Course,
            Decimal::from_f64(20.0).unwrap(),
            50,
        )
        .await
        .unwrap();

    // Referred user completes the course
    db.record_learning_completion(
        "comp_ref_bonus",
        &referred.id,
        &content.id,
        None,
        Some("learning_pool"),
    )
    .await
    .expect("completion ok");

    // Referrer should receive 5% of 20.0 = 1.0
    let bal_row = sqlx::query("SELECT available_balance FROM balances WHERE user_id = ?")
        .bind(&referrer.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let bal: Decimal = bal_row.get(0);
    assert_eq!(bal, Decimal::from_f64(1.0).unwrap());

    // Stats should show 1 referred
    let count = db.get_referral_stats(&referrer.id).await.unwrap();
    assert_eq!(count, 1);
}
