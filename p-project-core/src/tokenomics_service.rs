use csv::StringRecord;
use serde::Serialize;
use std::cmp::Ordering;
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
pub struct LpRatios {
    pub initial_lp_usdt: f64,
    pub initial_lp_tokens: f64,
    pub starting_price: f64,
    pub computed_price: f64,
    pub ratio_consistent: bool,
    pub notes: Vec<String>,
}

#[derive(Default)]
struct LpRatioInputs {
    initial_lp_usdt: Option<f64>,
    initial_lp_tokens: Option<f64>,
    starting_price: Option<f64>,
    notes: Vec<String>,
}

impl LpRatioInputs {
    fn push_notes(&mut self, item: &str, extras: &[String]) {
        if extras.is_empty() {
            return;
        }
        self.notes
            .push(format!("{} notes: {}", item, extras.join("; ")));
    }
}

impl LpRatios {
    const PRICE_TOLERANCE: f64 = 1e-9;

    fn from_inputs(inputs: LpRatioInputs) -> Option<Self> {
        let initial_lp_usdt = inputs.initial_lp_usdt?;
        let initial_lp_tokens = inputs.initial_lp_tokens?;
        let starting_price = inputs.starting_price?;

        let computed_price = if initial_lp_tokens.abs() < f64::EPSILON {
            0.0
        } else {
            initial_lp_usdt / initial_lp_tokens
        };
        let ratio_consistent = (computed_price - starting_price).abs() <= Self::PRICE_TOLERANCE;

        let mut notes = inputs.notes;
        if notes.is_empty() {
            notes.push("LP ratio metadata recorded without supplementary notes".to_string());
        }
        notes.push(format!(
            "Derived formula: {:.8} = {:.2} / {:.0}",
            computed_price, initial_lp_usdt, initial_lp_tokens
        ));

        Some(Self {
            initial_lp_usdt,
            initial_lp_tokens,
            starting_price,
            computed_price,
            ratio_consistent,
            notes,
        })
    }
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
    pub market_cap_required: Option<f64>,
    pub mechanisms: Option<String>,
    pub components: Option<String>,
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
    pub lp_ratios: Option<LpRatios>,
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
    market_cap_required: Option<f64>,
    mechanisms: Option<String>,
    components: Option<String>,
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
        let mut lp_ratio_inputs = LpRatioInputs::default();
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
                &mut lp_ratio_inputs,
                &mut launch_strategy,
                &mut price_targets_raw,
            )?;
        }

        let total_supply = total_supply.ok_or(TokenomicsError::MissingTotalSupply)?;

        let lp_ratios = LpRatios::from_inputs(lp_ratio_inputs);

        let price_targets = price_targets_raw
            .into_iter()
            .map(|raw| PriceTarget {
                target_price: raw.target_price,
                market_cap_required: raw.market_cap_required,
                mechanisms: raw.mechanisms,
                components: raw.components,
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
                lp_ratios,
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

    pub fn price_target_for_price(&self, price: f64) -> Option<&PriceTarget> {
        self.price_targets
            .iter()
            .find(|target| (target.target_price - price).abs() < f64::EPSILON)
    }

    pub fn market_cap_gap_to_target(&self, price: f64, actual_market_cap: f64) -> Option<f64> {
        self.price_target_for_price(price).and_then(|target| {
            target
                .market_cap_required
                .map(|required| required - actual_market_cap)
        })
    }

    pub fn next_price_target(&self, current_price: f64) -> Option<&PriceTarget> {
        self.price_targets
            .iter()
            .filter(|stage| stage.target_price > current_price)
            .min_by(|a, b| {
                a.target_price
                    .partial_cmp(&b.target_price)
                    .unwrap_or(Ordering::Equal)
            })
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
        lp_ratio_inputs: &mut LpRatioInputs,
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
                    .collect::<Vec<_>>();
                lp_ratio_inputs.push_notes(&item, &notes);

                match item.trim().to_uppercase().as_str() {
                    "INITIAL LP USDT" => {
                        let parsed = parse_float(value).ok_or_else(|| {
                            TokenomicsError::ParseError("invalid LP USDT value".to_string())
                        })?;
                        lp_ratio_inputs.initial_lp_usdt = Some(parsed);
                    }
                    "INITIAL LP TOKENS" => {
                        let parsed = parse_float(value).ok_or_else(|| {
                            TokenomicsError::ParseError("invalid LP Tokens value".to_string())
                        })?;
                        lp_ratio_inputs.initial_lp_tokens = Some(parsed);
                    }
                    "STARTING PRICE" => {
                        let parsed = parse_float(value).ok_or_else(|| {
                            TokenomicsError::ParseError("invalid Starting Price".to_string())
                        })?;
                        lp_ratio_inputs.starting_price = Some(parsed);
                    }
                    _ => {}
                }

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
                let market_cap_required = record.get(3).and_then(|s| parse_float(s));
                let mechanisms = record
                    .get(4)
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                let components = record
                    .get(5)
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                let note = record
                    .get(6)
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty());
                price_targets.push(PriceTargetRecord {
                    target_price,
                    market_cap_required,
                    mechanisms,
                    components,
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
    if trimmed.is_empty() {
        return None;
    }

    let mut cleaned = trimmed.replace(',', "").replace('_', "");
    let mut multiplier = 1.0;

    if let Some(last_char) = cleaned.chars().last() {
        match last_char.to_ascii_uppercase() {
            'K' => {
                multiplier = 1_000.0;
                cleaned.pop();
            }
            'M' => {
                multiplier = 1_000_000.0;
                cleaned.pop();
            }
            'B' => {
                multiplier = 1_000_000_000.0;
                cleaned.pop();
            }
            'T' => {
                multiplier = 1_000_000_000_000.0;
                cleaned.pop();
            }
            _ => {}
        }
    }

    let cleaned = cleaned.trim_start_matches('$').trim_end_matches('%').trim();

    if cleaned.is_empty() {
        return None;
    }

    cleaned.parse::<f64>().ok().map(|value| value * multiplier)
}
