//! Tokenized impact assets service for P-Coin.
//!
//! Provides:
//! - Carbon credit batch creation and issuance
//! - Charity receipt token minting and redemption
//! - NFT-backed impact bond issuance and purchases

use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TokenizedImpactAssetConfig {
    pub supported_currency: String,
    pub max_carbon_batch_size: f64,
}

impl Default for TokenizedImpactAssetConfig {
    fn default() -> Self {
        Self {
            supported_currency: "P".to_string(),
            max_carbon_batch_size: 1_000_000.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonCreditBatch {
    pub batch_id: String,
    pub project_name: String,
    pub country: String,
    pub verifying_body: String,
    pub vintage_year: u16,
    pub total_credits: f64,
    pub credits_allocated: f64,
    pub currency: String,
    pub issued_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub metadata: Option<serde_json::Value>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarbonCreditIssuance {
    pub issuance_id: String,
    pub batch_id: String,
    pub purchaser_id: String,
    pub credits: f64,
    pub token_reference: String,
    pub issued_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharityReceipt {
    pub receipt_id: String,
    pub donor_id: String,
    pub campaign_name: String,
    pub amount: f64,
    pub currency: String,
    pub issued_at: NaiveDateTime,
    pub metadata: Option<serde_json::Value>,
    pub status: CharityReceiptStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CharityReceiptStatus {
    Active,
    Redeemed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactBond {
    pub bond_id: String,
    pub project_name: String,
    pub issuer: String,
    pub face_value: f64,
    pub coupon_rate: f64,
    pub total_supply: usize,
    pub minted_supply: usize,
    pub currency: String,
    pub maturity_date: NaiveDateTime,
    pub nft_metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactBondNFT {
    pub nft_id: String,
    pub bond_id: String,
    pub owner_id: String,
    pub face_value: f64,
    pub coupon_rate: f64,
    pub maturity_date: NaiveDateTime,
    pub issued_at: NaiveDateTime,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssetTransaction {
    pub transaction_id: String,
    pub asset_type: ImpactAssetType,
    pub target_id: String,
    pub amount: f64,
    pub owner_id: String,
    pub timestamp: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImpactAssetType {
    CarbonCredit,
    CharityReceipt,
    ImpactBondNFT,
}

pub struct TokenizedImpactAssetService {
    config: TokenizedImpactAssetConfig,
    carbon_batches: HashMap<String, CarbonCreditBatch>,
    carbon_issuances: HashMap<String, CarbonCreditIssuance>,
    charity_receipts: HashMap<String, CharityReceipt>,
    impact_bonds: HashMap<String, ImpactBond>,
    bond_nfts: HashMap<String, ImpactBondNFT>,
    transactions: HashMap<String, ImpactAssetTransaction>,
}

impl TokenizedImpactAssetService {
    pub fn new(config: TokenizedImpactAssetConfig) -> Self {
        Self {
            config,
            carbon_batches: HashMap::new(),
            carbon_issuances: HashMap::new(),
            charity_receipts: HashMap::new(),
            impact_bonds: HashMap::new(),
            bond_nfts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    fn now(&self) -> NaiveDateTime {
        Utc::now().naive_utc()
    }

    fn ensure_currency(&self, currency: &str) -> Result<(), Box<dyn std::error::Error>> {
        if currency != self.config.supported_currency {
            Err(format!(
                "Only {} is supported in tokenized assets",
                self.config.supported_currency
            )
            .into())
        } else {
            Ok(())
        }
    }

    fn record_transaction(
        &mut self,
        asset_type: ImpactAssetType,
        target_id: String,
        owner_id: String,
        amount: f64,
    ) -> ImpactAssetTransaction {
        let tx_id = format!("asset_tx_{}", Uuid::new_v4());
        let tx = ImpactAssetTransaction {
            transaction_id: tx_id.clone(),
            asset_type,
            target_id,
            amount,
            owner_id,
            timestamp: self.now(),
        };
        self.transactions.insert(tx_id.clone(), tx.clone());
        tx
    }

    pub fn create_carbon_credit_batch(
        &mut self,
        project_name: String,
        country: String,
        verifying_body: String,
        vintage_year: u16,
        total_credits: f64,
        currency: String,
        expires_in_days: Option<i64>,
        metadata: Option<serde_json::Value>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.ensure_currency(&currency)?;
        if total_credits <= 0.0 {
            return Err("Total credits must be positive".into());
        }
        if total_credits > self.config.max_carbon_batch_size {
            return Err("Batch exceeds maximum allowed size".into());
        }

        let batch_id = format!("carbon_{}", Uuid::new_v4());
        let issued_at = self.now();
        let expires_at = expires_in_days.map(|days| issued_at + Duration::days(days));
        let batch = CarbonCreditBatch {
            batch_id: batch_id.clone(),
            project_name,
            country,
            verifying_body,
            vintage_year,
            total_credits,
            credits_allocated: 0.0,
            currency,
            issued_at,
            expires_at,
            metadata,
            is_active: true,
        };
        self.carbon_batches.insert(batch_id.clone(), batch);
        Ok(batch_id)
    }

    pub fn issue_carbon_credit(
        &mut self,
        batch_id: &str,
        purchaser_id: String,
        credits: f64,
    ) -> Result<CarbonCreditIssuance, Box<dyn std::error::Error>> {
        if credits <= 0.0 {
            return Err("Credits must be positive".into());
        }

        let batch = self
            .carbon_batches
            .get_mut(batch_id)
            .ok_or("Carbon batch not found")?;

        if !batch.is_active {
            return Err("Carbon batch is no longer active".into());
        }

        let remaining = batch.total_credits - batch.credits_allocated;
        if credits > remaining {
            return Err("Not enough credits remaining in batch".into());
        }

        batch.credits_allocated += credits;
        if (batch.credits_allocated - batch.total_credits).abs() < f64::EPSILON {
            batch.is_active = false;
        }

        let issuance_id = format!("carbon_issue_{}", Uuid::new_v4());
        let issuance = CarbonCreditIssuance {
            issuance_id: issuance_id.clone(),
            batch_id: batch_id.to_string(),
            purchaser_id: purchaser_id.clone(),
            credits,
            token_reference: format!("carbon_token_{}", Uuid::new_v4()),
            issued_at: self.now(),
        };
        self.carbon_issuances
            .insert(issuance_id.clone(), issuance.clone());
        self.record_transaction(
            ImpactAssetType::CarbonCredit,
            batch_id.to_string(),
            purchaser_id,
            credits,
        );
        Ok(issuance)
    }

    pub fn create_charity_receipt(
        &mut self,
        donor_id: String,
        campaign_name: String,
        amount: f64,
        currency: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.ensure_currency(&currency)?;
        if amount <= 0.0 {
            return Err("Receipt amount must be positive".into());
        }

        let receipt_id = format!("receipt_{}", Uuid::new_v4());
        let receipt = CharityReceipt {
            receipt_id: receipt_id.clone(),
            donor_id: donor_id.clone(),
            campaign_name,
            amount,
            currency,
            issued_at: self.now(),
            metadata,
            status: CharityReceiptStatus::Active,
        };
        self.charity_receipts.insert(receipt_id.clone(), receipt);
        self.record_transaction(
            ImpactAssetType::CharityReceipt,
            receipt_id.clone(),
            donor_id,
            amount,
        );
        Ok(receipt_id)
    }

    pub fn redeem_charity_receipt(
        &mut self,
        receipt_id: &str,
    ) -> Result<&CharityReceipt, Box<dyn std::error::Error>> {
        let receipt = self
            .charity_receipts
            .get_mut(receipt_id)
            .ok_or("Receipt not found")?;

        if receipt.status == CharityReceiptStatus::Redeemed {
            return Err("Receipt already redeemed".into());
        }

        receipt.status = CharityReceiptStatus::Redeemed;
        Ok(receipt)
    }

    pub fn create_impact_bond(
        &mut self,
        project_name: String,
        issuer: String,
        face_value: f64,
        coupon_rate: f64,
        currency: String,
        maturity_in_days: i64,
        total_supply: usize,
        nft_metadata: Option<serde_json::Value>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.ensure_currency(&currency)?;
        if face_value <= 0.0 || coupon_rate < 0.0 || total_supply == 0 {
            return Err("Invalid bond parameters".into());
        }

        let bond_id = format!("bond_{}", Uuid::new_v4());
        let now = self.now();
        let bond = ImpactBond {
            bond_id: bond_id.clone(),
            project_name,
            issuer: issuer.clone(),
            face_value,
            coupon_rate,
            total_supply,
            minted_supply: 0,
            currency,
            maturity_date: now + Duration::days(maturity_in_days),
            nft_metadata,
            created_at: now,
            is_active: true,
        };
        self.impact_bonds.insert(bond_id.clone(), bond);
        Ok(bond_id)
    }

    pub fn mint_impact_bond_nfts(
        &mut self,
        bond_id: &str,
        owner_id: String,
        units: usize,
    ) -> Result<Vec<ImpactBondNFT>, Box<dyn std::error::Error>> {
        if units == 0 {
            return Err("Must mint at least one unit".into());
        }

        // First, check if the bond exists and is active, and get the necessary data
        let (face_value, coupon_rate, maturity_date, nft_metadata, transaction_amount) = {
            let bond = self
                .impact_bonds
                .get(bond_id)
                .ok_or("Impact bond not found")?;

            if !bond.is_active {
                return Err("Bond is no longer active".into());
            }

            let remaining = bond.total_supply - bond.minted_supply;
            if units > remaining {
                return Err("Not enough NFTs remaining for this bond".into());
            }

            let transaction_amount = bond.face_value * (units as f64);
            (
                bond.face_value,
                bond.coupon_rate,
                bond.maturity_date,
                bond.nft_metadata.clone(),
                transaction_amount,
            )
        };

        let now = self.now();
        let mut minted = Vec::new();
        for _ in 0..units {
            let nft_id = format!("bond_nft_{}", Uuid::new_v4());
            let nft = ImpactBondNFT {
                nft_id: nft_id.clone(),
                bond_id: bond_id.to_string(),
                owner_id: owner_id.clone(),
                face_value,
                coupon_rate,
                maturity_date,
                issued_at: now,
                metadata: nft_metadata.clone(),
            };
            self.bond_nfts.insert(nft_id.clone(), nft.clone());
            minted.push(nft);
        }

        // Now get a mutable reference to update the bond
        let bond = self.impact_bonds.get_mut(bond_id).unwrap();
        bond.minted_supply += units;
        if bond.minted_supply >= bond.total_supply {
            bond.is_active = false;
        }

        // Drop the mutable reference before calling record_transaction
        drop(bond);

        self.record_transaction(
            ImpactAssetType::ImpactBondNFT,
            bond_id.to_string(),
            owner_id,
            transaction_amount,
        );

        Ok(minted)
    }

    pub fn get_bond_nfts_for_owner(&self, owner_id: &str) -> Vec<&ImpactBondNFT> {
        self.bond_nfts
            .values()
            .filter(|nft| nft.owner_id == owner_id)
            .collect()
    }

    pub fn get_transaction(&self, transaction_id: &str) -> Option<&ImpactAssetTransaction> {
        self.transactions.get(transaction_id)
    }
}
