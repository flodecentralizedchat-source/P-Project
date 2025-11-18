use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExchangeListings {
    pub dex_strategy: DexStrategy,
    pub cex_progression: CexProgression,
    pub market_makers: MarketMakers,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DexStrategy {
    pub start_with_permissionless: bool,
    pub platforms: Vec<DexPlatform>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DexPlatform {
    pub name: String,
    pub chain: String,
    pub permissionless: bool,
    pub listing_fee_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CexProgression {
    pub pursue_after_dex: bool,
    pub phases: Vec<CexPhase>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CexPhase {
    pub name: String,
    pub prerequisites: Vec<String>,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketMakers {
    pub partner_with_mm: bool,
    pub objectives: Vec<String>,
    pub guidance: MarketMakerGuidance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketMakerGuidance {
    pub target_min_liquidity_usd: f64,
    pub target_spread_bps: u32,
    pub inventory_bounds_pct: (f64, f64),
}

impl ExchangeListings {
    pub fn default() -> Self {
        Self {
            dex_strategy: DexStrategy {
                start_with_permissionless: true,
                platforms: vec![
                    DexPlatform {
                        name: "Uniswap".into(),
                        chain: "Ethereum".into(),
                        permissionless: true,
                        listing_fee_note: "No listing fees; permissionless pool creation".into(),
                    },
                    DexPlatform {
                        name: "PancakeSwap".into(),
                        chain: "BNB Chain".into(),
                        permissionless: true,
                        listing_fee_note: "No listing fees; permissionless pool creation".into(),
                    },
                    DexPlatform {
                        name: "Raydium".into(),
                        chain: "Solana".into(),
                        permissionless: true,
                        listing_fee_note: "No listing fees; permissionless pool creation".into(),
                    },
                ],
            },
            cex_progression: CexProgression {
                pursue_after_dex: true,
                phases: vec![
                    CexPhase {
                        name: "Tier-3/Tier-2".into(),
                        prerequisites: vec![
                            "Sustained DEX volume".into(),
                            "Liquidity depth and LP lock proof".into(),
                            "Community growth and KYC/audit proof".into(),
                        ],
                        examples: vec!["MEXC".into(), "Gate".into(), "Bitmart".into()],
                    },
                    CexPhase {
                        name: "Tier-1".into(),
                        prerequisites: vec![
                            "High daily volume and users".into(),
                            "Robust compliance profile".into(),
                            "Established market-maker support".into(),
                        ],
                        examples: vec!["Kraken".into(), "Coinbase".into(), "Binance".into()],
                    },
                ],
            },
            market_makers: MarketMakers {
                partner_with_mm: true,
                objectives: vec![
                    "Ensure consistent liquidity".into(),
                    "Reduce volatility".into(),
                    "Tight, healthy spreads".into(),
                ],
                guidance: MarketMakerGuidance {
                    target_min_liquidity_usd: 100_000.0,
                    target_spread_bps: 50,
                    inventory_bounds_pct: (0.2, 0.8),
                },
            },
        }
    }

    pub fn from_tokenomics_service(service: &crate::tokenomics_service::TokenomicsService) -> Self {
        let summary = service.summary();
        let mut strategy = Self::default();

        let mut saw_dex = false;
        let mut saw_cex = false;
        let mut saw_mm = false;
        for step in &summary.launch_strategy {
            let stage = step.stage.to_lowercase();
            let detail = step.detail.to_lowercase();
            if stage.contains("dex") && detail.contains("permissionless") {
                saw_dex = true;
            }
            if stage.contains("cex") && detail.contains("listing") {
                saw_cex = true;
            }
            if stage.contains("market") && detail.contains("making") {
                saw_mm = true;
            }
        }

        strategy.dex_strategy.start_with_permissionless =
            saw_dex || strategy.dex_strategy.start_with_permissionless;
        strategy.cex_progression.pursue_after_dex =
            saw_cex || strategy.cex_progression.pursue_after_dex;
        strategy.market_makers.partner_with_mm = saw_mm || strategy.market_makers.partner_with_mm;

        strategy
    }
}
