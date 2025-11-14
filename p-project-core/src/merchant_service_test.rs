#[cfg(test)]
mod tests {
    use super::super::merchant_service::{MerchantService, MerchantServiceConfig, MerchantCategory, MerchantPaymentRequest};
    use tokio;

    #[test]
    fn test_merchant_registration() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);
        
        let merchant_id = service.register_merchant(
            "Test Coffee Shop".to_string(),
            MerchantCategory::CoffeeShop,
            "0x123456789".to_string(),
            Some("A cozy coffee shop".to_string()),
            Some("123 Main St".to_string()),
            Some("contact@testcoffee.com".to_string()),
        ).unwrap();
        
        assert!(merchant_id.starts_with("merchant_"));
        
        let merchant = service.get_merchant(&merchant_id).unwrap();
        assert_eq!(merchant.name, "Test Coffee Shop");
        assert_eq!(merchant.category, MerchantCategory::CoffeeShop);
        assert_eq!(merchant.wallet_address, "0x123456789");
        assert_eq!(merchant.description, Some("A cozy coffee shop".to_string()));
        assert_eq!(merchant.location, Some("123 Main St".to_string()));
        assert_eq!(merchant.contact_info, Some("contact@testcoffee.com".to_string()));
        assert!(!merchant.is_verified);
    }
    
    #[test]
    fn test_merchant_verification() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);
        
        let merchant_id = service.register_merchant(
            "Test Restaurant".to_string(),
            MerchantCategory::Restaurant,
            "0x987654321".to_string(),
            None,
            None,
            None,
        ).unwrap();
        
        // Verify merchant
        let result = service.verify_merchant(&merchant_id);
        assert!(result.is_ok());
        
        let merchant = service.get_merchant(&merchant_id).unwrap();
        assert!(merchant.is_verified);
        assert!(merchant.verified_at.is_some());
    }
    
    #[test]
    fn test_create_payment_qr() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);
        
        let merchant_id = service.register_merchant(
            "Test Bookstore".to_string(),
            MerchantCategory::Bookstore,
            "0x111111111".to_string(),
            None,
            None,
            None,
        ).unwrap();
        
        // Verify merchant first
        service.verify_merchant(&merchant_id).unwrap();
        
        // Create QR code
        let qr_id = service.create_payment_qr(
            merchant_id.clone(),
            25.99,
            "P".to_string(),
            Some("Book purchase".to_string()),
            Some(3600), // Expires in 1 hour
        ).unwrap();
        
        assert!(qr_id.starts_with("qr_"));
        
        let qr_data = service.get_qr_payment(&qr_id).unwrap();
        assert_eq!(qr_data.merchant_id, merchant_id);
        assert_eq!(qr_data.amount, 25.99);
        assert_eq!(qr_data.currency, "P");
        assert_eq!(qr_data.description, Some("Book purchase".to_string()));
        assert!(qr_data.is_active);
        assert!(qr_data.expires_at.is_some());
    }
    
    #[tokio::test]
    async fn test_process_payment_success() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);
        
        let merchant_id = service.register_merchant(
            "Test Clinic".to_string(),
            MerchantCategory::Clinic,
            "0x222222222".to_string(),
            None,
            None,
            None,
        ).unwrap();
        
        // Verify merchant
        service.verify_merchant(&merchant_id).unwrap();
        
        // Create payment request
        let request = MerchantPaymentRequest {
            merchant_id: merchant_id.clone(),
            customer_wallet: "0x333333333".to_string(),
            amount: 75.50,
            currency: "P".to_string(),
            description: Some("Medical consultation".to_string()),
            qr_code: None,
        };
        
        let response = service.process_payment(request).await.unwrap();
        assert_eq!(response.merchant_id, merchant_id);
        assert_eq!(response.customer_wallet, "0x333333333");
        assert_eq!(response.amount, 75.50);
        assert_eq!(response.currency, "P");
        assert_eq!(response.status, "success");
        assert!(response.tx_hash.is_some());
    }
    
    #[tokio::test]
    async fn test_process_payment_with_qr() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);
        
        let merchant_id = service.register_merchant(
            "Test Repair Shop".to_string(),
            MerchantCategory::RepairShop,
            "0x444444444".to_string(),
            None,
            None,
            None,
        ).unwrap();
        
        // Verify merchant
        service.verify_merchant(&merchant_id).unwrap();
        
        // Create QR code
        let qr_id = service.create_payment_qr(
            merchant_id.clone(),
            120.00,
            "P".to_string(),
            Some("Laptop repair".to_string()),
            Some(3600),
        ).unwrap();
        
        // Process payment with QR
        let request = MerchantPaymentRequest {
            merchant_id: merchant_id.clone(),
            customer_wallet: "0x555555555".to_string(),
            amount: 120.00,
            currency: "P".to_string(),
            description: Some("Laptop repair".to_string()),
            qr_code: Some(qr_id),
        };
        
        let response = service.process_payment(request).await.unwrap();
        assert_eq!(response.status, "success");
        assert!(response.tx_hash.is_some());
        assert!(response.qr_code_used.is_some());
    }
    
    #[tokio::test]
    async fn test_process_payment_invalid_merchant() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);
        
        let request = MerchantPaymentRequest {
            merchant_id: "nonexistent_merchant".to_string(),
            customer_wallet: "0x666666666".to_string(),
            amount: 50.00,
            currency: "P".to_string(),
            description: None,
            qr_code: None,
        };
        
        let result = service.process_payment(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Merchant not found");
    }
    
    #[tokio::test]
    async fn test_process_payment_unverified_merchant() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);
        
        let merchant_id = service.register_merchant(
            "Unverified Shop".to_string(),
            MerchantCategory::CoffeeShop,
            "0x777777777".to_string(),
            None,
            None,
            None,
        ).unwrap();
        
        // Don't verify the merchant
        
        let request = MerchantPaymentRequest {
            merchant_id,
            customer_wallet: "0x888888888".to_string(),
            amount: 30.00,
            currency: "P".to_string(),
            description: None,
            qr_code: None,
        };
        
        let result = service.process_payment(request).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Merchant is not verified");
    }
    
    #[test]
    fn test_get_all_merchants() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);
        
        // Register multiple merchants
        service.register_merchant(
            "Coffee Shop".to_string(),
            MerchantCategory::CoffeeShop,
            "0x111111111".to_string(),
            None,
            None,
            None,
        ).unwrap();
        
        service.register_merchant(
            "Restaurant".to_string(),
            MerchantCategory::Restaurant,
            "0x222222222".to_string(),
            None,
            None,
            None,
        ).unwrap();
        
        let merchants = service.get_all_merchants();
        assert_eq!(merchants.len(), 2);
    }
    
    #[test]
    fn test_merchant_categories() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);
        
        // Test all required merchant categories
        let coffee_shop_id = service.register_merchant(
            "Coffee Shop".to_string(),
            MerchantCategory::CoffeeShop,
            "0x111111111".to_string(),
            None,
            None,
            None,
        ).unwrap();
        
        let restaurant_id = service.register_merchant(
            "Restaurant".to_string(),
            MerchantCategory::Restaurant,
            "0x222222222".to_string(),
            None,
            None,
            None,
        ).unwrap();
        
        let bookstore_id = service.register_merchant(
            "Bookstore".to_string(),
            MerchantCategory::Bookstore,
            "0x333333333".to_string(),
            None,
            None,
            None,
        ).unwrap();
        
        let clinic_id = service.register_merchant(
            "Clinic".to_string(),
            MerchantCategory::Clinic,
            "0x444444444".to_string(),
            None,
            None,
            None,
        ).unwrap();
        
        let repair_shop_id = service.register_merchant(
            "Repair Shop".to_string(),
            MerchantCategory::RepairShop,
            "0x555555555".to_string(),
            None,
            None,
            None,
        ).unwrap();
        
        // Verify all merchants exist
        assert!(service.get_merchant(&coffee_shop_id).is_some());
        assert!(service.get_merchant(&restaurant_id).is_some());
        assert!(service.get_merchant(&bookstore_id).is_some());
        assert!(service.get_merchant(&clinic_id).is_some());
        assert!(service.get_merchant(&repair_shop_id).is_some());
        
        // Check categories
        assert_eq!(service.get_merchant(&coffee_shop_id).unwrap().category, MerchantCategory::CoffeeShop);
        assert_eq!(service.get_merchant(&restaurant_id).unwrap().category, MerchantCategory::Restaurant);
        assert_eq!(service.get_merchant(&bookstore_id).unwrap().category, MerchantCategory::Bookstore);
        assert_eq!(service.get_merchant(&clinic_id).unwrap().category, MerchantCategory::Clinic);
        assert_eq!(service.get_merchant(&repair_shop_id).unwrap().category, MerchantCategory::RepairShop);
    }
}