//! IoT service module for P-Project
//! 
//! This module provides IoT integration features including:
//! - Smart donation boxes (hardware wallets)
//! - NFC wristbands for refugee camps
//! - QR-code-based food distribution system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{NaiveDateTime, Utc};

/// IoT service configuration
#[derive(Debug, Clone)]
pub struct IoTServiceConfig {
    pub api_endpoint: String,
    pub auth_token: String,
}

/// Smart donation box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartDonationBox {
    pub box_id: String,
    pub location: String,
    pub wallet_address: String,
    pub balance: f64,
    pub last_donation: Option<NaiveDateTime>,
    pub total_donations: u64,
    pub is_active: bool,
}

/// Donation box transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationBoxTransaction {
    pub transaction_id: String,
    pub box_id: String,
    pub amount: f64,
    pub donor_address: Option<String>,
    pub timestamp: NaiveDateTime,
}

/// NFC wristband for refugee camps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFCWristband {
    pub wristband_id: String,
    pub refugee_id: String,
    pub camp_id: String,
    pub balance: f64,
    pub last_transaction: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub is_active: bool,
}

/// Wristband transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WristbandTransaction {
    pub transaction_id: String,
    pub wristband_id: String,
    pub amount: f64,
    pub transaction_type: String, // "food", "medical", "supplies"
    pub vendor_id: String,
    pub timestamp: NaiveDateTime,
}

/// QR code for food distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodDistributionQR {
    pub qr_id: String,
    pub distribution_point: String,
    pub food_type: String,
    pub quantity: u32,
    pub expiration_date: NaiveDateTime,
    pub is_claimed: bool,
    pub claimed_by: Option<String>,
    pub claimed_at: Option<NaiveDateTime>,
}

/// QR code claim request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRClaimRequest {
    pub qr_id: String,
    pub recipient_id: String,
    pub recipient_nfc_id: Option<String>,
}

/// QR code claim response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRClaimResponse {
    pub success: bool,
    pub message: String,
    pub claimed_quantity: u32,
}

/// IoT Service for P-Project
pub struct IoTService {
    config: IoTServiceConfig,
    donation_boxes: HashMap<String, SmartDonationBox>,
    wristbands: HashMap<String, NFCWristband>,
    food_qr_codes: HashMap<String, FoodDistributionQR>,
}

impl IoTService {
    /// Create a new IoT service instance
    pub fn new(config: IoTServiceConfig) -> Self {
        Self {
            config,
            donation_boxes: HashMap::new(),
            wristbands: HashMap::new(),
            food_qr_codes: HashMap::new(),
        }
    }

    /// Register a new smart donation box
    pub fn register_donation_box(&mut self, box_id: String, location: String, wallet_address: String) -> Result<SmartDonationBox, Box<dyn std::error::Error>> {
        let box_data = SmartDonationBox {
            box_id: box_id.clone(),
            location,
            wallet_address,
            balance: 0.0,
            last_donation: None,
            total_donations: 0,
            is_active: true,
        };
        
        self.donation_boxes.insert(box_id, box_data.clone());
        Ok(box_data)
    }

    /// Record a donation to a smart donation box
    pub fn record_donation(&mut self, box_id: &str, amount: f64, donor_address: Option<String>) -> Result<DonationBoxTransaction, Box<dyn std::error::Error>> {
        let box_data = self.donation_boxes.get_mut(box_id)
            .ok_or("Donation box not found")?;
        
        if !box_data.is_active {
            return Err("Donation box is not active".into());
        }
        
        box_data.balance += amount;
        box_data.total_donations += 1;
        box_data.last_donation = Some(Utc::now().naive_utc());
        
        let transaction = DonationBoxTransaction {
            transaction_id: format!("tx_{}", uuid::Uuid::new_v4()),
            box_id: box_id.to_string(),
            amount,
            donor_address,
            timestamp: Utc::now().naive_utc(),
        };
        
        Ok(transaction)
    }

    /// Get donation box status
    pub fn get_donation_box_status(&self, box_id: &str) -> Option<&SmartDonationBox> {
        self.donation_boxes.get(box_id)
    }

    /// Register an NFC wristband for a refugee
    pub fn register_wristband(&mut self, wristband_id: String, refugee_id: String, camp_id: String) -> Result<NFCWristband, Box<dyn std::error::Error>> {
        let wristband = NFCWristband {
            wristband_id: wristband_id.clone(),
            refugee_id,
            camp_id,
            balance: 0.0,
            last_transaction: None,
            created_at: Utc::now().naive_utc(),
            is_active: true,
        };
        
        self.wristbands.insert(wristband_id, wristband.clone());
        Ok(wristband)
    }

