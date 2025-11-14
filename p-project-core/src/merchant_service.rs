//! Merchant service module for P-Project
//!
//! This module provides merchant payment features including:
//! - Small business acceptance (coffee shops, restaurants, bookstores, clinics, repair shops)
//! - QR code payment processing
//! - Merchant registration and management
//! - Payment transaction handling

use serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// Merchant category types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MerchantCategory {
    CoffeeShop,
    Restaurant,
    Bookstore,
    Clinic,
    RepairShop,
    Other(String),
}

/// Merchant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Merchant {
    pub id: String,
    pub name: String,
    pub category: MerchantCategory,
    pub wallet_address: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub contact_info: Option<String>,
    pub is_verified: bool,
    pub created_at: NaiveDateTime,
    pub verified_at: Option<NaiveDateTime>,
}

/// Payment request for merchant transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantPaymentRequest {
    pub merchant_id: String,
    pub customer_wallet: String,
    pub amount: f64,
    pub currency: String, // Should be "P" for P-Coin
    pub description: Option<String>,
    pub qr_code: Option<String>, // Optional QR code for the transaction
}

/// Payment response after processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantPaymentResponse {
    pub transaction_id: String,
    pub merchant_id: String,
    pub customer_wallet: String,
    pub amount: f64,
    pub currency: String,
    pub status: String, // "success", "failed", "pending"
    pub timestamp: NaiveDateTime,
    pub tx_hash: Option<String>, // Blockchain transaction hash
    pub qr_code_used: Option<String>,
}

/// QR code payment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRPaymentData {
    pub qr_id: String,
    pub merchant_id: String,
    pub amount: f64,
    pub currency: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

/// Merchant service configuration
#[derive(Debug, Clone)]
pub struct MerchantServiceConfig {
    pub fee_percentage: f64, // Platform fee percentage (e.g., 0.01 for 1%)
    pub max_transaction_amount: f64, // Maximum allowed transaction amount
}

/// Merchant service implementation
pub struct MerchantService {
    pub merchants: HashMap<String, Merchant>,
    pub qr_payments: HashMap<String, QRPaymentData>,
    pub config: MerchantServiceConfig,
}

impl MerchantService {
    /// Create a new merchant service
    pub fn new(config: MerchantServiceConfig) -> Self {
        Self {
            merchants: HashMap::new(),
            qr_payments: HashMap::new(),
            config,
        }
    }

    /// Register a new merchant
    pub fn register_merchant(
        &mut self,
        name: String,
        category: MerchantCategory,
        wallet_address: String,
        description: Option<String>,
        location: Option<String>,
        contact_info: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let merchant_id = format!("merchant_{}", Uuid::new_v4());
        
        let merchant = Merchant {
            id: merchant_id.clone(),
            name,
            category,
            wallet_address,
            description,
            location,
            contact_info,
            is_verified: false,
            created_at: Utc::now().naive_utc(),
            verified_at: None,
        };
        
        self.merchants.insert(merchant_id.clone(), merchant);
        Ok(merchant_id)
    }

    /// Verify a merchant (admin function)
    pub fn verify_merchant(&mut self, merchant_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let merchant = self.merchants.get_mut(merchant_id)
            .ok_or("Merchant not found")?;
        
        merchant.is_verified = true;
        merchant.verified_at = Some(Utc::now().naive_utc());
        Ok(())
    }

    /// Get merchant information
    pub fn get_merchant(&self, merchant_id: &str) -> Option<&Merchant> {
        self.merchants.get(merchant_id)
    }

    /// Get all merchants
    pub fn get_all_merchants(&self) -> Vec<&Merchant> {
        self.merchants.values().collect()
    }

