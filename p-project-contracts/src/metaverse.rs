use crate::token::{PProjectToken, TokenError};
use chrono::{NaiveDateTime, Utc};
use p_project_core::utils::generate_id;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Types of buildings that can be erected on Peace Island parcels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BuildingType {
    Home,
    School,
    Garden,
}

/// Represents an individual structure built by a landowner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub building_type: BuildingType,
    pub name: String,
    pub built_at: NaiveDateTime,
}

/// Status to track whether land is still available for purchase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandParcel {
    pub id: String,
    pub coordinates: (f64, f64),
    pub size_sqm: f64,
    pub price: f64,
    pub owner: Option<String>,
    pub buildings: Vec<Building>,
    pub locked: bool,
}

impl LandParcel {
    pub fn is_available(&self) -> bool {
        self.owner.is_none() && !self.locked
    }
}

/// Errors raised by Peace Island world operations.
#[derive(Debug, Clone, PartialEq)]
pub enum MetaverseError {
    ParcelNotFound,
    ParcelAlreadyOwned,
    UnauthorizedOwner,
    BuildLimitReached,
    TokenError(TokenError),
}

impl fmt::Display for MetaverseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetaverseError::ParcelNotFound => write!(f, "Land parcel not found"),
            MetaverseError::ParcelAlreadyOwned => write!(f, "Parcel is already owned"),
            MetaverseError::UnauthorizedOwner => write!(f, "Only parcel owner may build"),
            MetaverseError::BuildLimitReached => write!(f, "Structural limit reached on parcel"),
            MetaverseError::TokenError(err) => write!(f, "Token error: {}", err),
        }
    }
}

impl From<TokenError> for MetaverseError {
    fn from(err: TokenError) -> Self {
        MetaverseError::TokenError(err)
    }
}

/// World model for Peace Island virtual world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeaceIsland {
    pub name: String,
    pub treasury_address: String,
    pub parcels: HashMap<String, LandParcel>,
    pub land_limit: usize,
    pub builders: HashSet<String>,
}

impl PeaceIsland {
    pub fn new(name: String, treasury_address: String, land_limit: usize) -> Self {
        Self {
            name,
            treasury_address,
            parcels: HashMap::new(),
            land_limit,
            builders: HashSet::new(),
        }
    }

    /// Adds a new parcel onto the island so players can buy it.
    pub fn add_parcel(&mut self, parcel: LandParcel) -> Result<(), MetaverseError> {
        if self.parcels.len() >= self.land_limit {
            return Err(MetaverseError::BuildLimitReached);
        }
        self.parcels.insert(parcel.id.clone(), parcel);
        Ok(())
    }

    /// Allows a user to pay for a parcel with P-Coin.
    pub fn buy_land(
        &mut self,
        parcel_id: &str,
        buyer_id: &str,
        token_contract: &mut PProjectToken,
    ) -> Result<f64, MetaverseError> {
        let parcel = self
            .parcels
            .get_mut(parcel_id)
            .ok_or(MetaverseError::ParcelNotFound)?;

        if !parcel.is_available() {
            return Err(MetaverseError::ParcelAlreadyOwned);
        }

        token_contract.transfer(buyer_id, &self.treasury_address, parcel.price)?;
        parcel.owner = Some(buyer_id.to_string());
        Ok(parcel.price)
    }

    /// Builds a structure on a parcel (home, school, garden).
    pub fn build_structure(
        &mut self,
        parcel_id: &str,
        owner_id: &str,
        building_type: BuildingType,
        name: String,
    ) -> Result<(), MetaverseError> {
        let parcel = self
            .parcels
            .get_mut(parcel_id)
            .ok_or(MetaverseError::ParcelNotFound)?;

        if parcel.owner.as_deref() != Some(owner_id) {
            return Err(MetaverseError::UnauthorizedOwner);
        }

        if parcel.buildings.len() >= 4 {
            return Err(MetaverseError::BuildLimitReached);
        }

        let building = Building {
            id: generate_id(),
            building_type,
            name,
            built_at: Utc::now().naive_utc(),
        };
        parcel.buildings.push(building);
        self.builders.insert(owner_id.to_string());
        Ok(())
    }

    /// Returns list of parcel IDs owned by a user.
    pub fn owner_parcels(&self, owner_id: &str) -> Vec<&LandParcel> {
        self.parcels
            .values()
            .filter(|parcel| parcel.owner.as_deref() == Some(owner_id))
            .collect()
    }

    /// Total P-Coin value of all parcels owned by the DAO treasury.
    pub fn total_land_value(&self) -> f64 {
        self.parcels.values().map(|p| p.price).sum()
    }

    /// Fetch a parcel for reading.
    pub fn get_parcel(&self, parcel_id: &str) -> Option<&LandParcel> {
        self.parcels.get(parcel_id)
    }

    /// Register additional builders who can earn incentives.
    pub fn authorize_builder(&mut self, builder_id: String) {
        self.builders.insert(builder_id);
    }
}
