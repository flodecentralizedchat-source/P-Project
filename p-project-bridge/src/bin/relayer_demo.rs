use std::{env, error::Error};

use p_project_bridge::BridgeService;
use p_project_core::database::MySqlDatabase;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://user:password@localhost/p_project".to_string());
    let db = MySqlDatabase::new(&db_url).await?;
    db.init_tables().await?;

    let service = BridgeService::new(db);
    println!(
        "Bridge service ready. Supported chains: {:?}",
        service.get_supported_chains()
    );

    let relayer = service.relayer();
    println!("Relayer loop starting (Ctrl+C to stop)...");

    tokio::select! {
        _ = relayer.run_loop() => {}
        _ = signal::ctrl_c() => println!("Shutdown signal received, exiting relayer."),
    }

    Ok(())
}
