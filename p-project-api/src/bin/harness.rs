use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use sqlx::MySqlPool;
use std::env;
use uuid::Uuid;

#[derive(Deserialize)]
struct UserResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct TransferResponse {
    transaction_id: String,
    from_user_id: String,
    to_user_id: String,
    amount: f64,
}

#[derive(Debug, Deserialize)]
struct StakingInfo {
    user_id: String,
    amount: f64,
    start_time: String,
    end_time: Option<String>,
    rewards_earned: f64,
}

#[derive(Debug, Deserialize)]
struct AirdropClaimResponse {
    airdrop_id: String,
    user_id: String,
    amount: f64,
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let api_base = env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let db_url = env::var("API_DB_URL")
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "API_DB_URL not set"))?;
    let client = Client::new();
    let pool = MySqlPool::connect(&db_url).await?;

    // create two users
    let alice = create_user(
        &client,
        &api_base,
        "alice",
        "0xabab000000000000000000000000000000000000",
    )
    .await?;
    let bob = create_user(
        &client,
        &api_base,
        "bob",
        "0xbaba000000000000000000000000000000000000",
    )
    .await?;

    // seed balances directly
    seed_balance(&pool, &alice.id, 150.0).await?;

    let mut req = client
        .post(&format!("{}/transfer", api_base))
        .json(&json!({
            "from_user_id": alice.id,
            "to_user_id": bob.id,
            "amount": 50.0
        }));
    if let Ok(token) = env::var("AUTH_TOKEN") {
        req = req.bearer_auth(token);
    }
    let transfer = req
        .send()
        .await?
        .json::<TransferResponse>()
        .await?;
    println!("Transfer completed: {:#?}", transfer);

    let mut stake_req = client
        .post(&format!("{}/stake", api_base))
        .json(&json!({
            "user_id": alice.id,
            "amount": 40.0,
            "duration_days": 7
        }));
    if let Ok(token) = env::var("AUTH_TOKEN") {
        stake_req = stake_req.bearer_auth(token);
    }
    let stake = stake_req
        .send()
        .await?
        .json::<StakingInfo>()
        .await?;
    println!("Stake recorded: {:#?}", stake);

    let mut unstake_req = client
        .post(&format!("{}/unstake", api_base))
        .json(&json!({ "user_id": alice.id }));
    if let Ok(token) = env::var("AUTH_TOKEN") {
        unstake_req = unstake_req.bearer_auth(token);
    }
    let unstake = unstake_req.send()
        .await?
        .json::<StakingInfo>()
        .await?;
    println!("Unstaked: {:#?}", unstake);

    let airdrop_id = format!("airdrop-{}", uuid::Uuid::new_v4());
    let mut ad_req = client
        .post(&format!("{}/airdrop/create", api_base))
        .json(&json!({
            "total_amount": 200.0,
            "recipients": [
                { "user_id": alice.id, "amount": 70.0 },
                { "user_id": bob.id, "amount": 30.0 }
            ]
        }));
    if let Ok(token) = env::var("AUTH_TOKEN") {
        ad_req = ad_req.bearer_auth(token);
    }
    ad_req.send()
        .await?
        .error_for_status()?;

    let mut claim_req = client
        .post(&format!("{}/airdrop/claim", api_base))
        .json(&json!({
            "airdrop_id": airdrop_id,
            "user_id": alice.id
        }));
    if let Ok(token) = env::var("AUTH_TOKEN") {
        claim_req = claim_req.bearer_auth(token);
    }
    let claim = claim_req.send()
        .await?
        .json::<AirdropClaimResponse>()
        .await?;
    println!("Airdrop claim: {:#?}", claim);

    Ok(())
}

async fn create_user(
    client: &Client,
    base: &str,
    username: &str,
    wallet: &str,
) -> Result<UserResponse> {
    let res = client
        .post(&format!("{}/users", base))
        .json(&json!({ "username": username, "wallet_address": wallet }))
        .send()
        .await?
        .error_for_status()?
        .json::<UserResponse>()
        .await?;
    Ok(res)
}

async fn seed_balance(pool: &MySqlPool, user_id: &str, amount: f64) -> Result<()> {
    sqlx::query(
        "INSERT INTO balances (user_id, available_balance) VALUES (?, ?) ON DUPLICATE KEY UPDATE available_balance = ?",
    )
    .bind(user_id)
    .bind(amount)
    .bind(amount)
    .execute(pool)
    .await?;
    Ok(())
}
