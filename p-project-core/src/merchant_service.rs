//! Merchant service module for P-Project
//!
//! This module provides merchant payment features including:
//! - Small business acceptance (coffee shops, restaurants, bookstores, clinics, repair shops)
//! - QR code payment processing
//! - Merchant registration and management
//! - Payment transaction handling

use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
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

/// Global QR payment data for cross-border transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalQRPaymentData {
    pub qr_id: String,
    pub merchant_id: String,
    pub amount: f64,
    pub currency: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub country_code: String,  // ISO 3166-1 alpha-2 country code
    pub language_code: String, // ISO 639-1 language code
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub metadata: Option<serde_json::Value>, // Additional metadata for cross-border transactions
}

/// Digital goods product information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalGoodsProduct {
    pub product_id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub currency: String,
    pub category: String, // e.g., "software", "ebook", "music", "video", "game"
    pub country_restrictions: Option<Vec<String>>, // ISO 3166-1 alpha-2 country codes where product is restricted
    pub language_availability: Vec<String>,        // ISO 639-1 language codes
    pub digital_delivery_method: String,           // "download", "stream", "license_key", etc.
    pub download_url: Option<String>,
    pub license_key_template: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Digital goods transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalGoodsTransaction {
    pub transaction_id: String,
    pub product_id: String,
    pub customer_id: String,
    pub customer_country: String,  // ISO 3166-1 alpha-2 country code
    pub customer_language: String, // ISO 639-1 language code
    pub amount: f64,
    pub currency: String,
    pub status: String,          // "pending", "completed", "failed", "refunded"
    pub delivery_status: String, // "pending", "delivered", "failed"
    pub delivery_method: String,
    pub delivery_data: Option<serde_json::Value>, // License key, download link, etc.
    pub created_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
}

/// Customer loyalty points data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyPoints {
    pub customer_id: String,
    pub merchant_id: String,
    pub points: f64,       // Points can be fractional for precision
    pub total_earned: f64, // Total points earned over time
    pub total_spent: f64,  // Total points spent/redeemed
    pub last_updated: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

/// Cashback transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashbackTransaction {
    pub transaction_id: String,
    pub customer_id: String,
    pub merchant_id: String,
    pub original_amount: f64,       // Original purchase amount
    pub cashback_amount: f64,       // Cashback in P-Coin
    pub cashback_percentage: f64,   // Cashback percentage (e.g., 5.0 for 5%)
    pub loyalty_points_earned: f64, // Loyalty points earned from this purchase
    pub status: String,             // "pending", "completed", "failed"
    pub timestamp: NaiveDateTime,
    pub tx_hash: Option<String>, // Blockchain transaction hash for cashback
}

/// Cashback configuration for a merchant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashbackConfig {
    pub merchant_id: String,
    pub cashback_percentage: f64, // Percentage of purchase to give as cashback (e.g., 5.0 for 5%)
    pub min_purchase_amount: f64, // Minimum purchase amount to qualify for cashback
    pub max_cashback_amount: f64, // Maximum cashback amount per transaction
    pub loyalty_points_per_coin: f64, // How many loyalty points per P-Coin spent
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
    pub global_qr_payments: HashMap<String, GlobalQRPaymentData>,
    pub config: MerchantServiceConfig,
    pub digital_goods_products: HashMap<String, DigitalGoodsProduct>,
    pub digital_goods_transactions: HashMap<String, DigitalGoodsTransaction>,
    // New fields for cashback and loyalty systems
    pub cashback_configs: HashMap<String, CashbackConfig>,
    pub loyalty_points: HashMap<String, LoyaltyPoints>, // Key: customer_id + merchant_id
    pub cashback_transactions: HashMap<String, CashbackTransaction>,
}

