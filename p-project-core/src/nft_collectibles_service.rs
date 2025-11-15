//! NFT collectibles for peace-building heroes, art, and badges.
//!
//! Includes minting of peace hero avatars, NGO-backed art, and Medal of Peace badges.

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NftCollectiblesConfig {
    pub max_mints_per_owner: usize,
}

impl Default for NftCollectiblesConfig {
    fn default() -> Self {
        Self {
            max_mints_per_owner: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PeaceNFTType {
    HeroAvatar,
    ArtCollection,
    MedalOfPeace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeaceNFT {
    pub id: String,
    pub nft_type: PeaceNFTType,
    pub title: String,
    pub owner_id: String,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
    pub supporting_ngo: Option<String>,
    pub hero_power: Option<String>,
    pub badge_level: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NgoContribution {
    pub total_amount: f64,
    pub last_donation: NaiveDateTime,
}

pub struct NftCollectiblesService {
    config: NftCollectiblesConfig,
    nfts: HashMap<String, PeaceNFT>,
    ngo_contributions: HashMap<String, NgoContribution>,
    owner_counts: HashMap<String, usize>,
}

impl NftCollectiblesService {
    pub fn new(config: NftCollectiblesConfig) -> Self {
        Self {
            config,
            nfts: HashMap::new(),
            ngo_contributions: HashMap::new(),
            owner_counts: HashMap::new(),
        }
    }

    fn now() -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    fn ensure_owner_limit(&mut self, owner_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let count = self.owner_counts.entry(owner_id.to_string()).or_insert(0);
        if *count >= self.config.max_mints_per_owner {
            return Err("Owner reached max NFT minting limit".into());
        }
        *count += 1;
        Ok(())
    }

    fn record_nft(&mut self, nft: PeaceNFT) -> PeaceNFT {
        self.nfts.insert(nft.id.clone(), nft.clone());
        nft
    }

    pub fn mint_peace_hero_avatar(
        &mut self,
        hero_name: String,
        owner_id: String,
        hero_power: String,
        description: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<PeaceNFT, Box<dyn std::error::Error>> {
        self.ensure_owner_limit(&owner_id)?;
        let nft = PeaceNFT {
            id: format!("nft_{}", Uuid::new_v4()),
            nft_type: PeaceNFTType::HeroAvatar,
            title: hero_name.clone(),
            owner_id: owner_id.clone(),
            description,
            metadata,
            created_at: Self::now(),
            supporting_ngo: None,
            hero_power: Some(hero_power),
            badge_level: None,
        };
        Ok(self.record_nft(nft))
    }

    pub fn mint_peace_art_collection(
        &mut self,
        collection_name: String,
        owner_id: String,
        supporting_ngo: String,
        description: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<PeaceNFT, Box<dyn std::error::Error>> {
        self.ensure_owner_limit(&owner_id)?;
        let nft = PeaceNFT {
            id: format!("nft_{}", Uuid::new_v4()),
            nft_type: PeaceNFTType::ArtCollection,
            title: collection_name,
            owner_id: owner_id.clone(),
            description,
            metadata,
            created_at: Self::now(),
            supporting_ngo: Some(supporting_ngo.clone()),
            hero_power: None,
            badge_level: None,
        };
        self.ngo_contributions
            .entry(supporting_ngo.clone())
            .or_insert(NgoContribution {
                total_amount: 0.0,
                last_donation: Self::now(),
            });
        Ok(self.record_nft(nft))
    }

    pub fn mint_medal_of_peace(
        &mut self,
        holder_name: String,
        owner_id: String,
        badge_level: String,
        description: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<PeaceNFT, Box<dyn std::error::Error>> {
        self.ensure_owner_limit(&owner_id)?;
        let nft = PeaceNFT {
            id: format!("nft_{}", Uuid::new_v4()),
            nft_type: PeaceNFTType::MedalOfPeace,
            title: holder_name,
            owner_id: owner_id.clone(),
            description,
            metadata,
            created_at: Self::now(),
            supporting_ngo: None,
            hero_power: None,
            badge_level: Some(badge_level),
        };
        Ok(self.record_nft(nft))
    }

    pub fn transfer_nft(
        &mut self,
        nft_id: &str,
        new_owner: String,
    ) -> Result<&PeaceNFT, Box<dyn std::error::Error>> {
        let nft = self.nfts.get_mut(nft_id).ok_or("NFT not found")?;
        nft.owner_id = new_owner.clone();
        Ok(nft)
    }

    pub fn donate_to_ngo_via_nft(
        &mut self,
        nft_id: &str,
        amount: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if amount <= 0.0 {
            return Err("Donation amount must be positive".into());
        }
        let nft = self.nfts.get(nft_id).ok_or("NFT not found")?;
        let ngo = nft
            .supporting_ngo
            .as_ref()
            .ok_or("NFT is not linked to an NGO")?;
        let entry = self
            .ngo_contributions
            .entry(ngo.clone())
            .or_insert(NgoContribution {
                total_amount: 0.0,
                last_donation: Self::now(),
            });
        entry.total_amount += amount;
        entry.last_donation = Self::now();
        Ok(())
    }

    pub fn list_nfts_by_owner(&self, owner_id: &str) -> Vec<&PeaceNFT> {
        self.nfts
            .values()
            .filter(|nft| nft.owner_id == owner_id)
            .collect()
    }

    pub fn ngo_support_summary(&self, ngo_id: &str) -> Option<&NgoContribution> {
        self.ngo_contributions.get(ngo_id)
    }

    pub fn get_nft(&self, nft_id: &str) -> Option<&PeaceNFT> {
        self.nfts.get(nft_id)
    }
}
