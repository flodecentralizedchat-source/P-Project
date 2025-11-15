use chrono::Utc;
use csv::StringRecord;
use serde::Serialize;
use std::error::Error;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

#[derive(Debug, Serialize, Clone)]
pub enum TokenomicsSection {
    Tokenomics,
    Vesting,
    LpRatios,
    LaunchStrategy,
    PriceModel,
    Unknown(String),
}

impl TokenomicsSection {
    fn from_str(value: &str) -> Self {
        match value.trim().to_uppercase().as_str() {
            "TOKENOMICS" => TokenomicsSection::Tokenomics,
            "VESTING" => TokenomicsSection::Vesting,
            "LP_RATIOS" => TokenomicsSection::LpRatios,
            "LAUNCH_STRATEGY" => TokenomicsSection::LaunchStrategy,
            "PRICE_MODEL" => TokenomicsSection::PriceModel,
            other => TokenomicsSection::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct TokenAllocation {
    pub name: String,
    pub amount: f64,
    pub percent: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct VestingSchedule {
    pub name: String,
    pub percent: String,
    pub duration: Option<String>,
    pub cliff: Option<String>,
    pub cadence: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct LpMetric {
    pub name: String,
    pub value: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct LaunchStep {
    pub stage: String,
    pub detail: String,
    pub note: Option<String>,
    pub extra: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PriceTarget {
    pub target_price: f64,
    pub requirement: String,
    pub note: Option<String>,
    pub market_cap: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct TokenomicsSummary {
    pub total_supply: f64,
    pub circulating_at_launch: Option<f64>,
    pub allocations: Vec<TokenAllocation>,
    pub vesting: Vec<VestingSchedule>,
    pub lp_details: Vec<LpMetric>,
    pub launch_strategy: Vec<LaunchStep>,
    pub price_targets: Vec<PriceTarget>,
}

#[derive(Debug)]
pub enum TokenomicsError {
    MissingTotalSupply,
    ParseError(String),
    Io(io::Error),
}

impl Display for TokenomicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenomicsError::MissingTotalSupply => write!(f, "total supply entry is missing"),
            TokenomicsError::ParseError(err) => write!(f, "parse error: {}", err),
            TokenomicsError::Io(err) => write!(f, "io error: {}", err),
        }
    }
}

impl Error for TokenomicsError {}

impl From<io::Error> for TokenomicsError {
    fn from(err: io::Error) -> Self {
        TokenomicsError::Io(err)
    }
}

#[derive(Debug)]
struct PriceTargetRecord {
    target_price: f64,
    requirement: String,
    note: Option<String>,
}

pub struct TokenomicsService {
    summary: TokenomicsSummary,
}

impl TokenomicsService {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, TokenomicsError> {
        let file = File::open(path)?;
        Self::from_reader(file)
    }

    pub fn from_reader(reader: impl Read) -> Result<Self, TokenomicsError> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .trim(csv::Trim::All)
            .from_reader(reader);
        let mut allocations = Vec::new();
        let mut vesting = Vec::new();
        let mut lp_details = Vec::new();
        let mut launch_strategy = Vec::new();
        let mut price_targets_raw = Vec::new();
        let mut total_supply: Option<f64> = None;
        let mut circulating: Option<f64> = None;

        for result in rdr.records() {
            let record = result.map_err(|e| TokenomicsError::ParseError(e.to_string()))?;
            Self::process_record(
                &record,
                &mut total_supply,
                &mut circulating,
                &mut allocations,
                &mut vesting,
                &mut lp_details,
                &mut launch_strategy,
                &mut price_targets_raw,
            )?;
        }

        let total_supply = total_supply.ok_or(TokenomicsError::MissingTotalSupply)?;

        let price_targets = price_targets_raw
            .into_iter()
            .map(|raw| PriceTarget {
                target_price: raw.target_price,
                requirement: raw.requirement,
                note: raw.note,
                market_cap: raw.target_price * total_supply,
            })
            .collect();

        Ok(TokenomicsService {
            summary: TokenomicsSummary {
                total_supply,
                circulating_at_launch: circulating,
                allocations,
                vesting,
                lp_details,
                launch_strategy,
                price_targets,
            },
        })
    }

    pub fn summary(&self) -> &TokenomicsSummary {
        &self.summary
    }
}

impl TokenomicsSummary {
    pub fn market_cap_for_price(&self, price: f64) -> f64 {
        self.total_supply * price
    }

    pub fn price_for_market_cap(&self, market_cap: f64) -> f64 {
        market_cap / self.total_supply
    }
}

impl TokenomicsService {
    fn process_record(
        record: &StringRecord,
        total_supply: &mut Option<f64>,
        circulating: &mut Option<f64>,
        allocations: &mut Vec<TokenAllocation>,
        vesting: &mut Vec<VestingSchedule>,
        lp_details: &mut Vec<LpMetric>,
        launch_strategy: &mut Vec<LaunchStep>,
        price_targets: &mut Vec<PriceTargetRecord>,
    ) -> Result<(), TokenomicsError> {
        let section = TokenomicsSection::from_str(&record[0]);
        let item = record[1].trim().to_string();
        let value = record[2].trim();

        match section {
            TokenomicsSection::Tokenomics => match item.as_str() {
                "Total Supply" => *total_supply = parse_float(value),
                "Circulating at Launch" => *circulating = parse_float(value),
                other => {
                    let amount = parse_float(value).ok_or_else(|| {
                        TokenomicsError::ParseError(format!(
                            "invalid token allocation value for {}",
                            other
                        ))
                    })?;
                    let percent = record
                        .get(3)
                        .map(|p| p.trim().to_string())
                        .filter(|p| !p.is_empty());
                    let notes = record
                        .iter()
                        .skip(4)
                        .map(|n| n.trim().to_string())
                        .filter(|n| !n.is_empty())
                        .collect();
                    allocations.push(TokenAllocation {
                        name: other.to_string(),
                        amount,
                        percent,
                        notes,
                    });
                }
            },
            TokenomicsSection::Vesting => {
                vesting.push(VestingSchedule {
                    name: item,
                    percent: value.to_string(),
                    duration: record
                        .get(3)
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()),
                    cliff: record
                        .get(4)
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()),
                    cadence: record
                        .get(5)
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty()),
                });
            }
            TokenomicsSection::LpRatios => {
                let notes = record
                    .iter()
                    .skip(3)
                    .map(|n| n.trim().to_string())
                    .filter(|n| !n.is_empty())
                    .collect();
                lp_details.push(LpMetric {
                    name: item,
                    value: value.to_string(),
                    notes,
                });
            }
            TokenomicsSection::LaunchStrategy => {
                let extras: Vec<String> = record
                    .iter()
                    .skip(3)
                    .map(|n| n.trim().to_string())
                    .filter(|n| !n.is_empty())
                    .collect();
                launch_strategy.push(LaunchStep {
                    stage: item,
                    detail: value.to_string(),
                    note: extras.get(0).cloned(),
                    extra: extras.get(1).cloned(),
                });
            }
            TokenomicsSection::PriceModel => {
                let target_price = parse_float(value)
                    .ok_or_else(|| TokenomicsError::ParseError("invalid price target".into()))?;
                let requirement = record
                    .get(3)
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_default();
                let note = record
                    .get(4)
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                price_targets.push(PriceTargetRecord {
                    target_price,
                    requirement,
                    note,
                });
            }
            TokenomicsSection::Unknown(_) => {}
        }

        Ok(())
    }
}

fn parse_float(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    let no_commas = trimmed.replace(',', "");
    let no_dollar = no_commas.trim_start_matches('$');
    let no_percent = no_dollar.trim_end_matches('%');
    let cleaned = no_percent.trim();
    cleaned.parse::<f64>().ok()
}