    /// Add funds to an NFC wristband
    pub fn add_funds_to_wristband(&mut self, wristband_id: &str, amount: f64) -> Result<(), Box<dyn std::error::Error>> {
        let wristband = self.wristbands.get_mut(wristband_id)
            .ok_or("Wristband not found")?;
        
        if !wristband.is_active {
            return Err("Wristband is not active".into());
        }
        
        wristband.balance += amount;
        Ok(())
    }

    /// Process a transaction with an NFC wristband
    pub fn process_wristband_transaction(&mut self, wristband_id: &str, amount: f64, transaction_type: String, vendor_id: String) -> Result<WristbandTransaction, Box<dyn std::error::Error>> {
        let wristband = self.wristbands.get_mut(wristband_id)
            .ok_or("Wristband not found")?;
        
        if !wristband.is_active {
            return Err("Wristband is not active".into());
        }
        
        if wristband.balance < amount {
            return Err("Insufficient balance on wristband".into());
        }
        
        wristband.balance -= amount;
        wristband.last_transaction = Some(Utc::now().naive_utc());
        
        let transaction = WristbandTransaction {
            transaction_id: format!("tx_{}", uuid::Uuid::new_v4()),
            wristband_id: wristband_id.to_string(),
            amount,
            transaction_type,
            vendor_id,
            timestamp: Utc::now().naive_utc(),
        };
        
        Ok(transaction)
    }

    /// Get wristband status
    pub fn get_wristband_status(&self, wristband_id: &str) -> Option<&NFCWristband> {
        self.wristbands.get(wristband_id)
    }

    /// Create a new food distribution QR code
    pub fn create_food_qr(&mut self, distribution_point: String, food_type: String, quantity: u32, expiration_date: NaiveDateTime) -> Result<FoodDistributionQR, Box<dyn std::error::Error>> {
        let qr = FoodDistributionQR {
            qr_id: format!("qr_{}", uuid::Uuid::new_v4()),
            distribution_point,
            food_type,
            quantity,
            expiration_date,
            is_claimed: false,
            claimed_by: None,
            claimed_at: None,
        };
        
        self.food_qr_codes.insert(qr.qr_id.clone(), qr.clone());
        Ok(qr)
    }

    /// Claim a food distribution QR code
    pub fn claim_food_qr(&mut self, request: QRClaimRequest) -> Result<QRClaimResponse, Box<dyn std::error::Error>> {
        let qr = self.food_qr_codes.get_mut(&request.qr_id)
            .ok_or("QR code not found")?;
        
        if qr.is_claimed {
            return Ok(QRClaimResponse {
                success: false,
                message: "QR code already claimed".to_string(),
                claimed_quantity: 0,
            });
        }
        
        let now = Utc::now().naive_utc();
        if now > qr.expiration_date {
            return Ok(QRClaimResponse {
                success: false,
                message: "QR code has expired".to_string(),
                claimed_quantity: 0,
            });
        }
        
        qr.is_claimed = true;
        qr.claimed_by = Some(request.recipient_id);
        qr.claimed_at = Some(now);
        
        Ok(QRClaimResponse {
            success: true,
            message: "Food distribution claimed successfully".to_string(),
            claimed_quantity: qr.quantity,
        })
    }

