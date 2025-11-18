#[cfg(feature = "database-tests")]
use p_project_core::database::MySqlDatabase;
#[cfg(feature = "database-tests")]
use sqlx::MySqlPool;

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

    let _ = sqlx::query("DELETE FROM events").execute(&pool).await;
    let _ = sqlx::query("DELETE FROM users").execute(&pool).await;
    Some((db, pool))
}

#[cfg(feature = "database-tests")]
#[tokio::test]
async fn create_and_list_events() {
    let (db, _pool) = match init_test_db().await {
        Some(v) => v,
        None => {
            println!("Skipping events integration test - DB unavailable");
            return;
        }
    };

    // Create an admin user (creator) for FK
    let admin = db
        .create_user(
            "admin1",
            "admin1",
            "0xabc4000000000000000000000000000000000000",
        )
        .await
        .unwrap();

    let event_id = p_project_core::utils::generate_id();
    let start = chrono::Utc::now().naive_utc();
    db.create_event(
        &event_id,
        "Community AMA",
        "Quarterly roadmap AMA",
        "AMA",
        start,
        None,
        Some("https://t.me/p_project_ama"),
        &admin.id,
    )
    .await
    .unwrap();

    // List and fetch
    let list = db.list_events(10).await.unwrap();
    assert!(list.iter().any(|(id, ..)| id == &event_id));

    let get = db.get_event(&event_id).await.unwrap();
    assert!(get.is_some());
    let (gid, title, _, kind, ..) = get.unwrap();
    assert_eq!(gid, event_id);
    assert_eq!(title, "Community AMA");
    assert_eq!(kind, "AMA");
}
