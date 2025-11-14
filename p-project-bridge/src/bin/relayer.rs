use std::sync::Arc;

use p_project_core::database::MySqlDatabase;
use p_project_bridge::BridgeService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read configuration
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "DATABASE_URL not set"))?;

    // Connect to database and ensure schema exists
    let db = MySqlDatabase::new(&db_url).await?;
    db.init_tables().await?;

    let service = BridgeService::new(Arc::new(db));

    // Log supported chains and start relayer loop
    let chains = service.get_supported_chains().join(", ");
    println!("[Relayer] starting. Supported chains: {}", chains);
    service.relayer().run_loop().await;

    Ok(())
}
