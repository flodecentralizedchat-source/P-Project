use p_project_core::{models::User, utils::shorten_wallet_address};
use p_project_contracts::{nft::{NFTMetadata, NFT, NFTCollection, MarketplaceListing}, NFTContract};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Proposal struct for frontend
#[wasm_bindgen]
#[derive(Debug)]
pub struct Proposal {
    id: String,
    title: String,
    description: String,
    creator_id: String,
    created_at: String, // ISO string format
    voting_end_time: String, // ISO string format
    status: String,
}

#[wasm_bindgen]
impl Proposal {
    #[wasm_bindgen(constructor)]
    pub fn new(
        id: String,
        title: String,
        description: String,
        creator_id: String,
        created_at: String,
        voting_end_time: String,
        status: String,
    ) -> Proposal {
        Proposal {
            id,
            title,
            description,
            creator_id,
            created_at,
            voting_end_time,
            status,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn title(&self) -> String {
        self.title.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> String {
        self.description.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn creator_id(&self) -> String {
        self.creator_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn created_at(&self) -> String {
        self.created_at.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn voting_end_time(&self) -> String {
        self.voting_end_time.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> String {
        self.status.clone()
    }
}

#[wasm_bindgen]
pub struct WebUser {
    inner: User,
}

#[wasm_bindgen]
impl WebUser {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, username: String, wallet_address: String) -> WebUser {
        WebUser {
            inner: User {
                id,
                username,
                wallet_address,
                created_at: chrono::Utc::now().naive_utc(),
            },
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn username(&self) -> String {
        self.inner.username.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn wallet_address(&self) -> String {
        self.inner.wallet_address.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn short_wallet_address(&self) -> String {
        shorten_wallet_address(&self.inner.wallet_address)
    }

    #[wasm_bindgen]
    pub fn to_string(&self) -> String {
        format!(
            "User({}, {})",
            self.inner.username,
            self.short_wallet_address()
        )
    }
}

// Staking calculator result
#[wasm_bindgen]
pub struct StakingYieldResult {
    amount: f64,
    duration_days: i64,
    projected_rewards: f64,
    total_return: f64,
    apy_rate: f64,
}

#[wasm_bindgen]
impl StakingYieldResult {
    #[wasm_bindgen(constructor)]
    pub fn new(
        amount: f64,
        duration_days: i64,
        projected_rewards: f64,
        total_return: f64,
        apy_rate: f64,
    ) -> StakingYieldResult {
        StakingYieldResult {
            amount,
            duration_days,
            projected_rewards,
            total_return,
            apy_rate,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn amount(&self) -> f64 {
        self.amount
    }

    #[wasm_bindgen(getter)]
    pub fn duration_days(&self) -> i64 {
        self.duration_days
    }

    #[wasm_bindgen(getter)]
    pub fn projected_rewards(&self) -> f64 {
        self.projected_rewards
    }

    #[wasm_bindgen(getter)]
    pub fn total_return(&self) -> f64 {
        self.total_return
    }

    #[wasm_bindgen(getter)]
    pub fn apy_rate(&self) -> f64 {
        self.apy_rate
    }
}

// NFT struct for frontend
#[wasm_bindgen]
pub struct WebNFT {
    id: String,
    collection_id: String,
    name: String,
    description: String,
    image: String,
    owner: String,
    creator: String,
    royalty_percentage: f64,
}

#[wasm_bindgen]
impl WebNFT {
    #[wasm_bindgen(constructor)]
    pub fn new(
        id: String,
        collection_id: String,
        name: String,
        description: String,
        image: String,
        owner: String,
        creator: String,
        royalty_percentage: f64,
    ) -> WebNFT {
        WebNFT {
            id,
            collection_id,
            name,
            description,
            image,
            owner,
            creator,
            royalty_percentage,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn collection_id(&self) -> String {
        self.collection_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> String {
        self.description.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn image(&self) -> String {
        self.image.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn owner(&self) -> String {
        self.owner.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn creator(&self) -> String {
        self.creator.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn royalty_percentage(&self) -> f64 {
        self.royalty_percentage
    }
}

// NFT Collection struct for frontend
#[wasm_bindgen]
pub struct WebNFTCollection {
    id: String,
    name: String,
    symbol: String,
    creator: String,
    description: String,
}

#[wasm_bindgen]
impl WebNFTCollection {
    #[wasm_bindgen(constructor)]
    pub fn new(
        id: String,
        name: String,
        symbol: String,
        creator: String,
        description: String,
    ) -> WebNFTCollection {
        WebNFTCollection {
            id,
            name,
            symbol,
            creator,
            description,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn symbol(&self) -> String {
        self.symbol.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn creator(&self) -> String {
        self.creator.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> String {
        self.description.clone()
    }
}

// Marketplace listing struct for frontend
#[wasm_bindgen]
pub struct WebMarketplaceListing {
    id: String,
    nft_id: String,
    seller: String,
    price: f64,
    currency: String,
}

#[wasm_bindgen]
impl WebMarketplaceListing {
    #[wasm_bindgen(constructor)]
    pub fn new(
        id: String,
        nft_id: String,
        seller: String,
        price: f64,
        currency: String,
    ) -> WebMarketplaceListing {
        WebMarketplaceListing {
            id,
            nft_id,
            seller,
            price,
            currency,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn nft_id(&self) -> String {
        self.nft_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn seller(&self) -> String {
        self.seller.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn price(&self) -> f64 {
        self.price
    }

    #[wasm_bindgen(getter)]
    pub fn currency(&self) -> String {
        self.currency.clone()
    }
}

// Airdrop status
#[wasm_bindgen]
pub struct AirdropStatus {
    airdrop_id: String,
    total_amount: f64,
    distributed_amount: f64,
    total_recipients: usize,
    claimed_recipients: usize,
    is_paused: bool,
    progress_percentage: f64,
}

#[wasm_bindgen]
impl AirdropStatus {
    #[wasm_bindgen(constructor)]
    pub fn new(
        airdrop_id: String,
        total_amount: f64,
        distributed_amount: f64,
        total_recipients: usize,
        claimed_recipients: usize,
        is_paused: bool,
        progress_percentage: f64,
    ) -> AirdropStatus {
        AirdropStatus {
            airdrop_id,
            total_amount,
            distributed_amount,
            total_recipients,
            claimed_recipients,
            is_paused,
            progress_percentage,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn airdrop_id(&self) -> String {
        self.airdrop_id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn total_amount(&self) -> f64 {
        self.total_amount
    }

    #[wasm_bindgen(getter)]
    pub fn distributed_amount(&self) -> f64 {
        self.distributed_amount
    }

    #[wasm_bindgen(getter)]
    pub fn total_recipients(&self) -> usize {
        self.total_recipients
    }

    #[wasm_bindgen(getter)]
    pub fn claimed_recipients(&self) -> usize {
        self.claimed_recipients
    }

    #[wasm_bindgen(getter)]
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    #[wasm_bindgen(getter)]
    pub fn progress_percentage(&self) -> f64 {
        self.progress_percentage
    }
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to P-Project!", name)
}

#[wasm_bindgen]
pub fn initialize_app() {
    log("P-Project Web App Initialized");
}

// DAO Governance functions
#[wasm_bindgen]
pub fn create_proposal(title: &str, description: &str, creator_id: &str) -> Proposal {
    let now = chrono::Utc::now();
    Proposal::new(
        "proposal-1".to_string(), // In real implementation, this would be generated
        title.to_string(),
        description.to_string(),
        creator_id.to_string(),
        now.to_rfc3339(),
        (now + chrono::Duration::days(7)).to_rfc3339(), // 7-day voting period
        "Active".to_string(),
    )
}

#[wasm_bindgen]
pub fn vote_on_proposal(proposal_id: &str, user_id: &str, approve: bool) -> bool {
    // In a real implementation, this would call the backend API
    log(&format!(
        "User {} voted {} on proposal {}",
        user_id,
        if approve { "YES" } else { "NO" },
        proposal_id
    ));
    true // Success
}

#[wasm_bindgen]
pub fn delegate_vote(from_user_id: &str, to_user_id: &str) -> bool {
    // In a real implementation, this would call the backend API
    log(&format!(
        "User {} delegated vote to user {}",
        from_user_id, to_user_id
    ));
    true // Success
}

// Staking functions
#[wasm_bindgen]
pub fn calculate_staking_yield(amount: f64, duration_days: i64) -> StakingYieldResult {
    // Simple calculation for demonstration
    let apy_rate = 0.1; // 10% APY
    let years = duration_days as f64 / 365.0;
    let projected_rewards = amount * apy_rate * years;
    let total_return = amount + projected_rewards;
    
    StakingYieldResult::new(
        amount,
        duration_days,
        projected_rewards,
        total_return,
        apy_rate,
    )
}

// Airdrop functions
#[wasm_bindgen]
pub fn get_airdrop_status() -> AirdropStatus {
    AirdropStatus::new(
        "airdrop-1".to_string(),
        1000000.0, // total_amount
        250000.0,  // distributed_amount
        10000,     // total_recipients
        2500,      // claimed_recipients
        false,     // is_paused
        25.0,      // progress_percentage
    )
}

// NFT Marketplace functions
#[wasm_bindgen]
pub fn create_nft_collection(
    name: &str,
    symbol: &str,
    creator: &str,
    description: &str,
) -> WebNFTCollection {
    WebNFTCollection::new(
        "collection-1".to_string(), // In real implementation, this would be generated
        name.to_string(),
        symbol.to_string(),
        creator.to_string(),
        description.to_string(),
    )
}

#[wasm_bindgen]
pub fn mint_nft(
    collection_id: &str,
    name: &str,
    description: &str,
    image: &str,
    owner: &str,
    royalty_percentage: f64,
) -> WebNFT {
    WebNFT::new(
        "nft-1".to_string(), // In real implementation, this would be generated
        collection_id.to_string(),
        name.to_string(),
        description.to_string(),
        image.to_string(),
        owner.to_string(),
        owner.to_string(), // creator
        royalty_percentage,
    )
}

#[wasm_bindgen]
pub fn list_nft_for_sale(
    nft_id: &str,
    seller: &str,
    price: f64,
    currency: &str,
) -> WebMarketplaceListing {
    WebMarketplaceListing::new(
        "listing-1".to_string(), // In real implementation, this would be generated
        nft_id.to_string(),
        seller.to_string(),
        price,
        currency.to_string(),
    )
}

#[wasm_bindgen]
pub fn buy_nft(listing_id: &str, buyer: &str) -> bool {
    // In a real implementation, this would call the backend API
    log(&format!(
        "User {} bought NFT from listing {}",
        buyer,
        listing_id
    ));
    true // Success
}

#[wasm_bindgen]
pub fn get_user_nfts(user_id: &str) -> String {
    // In a real implementation, this would fetch from backend
    log(&format!(
        "Fetching NFTs for user {}",
        user_id
    ));
    "[]".to_string() // Return empty array for now
}

#[wasm_bindgen]
pub fn get_active_listings() -> String {
    // In a real implementation, this would fetch from backend
    log("Fetching active marketplace listings");
    "[]".to_string() // Return empty array for now
}

// Liquidity Pool functions
#[wasm_bindgen]
pub fn create_liquidity_pool(
    pool_id: &str,
    token_a: &str,
    token_b: &str,
    fee_tier: f64,
    reward_token: &str,
    total_reward_allocation: f64,
    apr_rate: f64,
) -> bool {
    // In a real implementation, this would call the backend API
    log(&format!(
        "Creating liquidity pool {} for tokens {} and {} with fee tier {}",
        pool_id, token_a, token_b, fee_tier
    ));
    true // Success
}

#[wasm_bindgen]
pub fn add_liquidity(
    pool_id: &str,
    user_id: &str,
    token_a_amount: f64,
    token_b_amount: f64,
    duration_days: i64,
) -> bool {
    // In a real implementation, this would call the backend API
    log(&format!(
        "Adding liquidity to pool {}: {} of {} and {} of {} for {} days",
        pool_id, token_a_amount, token_a_amount, token_b_amount, token_b_amount, duration_days
    ));
    true // Success
}

#[wasm_bindgen]
pub fn remove_liquidity(pool_id: &str, user_id: &str) -> bool {
    // In a real implementation, this would call the backend API
    log(&format!(
        "Removing liquidity from pool {} for user {}",
        pool_id, user_id
    ));
    true // Success
}

#[wasm_bindgen]
pub fn swap_tokens(
    pool_id: &str,
    input_token: &str,
    input_amount: f64,
    user_id: &str,
) -> f64 {
    // In a real implementation, this would call the backend API
    log(&format!(
        "Swapping {} {} in pool {} for user {}",
        input_amount, input_token, pool_id, user_id
    ));
    input_amount * 0.997 // Simulate 0.3% fee
}

#[wasm_bindgen]
pub fn get_pool_stats(pool_id: &str) -> String {
    // In a real implementation, this would fetch from backend
    log(&format!(
        "Fetching stats for pool {}",
        pool_id
    ));
    // Return mock JSON data
    r#"{"total_liquidity": 1000000.0, "total_volume": 5000000.0, "total_fees": 15000.0, "total_providers": 150, "avg_liquidity": 6666.67, "apr_rate": 0.12}"#.to_string()
}

#[wasm_bindgen]
pub fn get_user_position(pool_id: &str, user_id: &str) -> String {
    // In a real implementation, this would fetch from backend
    log(&format!(
        "Fetching position for user {} in pool {}",
        user_id, pool_id
    ));
    // Return mock JSON data
    r#"{"liquidity_amount": 1000.0, "token_a_amount": 500.0, "token_b_amount": 500.0, "accumulated_rewards": 25.0, "claimed_rewards": 10.0}"#.to_string()
}

#[wasm_bindgen]
pub fn claim_rewards(pool_id: &str, user_id: &str) -> f64 {
    // In a real implementation, this would call the backend API
    log(&format!(
        "Claiming rewards for user {} in pool {}",
        user_id, pool_id
    ));
    25.0 // Mock reward amount
}

#[wasm_bindgen]
pub fn get_claimable_rewards(pool_id: &str, user_id: &str) -> f64 {
    // In a real implementation, this would fetch from backend
    log(&format!(
        "Fetching claimable rewards for user {} in pool {}",
        user_id, pool_id
    ));
    25.0 // Mock claimable amount
}

#[wasm_bindgen]
pub fn calculate_projected_yield(liquidity_amount: f64, days: f64, apr_rate: f64) -> f64 {
    // Calculate yield using compound interest formula
    let principal = liquidity_amount;
    let rate = apr_rate;
    let n = 365.0; // Daily compounding
    let t = days / 365.0; // Time in years
    
    let yield_amount = principal * ((1.0 + rate / n).powf(n * t) - 1.0);
    
    yield_amount
}
