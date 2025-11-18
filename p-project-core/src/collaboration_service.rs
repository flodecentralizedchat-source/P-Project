//! Cross-Project Collaboration service for P-Project
//!
//! Implements:
//! - Complementary Projects: partner profiling and complementarity linking/search
//! - Technology Integrations: DeFi/tool integrations with verification
//! - Community Cross-Promotion: joint campaigns and metrics

use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{
    marketing_service::SocialPlatform,
    models::{Partner, PartnerIntegrationType},
    utils::generate_id,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartnerProfile {
    pub partner: Partner,
    pub primary_market: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Complementarity {
    pub id: String,
    pub partner_a_id: String,
    pub partner_b_id: String,
    pub shared_tags: Vec<String>,
    pub different_markets: bool,
    pub score: f32, // 0.0 - 1.0 similarity score
    pub created_at: NaiveDateTime,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntegrationStatus {
    Planned,
    Integrated,
    Verified,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechIntegration {
    pub id: String,
    pub partner_id: String,
    pub protocol_name: String,
    pub capabilities: Vec<String>,
    pub status: IntegrationStatus,
    pub created_at: NaiveDateTime,
    pub verified_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CrossPromoEventType {
    Impression,
    Click,
    Signup,
    Conversion,
    Volume, // value carries the amount/TVL/etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPromoEvent {
    pub timestamp: NaiveDateTime,
    pub event_type: CrossPromoEventType,
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPromoCampaign {
    pub id: String,
    pub partner_id: String,
    pub title: String,
    pub description: String,
    pub channels: Vec<SocialPlatform>,
    pub start_date: NaiveDateTime,
    pub end_date: Option<NaiveDateTime>,
    pub reward_split_pct: f32, // 0..=100 shared incentive
    pub events: Vec<CrossPromoEvent>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrossPromoMetrics {
    pub impressions: u64,
    pub clicks: u64,
    pub signups: u64,
    pub conversions: u64,
    pub volume: f64,
    pub ctr: f32,
    pub conversion_rate: f32,
}

#[derive(Default)]
pub struct CollaborationService {
    partners: HashMap<String, PartnerProfile>,
    integrations: HashMap<String, TechIntegration>,
    complementaries: HashMap<String, Complementarity>,
    campaigns: HashMap<String, CrossPromoCampaign>,
}

impl CollaborationService {
    pub fn new() -> Self {
        Self::default()
    }

    // ----- Partners & Complementary Projects -----

    pub fn register_partner(
        &mut self,
        name: &str,
        integration_type: PartnerIntegrationType,
        primary_market: &str,
        tags: Vec<String>,
        webhook_secret: Option<String>,
        metadata: serde_json::Value,
    ) -> PartnerProfile {
        let id = generate_id();
        let partner = Partner {
            id: id.clone(),
            name: name.to_string(),
            integration_type,
            metadata,
            webhook_secret,
            active: true,
            created_at: Utc::now().naive_utc(),
        };
        let profile = PartnerProfile {
            partner,
            primary_market: primary_market.to_string(),
            tags,
        };
        self.partners.insert(id.clone(), profile.clone());
        profile
    }

    pub fn set_webhook_secret(&mut self, partner_id: &str, secret: &str) -> Result<(), String> {
        let p = self
            .partners
            .get_mut(partner_id)
            .ok_or_else(|| "partner_not_found".to_string())?;
        p.partner.webhook_secret = Some(secret.to_string());
        Ok(())
    }

    pub fn get_partner(&self, partner_id: &str) -> Option<PartnerProfile> {
        self.partners.get(partner_id).cloned()
    }

    pub fn search_complementary(
        &self,
        required_tag: Option<&str>,
        exclude_market: Option<&str>,
    ) -> Vec<PartnerProfile> {
        let mut v: Vec<_> = self
            .partners
            .values()
            .filter(|p| match exclude_market {
                Some(m) => !p.primary_market.eq_ignore_ascii_case(m),
                None => true,
            })
            .filter(|p| match required_tag {
                Some(tag) => p.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)),
                None => true,
            })
            .cloned()
            .collect();

        // Sort by tag count (descending) as a simple relevance metric
        v.sort_by(|a, b| b.tags.len().cmp(&a.tags.len()));
        v
    }

    pub fn link_complementary(
        &mut self,
        partner_a_id: &str,
        partner_b_id: &str,
        notes: Option<String>,
    ) -> Result<Complementarity, String> {
        if partner_a_id == partner_b_id {
            return Err("cannot_link_same_partner".to_string());
        }
        let a = self
            .partners
            .get(partner_a_id)
            .ok_or_else(|| "partner_a_not_found".to_string())?;
        let b = self
            .partners
            .get(partner_b_id)
            .ok_or_else(|| "partner_b_not_found".to_string())?;

        let set_a: HashSet<String> = a.tags.iter().map(|s| s.to_lowercase()).collect();
        let set_b: HashSet<String> = b.tags.iter().map(|s| s.to_lowercase()).collect();
        let shared: Vec<String> = set_a.intersection(&set_b).cloned().collect::<Vec<_>>();
        let union_size = set_a.union(&set_b).count().max(1) as f32;
        let mut score = (shared.len() as f32) / union_size;
        let different_markets = !a.primary_market.eq_ignore_ascii_case(&b.primary_market);
        if !different_markets {
            // Penalize if same primary market (we want complementary markets)
            score *= 0.5;
        }

        let c = Complementarity {
            id: generate_id(),
            partner_a_id: partner_a_id.to_string(),
            partner_b_id: partner_b_id.to_string(),
            shared_tags: shared,
            different_markets,
            score,
            created_at: Utc::now().naive_utc(),
            notes,
        };
        self.complementaries.insert(c.id.clone(), c.clone());
        Ok(c)
    }

    pub fn list_complementarities_for(&self, partner_id: &str) -> Vec<Complementarity> {
        let mut v: Vec<_> = self
            .complementaries
            .values()
            .filter(|c| c.partner_a_id == partner_id || c.partner_b_id == partner_id)
            .cloned()
            .collect();
        v.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        v
    }

    // ----- Technology Integrations -----

    pub fn integrate_with_protocol(
        &mut self,
        partner_id: &str,
        protocol_name: &str,
        capabilities: Vec<String>,
    ) -> Result<TechIntegration, String> {
        let p = self
            .partners
            .get(partner_id)
            .ok_or_else(|| "partner_not_found".to_string())?;
        // Focused on DeFi integrations per requirements
        if p.partner.integration_type != PartnerIntegrationType::DeFi {
            return Err("not_defi_partner".to_string());
        }

        let integ = TechIntegration {
            id: generate_id(),
            partner_id: partner_id.to_string(),
            protocol_name: protocol_name.to_string(),
            capabilities,
            status: IntegrationStatus::Integrated,
            created_at: Utc::now().naive_utc(),
            verified_at: None,
        };
        self.integrations.insert(integ.id.clone(), integ.clone());
        Ok(integ)
    }

    pub fn verify_integration(
        &mut self,
        integration_id: &str,
        provided_secret: &str,
    ) -> Result<TechIntegration, String> {
        let integ = self
            .integrations
            .get_mut(integration_id)
            .ok_or_else(|| "integration_not_found".to_string())?;
        let partner = self
            .partners
            .get(&integ.partner_id)
            .ok_or_else(|| "partner_not_found".to_string())?;
        let Some(expected) = &partner.partner.webhook_secret else {
            return Err("missing_partner_secret".to_string());
        };
        if expected != provided_secret {
            return Err("invalid_secret".to_string());
        }
        integ.status = IntegrationStatus::Verified;
        integ.verified_at = Some(Utc::now().naive_utc());
        Ok(integ.clone())
    }

    pub fn deprecate_integration(
        &mut self,
        integration_id: &str,
    ) -> Result<TechIntegration, String> {
        let integ = self
            .integrations
            .get_mut(integration_id)
            .ok_or_else(|| "integration_not_found".to_string())?;
        integ.status = IntegrationStatus::Deprecated;
        Ok(integ.clone())
    }

    pub fn get_integration(&self, integration_id: &str) -> Option<TechIntegration> {
        self.integrations.get(integration_id).cloned()
    }

    // ----- Community Cross-Promotion -----

    pub fn create_cross_promo_campaign(
        &mut self,
        partner_id: &str,
        title: &str,
        description: &str,
        channels: Vec<SocialPlatform>,
        duration_days: Option<i64>,
        reward_split_pct: f32,
    ) -> Result<CrossPromoCampaign, String> {
        if !self.partners.contains_key(partner_id) {
            return Err("partner_not_found".to_string());
        }
        let now = Utc::now().naive_utc();
        let end_date = duration_days.map(|d| now + Duration::days(d.max(0)));
        let split = reward_split_pct.clamp(0.0, 100.0);
        let c = CrossPromoCampaign {
            id: generate_id(),
            partner_id: partner_id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            channels,
            start_date: now,
            end_date,
            reward_split_pct: split,
            events: Vec::new(),
            created_at: now,
        };
        self.campaigns.insert(c.id.clone(), c.clone());
        Ok(c)
    }

    pub fn record_cross_promo_event(
        &mut self,
        campaign_id: &str,
        event_type: CrossPromoEventType,
        value: Option<f64>,
    ) -> Result<(), String> {
        let c = self
            .campaigns
            .get_mut(campaign_id)
            .ok_or_else(|| "campaign_not_found".to_string())?;
        c.events.push(CrossPromoEvent {
            timestamp: Utc::now().naive_utc(),
            event_type,
            value,
        });
        Ok(())
    }

    pub fn cross_promo_metrics(&self, campaign_id: &str) -> Result<CrossPromoMetrics, String> {
        let c = self
            .campaigns
            .get(campaign_id)
            .ok_or_else(|| "campaign_not_found".to_string())?;
        let mut impressions = 0u64;
        let mut clicks = 0u64;
        let mut signups = 0u64;
        let mut conversions = 0u64;
        let mut volume = 0.0f64;
        for e in &c.events {
            match e.event_type {
                CrossPromoEventType::Impression => impressions += 1,
                CrossPromoEventType::Click => clicks += 1,
                CrossPromoEventType::Signup => signups += 1,
                CrossPromoEventType::Conversion => conversions += 1,
                CrossPromoEventType::Volume => volume += e.value.unwrap_or(0.0),
            }
        }
        let ctr = if impressions > 0 {
            (clicks as f32) / (impressions as f32)
        } else {
            0.0
        };
        let conversion_rate = if clicks > 0 {
            (conversions as f32) / (clicks as f32)
        } else {
            0.0
        };
        Ok(CrossPromoMetrics {
            impressions,
            clicks,
            signups,
            conversions,
            volume,
            ctr,
            conversion_rate,
        })
    }
}