    /// Create a QR code for payment
    pub fn create_payment_qr(
        &mut self,
        merchant_id: String,
        amount: f64,
        currency: String,
        description: Option<String>,
        expires_in_seconds: Option<i64>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Verify merchant exists and is verified
        let merchant = self.merchants.get(&merchant_id)
            .ok_or("Merchant not found")?;
        
        if !merchant.is_verified {
            return Err("Merchant is not verified".into());
        }
        
        // Validate amount
        if amount <= 0.0 {
            return Err("Amount must be positive".into());
        }
        
        if amount > self.config.max_transaction_amount {
            return Err("Amount exceeds maximum transaction limit".into());
        }
        
        let qr_id = format!("qr_{}", Uuid::new_v4());
        
        let expires_at = expires_in_seconds.map(|seconds| {
            Utc::now().naive_utc() + chrono::Duration::seconds(seconds)
        });
        
        let qr_data = QRPaymentData {
            qr_id: qr_id.clone(),
            merchant_id,
            amount,
            currency,
            description,
            is_active: true,
            expires_at,
            created_at: Utc::now().naive_utc(),
        };
        
        self.qr_payments.insert(qr_id.clone(), qr_data);
        Ok(qr_id)
    }

    /// Get QR payment data
    pub fn get_qr_payment(&self, qr_id: &str) -> Option<&QRPaymentData> {
        self.qr_payments.get(qr_id)
    }

    /// Process a merchant payment
    pub async fn process_payment(
        &mut self,
        request: MerchantPaymentRequest,
    ) -> Result<MerchantPaymentResponse, Box<dyn std::error::Error>> {
        // Verify merchant exists and is verified
        let merchant = self.merchants.get(&request.merchant_id)
            .ok_or("Merchant not found")?;
        
        if !merchant.is_verified {
            return Err("Merchant is not verified".into());
        }
        
        // Validate amount
        if request.amount <= 0.0 {
            return Err("Amount must be positive".into());
        }
        
        if request.amount > self.config.max_transaction_amount {
            return Err("Amount exceeds maximum transaction limit".into());
        }
        
        // Validate currency (should be P-Coin)
        if request.currency != "P" {
            return Err("Only P-Coin payments are supported".into());
        }
        
        // Check QR code if provided
        let mut qr_valid = true;
        let mut qr_expired = false;
        
        if let Some(ref qr_code) = request.qr_code {
            if let Some(qr_data) = self.qr_payments.get(qr_code) {
                // Check if QR is active
                if !qr_data.is_active {
                    qr_valid = false;
                }
                
                // Check if QR has expired
                if let Some(expires_at) = qr_data.expires_at {
                    if Utc::now().naive_utc() > expires_at {
                        qr_expired = true;
                        qr_valid = false;
                    }
                }
                
                // Check if amount matches QR data
                if (request.amount - qr_data.amount).abs() > 0.001 {
                    qr_valid = false;
                }
                
                // Check if merchant matches QR data
                if request.merchant_id != qr_data.merchant_id {
                    qr_valid = false;
                }
            } else {
                qr_valid = false;
            }
        }
        
        // Create transaction ID
        let transaction_id = format!("tx_{}", Uuid::new_v4());
        
        // In a real implementation, this would interact with the blockchain
        // to transfer tokens from customer to merchant, taking a small fee
        let status = if qr_valid && !qr_expired {
            "success"
        } else if qr_expired {
            "failed"
        } else {
            "failed"
        };
        
        let tx_hash = if status == "success" {
            Some(format!("0x{}", Uuid::new_v4().simple()))
        } else {
            None
        };
        
        let response = MerchantPaymentResponse {
            transaction_id,
            merchant_id: request.merchant_id,
            customer_wallet: request.customer_wallet,
            amount: request.amount,
            currency: request.currency,
            status: status.to_string(),
            timestamp: Utc::now().naive_utc(),
            tx_hash,
            qr_code_used: request.qr_code,
        };
        
        Ok(response)
    }

    /// Get merchant statistics
    pub fn get_merchant_stats(&self, merchant_id: &str) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let merchant = self.merchants.get(merchant_id)
            .ok_or("Merchant not found")?;
        
        let mut stats = HashMap::new();
        stats.insert("total_transactions".to_string(), 0.0);
        stats.insert("total_volume".to_string(), 0.0);
        stats.insert("average_transaction".to_string(), 0.0);
        
        // In a real implementation, this would query transaction data
        // For now, we return default values
        
        Ok(stats)
    }
}