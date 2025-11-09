use std::collections::HashMap;
use std::time::Duration;

use crate::adapter::ChainAdapter;
use p_project_core::database::MySqlDatabase;

pub struct Relayer<'a> {
    adapters: &'a HashMap<String, Box<dyn ChainAdapter + Send + Sync>>,
    db: &'a MySqlDatabase,
}

impl<'a> Relayer<'a> {
    pub fn new(
        adapters: &'a HashMap<String, Box<dyn ChainAdapter + Send + Sync>>,
        db: &'a MySqlDatabase,
    ) -> Self {
        Self { adapters, db }
    }

    pub async fn run_once(&self) {
        // Process Ethereum-locked txs awaiting destination mint
        if let Some(_) = self.adapters.get("Ethereum") {
            if let Ok(items) = self.db.list_bridge_locked_without_dst("Ethereum").await {
                println!("[Relayer] found {} locked bridge tx(s)", items.len());
                for rec in items {
                    println!(
                        "[Relayer] checking bridge tx {} (src {})",
                        rec.id,
                        rec.src_tx_hash.as_deref().unwrap_or("<missing>")
                    );
                    // Check confirmations on source chain
                    if let Some(eth) = self.adapters.get("Ethereum") {
                        if let Ok(status) = eth
                            .get_tx_status(rec.src_tx_hash.as_deref().unwrap_or(""))
                            .await
                        {
                            if status.status == "Success" && status.confirmations >= 1 {
                                // Mint on destination
                                if let Some(dst) = self.adapters.get(&rec.to_chain) {
                                    match dst
                                        .mint_or_release(
                                            &rec.user_id,
                                            &rec.token,
                                            rec.amount,
                                            &rec.from_chain,
                                            rec.src_tx_hash.as_deref().unwrap_or(""),
                                            rec.lock_id.as_deref(),
                                        )
                                        .await
                                    {
                                        Ok(dst_tx) => {
                                            let _ =
                                                self.db.set_bridge_dst_tx(&rec.id, &dst_tx).await;
                                            let _ = self
                                                .db
                                                .update_bridge_status(&rec.id, "Minted", None)
                                                .await;
                                            println!(
                                                "[Relayer] minted bridge tx {} -> {} (dst tx {})",
                                                rec.id, rec.to_chain, dst_tx
                                            );
                                        }
                                        Err(e) => {
                                            println!(
                                                "[Relayer] mint error for tx {}: {:?}",
                                                rec.id, e
                                            );
                                            let _ = self
                                                .db
                                                .update_bridge_status(
                                                    &rec.id,
                                                    "Failed",
                                                    Some(&format!("{:?}", e)),
                                                )
                                                .await;
                                        }
                                    }
                                }
                            } else {
                                println!("[Relayer] waiting for confirmations: status={} confirmations={}", status.status, status.confirmations);
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn run_loop(&self) {
        loop {
            self.run_once().await;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