impl MerchantService {
    /// Create a new merchant service
    pub fn new(config: MerchantServiceConfig) -> Self {
        Self {
            merchants: HashMap::new(),
            qr_payments: HashMap::new(),
            global_qr_payments: HashMap::new(),
            config,
            digital_goods_products: HashMap::new(),
            digital_goods_transactions: HashMap::new(),
            cashback_configs: HashMap::new(),
            loyalty_points: HashMap::new(),
            cashback_transactions: HashMap::new(),
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
        let merchant = self
            .merchants
            .get_mut(merchant_id)
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
        let merchant = self
            .merchants
            .get(&merchant_id)
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

        let expires_at = expires_in_seconds
            .map(|seconds| Utc::now().naive_utc() + chrono::Duration::seconds(seconds));

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
        let merchant = self
            .merchants
            .get(&request.merchant_id)
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

    /// Create a global QR code for payment (cross-border compatible)
    pub fn create_global_payment_qr(
        &mut self,
        merchant_id: String,
        amount: f64,
        currency: String,
        description: Option<String>,
        country_code: String,
        language_code: String,
        expires_in_seconds: Option<i64>,
        metadata: Option<serde_json::Value>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Verify merchant exists and is verified
        let merchant = self
            .merchants
            .get(&merchant_id)
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

        // Validate country code (basic validation)
        if country_code.len() != 2 || country_code.chars().any(|c| !c.is_ascii_alphabetic()) {
            return Err("Invalid country code".into());
        }

        // Validate language code (basic validation)
        if language_code.len() != 2 || language_code.chars().any(|c| !c.is_ascii_alphabetic()) {
            return Err("Invalid language code".into());
        }

        let qr_id = format!("gqr_{}", Uuid::new_v4());

        let expires_at = expires_in_seconds
            .map(|seconds| Utc::now().naive_utc() + chrono::Duration::seconds(seconds));

        let qr_data = GlobalQRPaymentData {
            qr_id: qr_id.clone(),
            merchant_id,
            amount,
            currency,
            description,
            is_active: true,
            country_code,
            language_code,
            expires_at,
            created_at: Utc::now().naive_utc(),
            metadata,
        };

        self.global_qr_payments.insert(qr_id.clone(), qr_data);
        Ok(qr_id)
    }

    /// Get global QR payment data
    pub fn get_global_qr_payment(&self, qr_id: &str) -> Option<&GlobalQRPaymentData> {
        self.global_qr_payments.get(qr_id)
    }

    /// Process a global QR payment
    pub async fn process_global_qr_payment(
        &mut self,
        request: MerchantPaymentRequest,
        qr_id: String,
    ) -> Result<MerchantPaymentResponse, Box<dyn std::error::Error>> {
        // Verify merchant exists and is verified
        let merchant = self
            .merchants
            .get(&request.merchant_id)
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

        // Check global QR code
        let mut qr_valid = true;
        let mut qr_expired = false;

        if let Some(qr_data) = self.global_qr_payments.get(&qr_id) {
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
            qr_code_used: Some(qr_id),
        };

        Ok(response)
    }

    /// Get merchant statistics
    pub fn get_merchant_stats(
        &self,
        merchant_id: &str,
    ) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let merchant = self
            .merchants
            .get(merchant_id)
            .ok_or("Merchant not found")?;

        let mut stats = HashMap::new();
        stats.insert("total_transactions".to_string(), 0.0);
        stats.insert("total_volume".to_string(), 0.0);
        stats.insert("average_transaction".to_string(), 0.0);

        // In a real implementation, this would query transaction data
        // For now, we return default values

        Ok(stats)
    }

    /// Add a digital goods product to a merchant
    pub fn add_digital_goods_product(
        &mut self,
        merchant_id: &str,
        product: DigitalGoodsProduct,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Verify merchant exists and is verified
        let merchant = self
            .merchants
            .get(merchant_id)
            .ok_or("Merchant not found")?;

        if !merchant.is_verified {
            return Err("Merchant is not verified".into());
        }

        // Validate price
        if product.price <= 0.0 {
            return Err("Price must be positive".into());
        }

        // Validate currency (should be P-Coin)
        if product.currency != "P" {
            return Err("Only P-Coin pricing is supported".into());
        }

        // Store the product (in a real implementation, this would be in a database)
        self.digital_goods_products
            .insert(product.product_id.clone(), product);
        Ok(())
    }

    /// Get a digital goods product
    pub fn get_digital_goods_product(&self, product_id: &str) -> Option<&DigitalGoodsProduct> {
        self.digital_goods_products.get(product_id)
    }

    /// Purchase a digital good
    pub async fn purchase_digital_good(
        &mut self,
        product_id: String,
        customer_id: String,
        customer_country: String,
        customer_language: String,
    ) -> Result<DigitalGoodsTransaction, Box<dyn std::error::Error>> {
        // Get the product
        let product = self
            .digital_goods_products
            .get(&product_id)
            .ok_or("Product not found")?;

        // Check country restrictions
        if let Some(restrictions) = &product.country_restrictions {
            if restrictions.contains(&customer_country) {
                return Err("Product is not available in your country".into());
            }
        }

        // Check language availability
        if !product.language_availability.contains(&customer_language) {
            // This is not a hard restriction, but we should note it
            println!("Product not available in customer's preferred language");
        }

        // Create transaction
        let transaction_id = format!("dgtx_{}", Uuid::new_v4());

        // Prepare delivery data based on delivery method
        let delivery_data = match product.digital_delivery_method.as_str() {
            "download" => serde_json::to_value(serde_json::json!({
                "download_url": product.download_url,
                "filename": product.name
            }))
            .ok(),
            "license_key" => {
                // In a real implementation, we would generate a unique license key
                serde_json::to_value(serde_json::json!({
                    "license_key": product.license_key_template.as_ref().map(|t| format!("{}-{}", t, Uuid::new_v4().simple()))
                })).ok()
            }
            _ => None,
        };

        let transaction = DigitalGoodsTransaction {
            transaction_id: transaction_id.clone(),
            product_id: product_id.clone(),
            customer_id,
            customer_country,
            customer_language,
            amount: product.price,
            currency: product.currency.clone(),
            status: "completed".to_string(),
            delivery_status: "delivered".to_string(),
            delivery_method: product.digital_delivery_method.clone(),
            delivery_data,
            created_at: Utc::now().naive_utc(),
            completed_at: Some(Utc::now().naive_utc()),
        };

        // Store the transaction
        self.digital_goods_transactions
            .insert(transaction_id, transaction.clone());

        Ok(transaction)
    }

    /// Get digital goods transaction
    pub fn get_digital_goods_transaction(
        &self,
        transaction_id: &str,
    ) -> Option<&DigitalGoodsTransaction> {
        self.digital_goods_transactions.get(transaction_id)
    }

    /// Get all digital goods transactions for a customer
    pub fn get_customer_digital_goods_transactions(
        &self,
        customer_id: &str,
    ) -> Vec<&DigitalGoodsTransaction> {
        self.digital_goods_transactions
            .values()
            .filter(|tx| tx.customer_id == customer_id)
            .collect()
    }

    /// Configure cashback for a merchant
    pub fn configure_cashback(
        &mut self,
        merchant_id: String,
        cashback_percentage: f64,
        min_purchase_amount: f64,
        max_cashback_amount: f64,
        loyalty_points_per_coin: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Verify merchant exists and is verified
        let merchant = self
            .merchants
            .get(&merchant_id)
            .ok_or("Merchant not found")?;

        if !merchant.is_verified {
            return Err("Merchant is not verified".into());
        }

        // Validate parameters
        if cashback_percentage < 0.0 || cashback_percentage > 50.0 {
            return Err("Cashback percentage must be between 0 and 50".into());
        }

        if min_purchase_amount < 0.0 {
            return Err("Minimum purchase amount must be positive".into());
        }

        if max_cashback_amount <= 0.0 {
            return Err("Maximum cashback amount must be positive".into());
        }

        if loyalty_points_per_coin < 0.0 {
            return Err("Loyalty points per coin must be non-negative".into());
        }

        let now = Utc::now().naive_utc();
        let config = CashbackConfig {
            merchant_id: merchant_id.clone(),
            cashback_percentage,
            min_purchase_amount,
            max_cashback_amount,
            loyalty_points_per_coin,
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        self.cashback_configs.insert(merchant_id, config);
        Ok(())
    }

    /// Get cashback configuration for a merchant
    pub fn get_cashback_config(&self, merchant_id: &str) -> Option<&CashbackConfig> {
        self.cashback_configs.get(merchant_id)
    }

    /// Calculate cashback for a purchase
    pub fn calculate_cashback(
        &self,
        merchant_id: &str,
        purchase_amount: f64,
    ) -> Result<(f64, f64), Box<dyn std::error::Error>> {
        // Get merchant cashback configuration
        let config = self
            .cashback_configs
            .get(merchant_id)
            .ok_or("No cashback configuration for this merchant")?;

        if !config.is_active {
            return Err("Cashback is not active for this merchant".into());
        }

        // Check if purchase qualifies for cashback
        if purchase_amount < config.min_purchase_amount {
            return Ok((0.0, 0.0)); // No cashback, no loyalty points
        }

        // Calculate cashback amount
        let cashback_amount =
            (purchase_amount * config.cashback_percentage / 100.0).min(config.max_cashback_amount);

        // Calculate loyalty points
        let loyalty_points = purchase_amount * config.loyalty_points_per_coin;

        Ok((cashback_amount, loyalty_points))
    }

    /// Process a purchase with cashback and loyalty points
    pub async fn process_purchase_with_cashback(
        &mut self,
        merchant_id: String,
        customer_id: String,
        purchase_amount: f64,
        customer_wallet: String,
    ) -> Result<CashbackTransaction, Box<dyn std::error::Error>> {
        // Verify merchant exists and is verified
        let merchant = self
            .merchants
            .get(&merchant_id)
            .ok_or("Merchant not found")?;

        if !merchant.is_verified {
            return Err("Merchant is not verified".into());
        }

        // Validate purchase amount
        if purchase_amount <= 0.0 {
            return Err("Purchase amount must be positive".into());
        }

        if purchase_amount > self.config.max_transaction_amount {
            return Err("Purchase amount exceeds maximum transaction limit".into());
        }

        // Validate currency (should be P-Coin)
        if merchant_id != merchant.id {
            return Err("Invalid merchant".into());
        }

        // Calculate cashback and loyalty points
        let (cashback_amount, loyalty_points) =
            self.calculate_cashback(&merchant_id, purchase_amount)?;

        // Create cashback transaction
        let transaction_id = format!("cashback_{}", Uuid::new_v4());
        let now = Utc::now().naive_utc();

        let config = self
            .cashback_configs
            .get(&merchant_id)
            .ok_or("No cashback configuration for this merchant")?;

        let cashback_tx = CashbackTransaction {
            transaction_id: transaction_id.clone(),
            customer_id: customer_id.clone(),
            merchant_id: merchant_id.clone(),
            original_amount: purchase_amount,
            cashback_amount,
            cashback_percentage: config.cashback_percentage,
            loyalty_points_earned: loyalty_points,
            status: if cashback_amount > 0.0 {
                "pending"
            } else {
                "completed"
            }
            .to_string(),
            timestamp: now,
            tx_hash: None,
        };

        // Update loyalty points
        let loyalty_key = format!("{}_{}", customer_id, merchant_id);
        let loyalty_points_entry = self
            .loyalty_points
            .entry(loyalty_key.clone())
            .or_insert_with(|| LoyaltyPoints {
                customer_id: customer_id.clone(),
                merchant_id: merchant_id.clone(),
                points: 0.0,
                total_earned: 0.0,
                total_spent: 0.0,
                last_updated: now,
                created_at: now,
            });

        loyalty_points_entry.points += loyalty_points;
        loyalty_points_entry.total_earned += loyalty_points;
        loyalty_points_entry.last_updated = now;

        // If there's cashback to distribute, simulate the blockchain transaction
        if cashback_amount > 0.0 {
            // In a real implementation, this would interact with the blockchain
            // to transfer cashback tokens from merchant to customer
            let tx_hash = format!("0x{}", Uuid::new_v4().simple());

            // Update transaction with tx_hash and mark as completed
            let mut completed_tx = cashback_tx.clone();
            completed_tx.tx_hash = Some(tx_hash);
            completed_tx.status = "completed".to_string();

            self.cashback_transactions
                .insert(transaction_id.clone(), completed_tx.clone());
            Ok(completed_tx)
        } else {
            self.cashback_transactions
                .insert(transaction_id.clone(), cashback_tx.clone());
            Ok(cashback_tx)
        }
    }

    /// Get customer loyalty points for a specific merchant
    pub fn get_customer_loyalty_points(
        &self,
        customer_id: &str,
        merchant_id: &str,
    ) -> Option<&LoyaltyPoints> {
        let loyalty_key = format!("{}_{}", customer_id, merchant_id);
        self.loyalty_points.get(&loyalty_key)
    }

    /// Get all loyalty points for a customer across all merchants
    pub fn get_all_customer_loyalty_points(&self, customer_id: &str) -> Vec<&LoyaltyPoints> {
        self.loyalty_points
            .values()
            .filter(|lp| lp.customer_id == customer_id)
            .collect()
    }

    /// Get cashback transaction
    pub fn get_cashback_transaction(&self, transaction_id: &str) -> Option<&CashbackTransaction> {
        self.cashback_transactions.get(transaction_id)
    }

    /// Get all cashback transactions for a customer
    pub fn get_customer_cashback_transactions(
        &self,
        customer_id: &str,
    ) -> Vec<&CashbackTransaction> {
        self.cashback_transactions
            .values()
            .filter(|tx| tx.customer_id == customer_id)
            .collect()
    }

    /// Redeem loyalty points
    pub fn redeem_loyalty_points(
        &mut self,
        customer_id: &str,
        merchant_id: &str,
        points_to_redeem: f64,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        // Verify merchant exists and is verified
        let merchant = self
            .merchants
            .get(merchant_id)
            .ok_or("Merchant not found")?;

        if !merchant.is_verified {
            return Err("Merchant is not verified".into());
        }

        // Validate points to redeem
        if points_to_redeem <= 0.0 {
            return Err("Points to redeem must be positive".into());
        }

        // Get customer loyalty points for this merchant
        let loyalty_key = format!("{}_{}", customer_id, merchant_id);
        let loyalty_points = self
            .loyalty_points
            .get_mut(&loyalty_key)
            .ok_or("No loyalty points found for this customer and merchant")?;

        // Check if customer has enough points
        if loyalty_points.points < points_to_redeem {
            return Err("Insufficient loyalty points".into());
        }

        // Deduct points
        loyalty_points.points -= points_to_redeem;
        loyalty_points.total_spent += points_to_redeem;
        loyalty_points.last_updated = Utc::now().naive_utc();

        // Convert points to P-Coin value (assuming 1 point = 0.01 P-Coin)
        let coin_value = points_to_redeem * 0.01;

        // In a real implementation, this would interact with the blockchain
        // to transfer tokens from merchant to customer

        Ok(coin_value)
    }
}
