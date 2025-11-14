#[cfg(test)]
mod tests {
    use super::super::handlers::*;
    use axum::{http::StatusCode, Json};
    use p_project_core::merchant_service::{MerchantCategory, MerchantService, MerchantServiceConfig};
    use serde_json::json;
    
    #[tokio::test]
    async fn test_create_merchant_handler() {
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
    }
    
    #[tokio::test]
    async fn test_merchant_categories() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);
        
        // Test all required merchant categories from the use cases
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