    /// Get QR code status
    pub fn get_qr_status(&self, qr_id: &str) -> Option<&FoodDistributionQR> {
        self.food_qr_codes.get(qr_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_iot_service_creation() {
        let config = IoTServiceConfig {
            api_endpoint: "https://api.example.com".to_string(),
            auth_token: "test_token".to_string(),
        };
        
        let service = IoTService::new(config);
        assert!(service.donation_boxes.is_empty());
        assert!(service.wristbands.is_empty());
        assert!(service.food_qr_codes.is_empty());
    }

    #[test]
    fn test_donation_box_registration() {
        let config = IoTServiceConfig {
            api_endpoint: "https://api.example.com".to_string(),
            auth_token: "test_token".to_string(),
        };
        
        let mut service = IoTService::new(config);
        
        let result = service.register_donation_box(
            "box123".to_string(),
            "Central Park".to_string(),
            "0x1234567890abcdef".to_string()
        );
        
        assert!(result.is_ok());
        let box_data = result.unwrap();
        assert_eq!(box_data.box_id, "box123");
        assert_eq!(box_data.location, "Central Park");
        assert_eq!(box_data.wallet_address, "0x1234567890abcdef");
        assert_eq!(box_data.balance, 0.0);
    }

    #[test]
    fn test_donation_recording() {
        let config = IoTServiceConfig {
            api_endpoint: "https://api.example.com".to_string(),
            auth_token: "test_token".to_string(),
        };
        
        let mut service = IoTService::new(config);
        
        // Register a box first
        service.register_donation_box(
            "box123".to_string(),
            "Central Park".to_string(),
            "0x1234567890abcdef".to_string()
        ).unwrap();
        
        // Record a donation
        let result = service.record_donation("box123", 50.0, Some("0xabcdef1234567890".to_string()));
        
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert_eq!(transaction.box_id, "box123");
        assert_eq!(transaction.amount, 50.0);
        assert_eq!(transaction.donor_address, Some("0xabcdef1234567890".to_string()));
        
        // Check box status
        let box_status = service.get_donation_box_status("box123").unwrap();
        assert_eq!(box_status.balance, 50.0);
        assert_eq!(box_status.total_donations, 1);
        assert!(box_status.last_donation.is_some());
    }

    #[test]
    fn test_wristband_registration() {
        let config = IoTServiceConfig {
            api_endpoint: "https://api.example.com".to_string(),
            auth_token: "test_token".to_string(),
        };
        
        let mut service = IoTService::new(config);
        
        let result = service.register_wristband(
            "wristband123".to_string(),
            "refugee456".to_string(),
            "camp789".to_string()
        );
        
        assert!(result.is_ok());
        let wristband = result.unwrap();
        assert_eq!(wristband.wristband_id, "wristband123");
        assert_eq!(wristband.refugee_id, "refugee456");
        assert_eq!(wristband.camp_id, "camp789");
        assert_eq!(wristband.balance, 0.0);
    }

    #[test]
    fn test_wristband_transactions() {
        let config = IoTServiceConfig {
            api_endpoint: "https://api.example.com".to_string(),
            auth_token: "test_token".to_string(),
        };
        
        let mut service = IoTService::new(config);
        
        // Register a wristband
        service.register_wristband(
            "wristband123".to_string(),
            "refugee456".to_string(),
            "camp789".to_string()
        ).unwrap();
        
        // Add funds
        let result = service.add_funds_to_wristband("wristband123", 100.0);
        assert!(result.is_ok());
        
        // Check balance
        let wristband = service.get_wristband_status("wristband123").unwrap();
        assert_eq!(wristband.balance, 100.0);
        
        // Process transaction
        let result = service.process_wristband_transaction(
            "wristband123",
            25.0,
            "food".to_string(),
            "vendor123".to_string()
        );
        
        assert!(result.is_ok());
        let transaction = result.unwrap();
        assert_eq!(transaction.wristband_id, "wristband123");
        assert_eq!(transaction.amount, 25.0);
        assert_eq!(transaction.transaction_type, "food");
        assert_eq!(transaction.vendor_id, "vendor123");
        
        // Check updated balance
        let wristband = service.get_wristband_status("wristband123").unwrap();
        assert_eq!(wristband.balance, 75.0);
    }

    #[test]
    fn test_food_qr_creation_and_claim() {
        let config = IoTServiceConfig {
            api_endpoint: "https://api.example.com".to_string(),
            auth_token: "test_token".to_string(),
        };
        
        let mut service = IoTService::new(config);
        
        let expiration = Utc::now().naive_utc() + Duration::days(7);
        
        // Create QR code
        let result = service.create_food_qr(
            "Food Bank Downtown".to_string(),
            "Rice".to_string(),
            10,
            expiration
        );
        
        assert!(result.is_ok());
        let qr = result.unwrap();
        assert_eq!(qr.distribution_point, "Food Bank Downtown");
        assert_eq!(qr.food_type, "Rice");
        assert_eq!(qr.quantity, 10);
        assert_eq!(qr.expiration_date, expiration);
        assert!(!qr.is_claimed);
        assert!(qr.claimed_by.is_none());
        assert!(qr.claimed_at.is_none());
        
        // Claim QR code
        let request = QRClaimRequest {
            qr_id: qr.qr_id.clone(),
            recipient_id: "recipient123".to_string(),
            recipient_nfc_id: Some("nfc456".to_string()),
        };
        
        let result = service.claim_food_qr(request);
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.claimed_quantity, 10);
        
        // Check QR status
        let qr_status = service.get_qr_status(&qr.qr_id).unwrap();
        assert!(qr_status.is_claimed);
        assert_eq!(qr_status.claimed_by, Some("recipient123".to_string()));
        assert!(qr_status.claimed_at.is_some());
    }
}