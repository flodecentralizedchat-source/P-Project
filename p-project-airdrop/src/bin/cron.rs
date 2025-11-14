use p_project_core::database::MySqlDatabase;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read configuration (env-only; fail if missing)
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "DATABASE_URL not set"))?;

    // Connect and ensure schema exists
    let db = MySqlDatabase::new(&db_url).await?;
    db.init_tables().await?;

    // Minimal maintenance example: record or print latest airdrop state
    match db.load_latest_airdrop_state().await? {
        Some(state) => {
            println!("[Airdrop Cron] latest state: {}", state);
        }
        None => {
            println!("[Airdrop Cron] no state found; initializing");
            db.save_airdrop_state("initialized").await?;
        }
    }

    println!("[Airdrop Cron] run complete");
    Ok(())
}
