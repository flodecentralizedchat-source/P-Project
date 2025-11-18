//! Exchange Listing service encapsulates DEX/CEX plans, launch guidelines, and fee budgeting.
//!
//! Implements the table-driven features requested in the growth roadmap.

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::generate_id;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ListingFeatureType {
    Primary,
    Phase2,
    Efficiency,
    Reference,
    Cost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeListingFeature {
    pub id: String,
    pub name: String,
    pub feature_type: ListingFeatureType,
    pub details: String,
    pub purpose: String,
    pub platform: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeeStatus {
    Planned,
    Reserved,
    Spent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListingFeePlan {
    pub id: String,
    pub title: String,
    pub min_usd: f64,
    pub max_usd: f64,
    pub status: FeeStatus,
    pub notes: String,
    pub last_updated: NaiveDateTime,
}

#[derive(Default)]
pub struct ExchangeListingService {
    features: HashMap<String, ExchangeListingFeature>,
    fee_plan: ListingFeePlan,
}

impl ExchangeListingService {
    pub fn new() -> Self {
        let now = Utc::now().naive_utc();
        let fee_plan = ListingFeePlan {
            id: generate_id(),
            title: "Listing Fee Budget".into(),
            min_usd: 0.0,
            max_usd: 5000.0,
            status: FeeStatus::Planned,
            notes: "Budget range covers DEX launch fees.".into(),
            last_updated: now,
        };
        let mut svc = Self {
            features: HashMap::new(),
            fee_plan,
        };
        svc.add_default_features(now);
        svc
    }

    fn add_default_features(&mut self, now: NaiveDateTime) {
        let defaults = vec![
            ExchangeListingFeature {
                id: generate_id(),
                name: "DEX Listing".into(),
                feature_type: ListingFeatureType::Primary,
                details: "Monkol-DEX".into(),
                purpose: "Initial liquidity".into(),
                platform: Some("Monkol-Chain".into()),
                created_at: now,
            },
            ExchangeListingFeature {
                id: generate_id(),
                name: "CEX Listing".into(),
                feature_type: ListingFeatureType::Phase2,
                details: "After stability".into(),
                purpose: "Wider exposure".into(),
                platform: None,
                created_at: now,
            },
            ExchangeListingFeature {
                id: generate_id(),
                name: "DEX Launch Strategy".into(),
                feature_type: ListingFeatureType::Efficiency,
                details: "Launch on low-cost platform".into(),
                purpose: "Minimize listing fees".into(),
                platform: None,
                created_at: now,
            },
            ExchangeListingFeature {
                id: generate_id(),
                name: "DEX Listing Guide".into(),
                feature_type: ListingFeatureType::Reference,
                details: "Major platforms overview".into(),
                purpose: "Listing readiness".into(),
                platform: None,
                created_at: now,
            },
            ExchangeListingFeature {
                id: generate_id(),
                name: "Listing Fees".into(),
                feature_type: ListingFeatureType::Cost,
                details: "0-5000 USD".into(),
                purpose: "Budget planning".into(),
                platform: None,
                created_at: now,
            },
        ];
        for feature in defaults {
            self.features.insert(feature.id.clone(), feature);
        }
    }

    pub fn list_features(&self) -> Vec<ExchangeListingFeature> {
        let mut v: Vec<_> = self.features.values().cloned().collect();
        v.sort_by_key(|f| f.name.clone());
        v
    }

    pub fn features_by_type(
        &self,
        feature_type: ListingFeatureType,
    ) -> Vec<ExchangeListingFeature> {
        let mut v: Vec<_> = self
            .features
            .values()
            .filter(|f| f.feature_type == feature_type)
            .cloned()
            .collect();
        v.sort_by_key(|f| f.created_at);
        v
    }

    pub fn add_feature(&mut self, feature: ExchangeListingFeature) {
        self.features.insert(feature.id.clone(), feature);
    }

    pub fn get_feature_by_name(&self, name: &str) -> Option<ExchangeListingFeature> {
        self.features
            .values()
            .find(|f| f.name.eq_ignore_ascii_case(name))
            .cloned()
    }

    pub fn fee_plan(&self) -> ListingFeePlan {
        self.fee_plan.clone()
    }

    pub fn update_fee_plan(
        &mut self,
        min_usd: f64,
        max_usd: f64,
        status: FeeStatus,
        notes: Option<String>,
    ) -> Result<ListingFeePlan, String> {
        if min_usd < 0.0 || max_usd < 0.0 {
            return Err("fee_bounds_illegal".into());
        }
        if max_usd < min_usd {
            return Err("max_lt_min".into());
        }
        self.fee_plan.min_usd = min_usd;
        self.fee_plan.max_usd = max_usd;
        self.fee_plan.status = status;
        self.fee_plan.notes = notes.unwrap_or_else(|| self.fee_plan.notes.clone());
        self.fee_plan.last_updated = Utc::now().naive_utc();
        Ok(self.fee_plan.clone())
    }
}
