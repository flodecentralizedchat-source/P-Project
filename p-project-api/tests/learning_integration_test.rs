#[cfg(feature = "database-tests")]
use p_project_core::database::{BalanceError, MySqlDatabase};
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
async fn learning_completion_records_reward_and_points() {
    let (db, pool) = match init_test_db().await {
        Some(v) => v,
        None => {
            println!("Skipping learning integration test - DB unavailable");
            return;
        }
    };

    let learner = db
        .create_user(
            "learner",
            "learner",
            "0xabc1000000000000000000000000000000000000",
        )
        .await
        .unwrap();

    let content = db
        .register_learning_content(
            "course_101",
            "Peace 101",
            "Intro course",
            p_project_core::models::LearningContentType::Course,
            Decimal::from_f64(15.0).unwrap(),
            100,
        )
        .await
        .unwrap();

    let list = db.list_learning_content(10).await.unwrap();
    assert!(list.iter().any(|item| item.id == content.id));

    let completion = db
        .record_learning_completion(
            "completion_1",
            &learner.id,
            &content.id,
            Some("quiz-2024"),
            Some("learning_pool"),
        )
        .await
        .expect("record success");

    assert_eq!(completion.user_id, learner.id);
    assert_eq!(completion.content_id, content.id);
    assert_eq!(completion.reward_tokens, Decimal::from_f64(15.0).unwrap());
    assert_eq!(completion.reward_points, 100);

    let balance_row = sqlx::query("SELECT available_balance FROM balances WHERE user_id = ?")
        .bind(&learner.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let balance: Decimal = balance_row.get(0);
    assert_eq!(balance, Decimal::from_f64(15.0).unwrap());

    let second_attempt = db
        .record_learning_completion(
            "completion_2",
            &learner.id,
            &content.id,
            Some("quiz-2024"),
            Some("learning_pool"),
        )
        .await;
    assert!(matches!(
        second_attempt,
        Err(BalanceError::AlreadyCompleted)
    ));

    let completions = db
        .list_user_learning_completions(&learner.id, 10)
        .await
        .unwrap();
    assert_eq!(completions.len(), 1);
    assert_eq!(completions[0].id, completion.id);
}
