#[cfg(test)]
mod tests {
    use super::super::merchant_service::{
        DigitalGoodsProduct, MerchantCategory, MerchantPaymentRequest, MerchantService,
        MerchantServiceConfig,
    };
    use chrono::Utc;
    use tokio;

    #[test]
    fn test_merchant_registration() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);

        let merchant_id = service
            .register_merchant(
                "Test Coffee Shop".to_string(),
                MerchantCategory::CoffeeShop,
                "0x123456789".to_string(),
                Some("A cozy coffee shop".to_string()),
                Some("123 Main St".to_string()),
                Some("contact@testcoffee.com".to_string()),
            )
            .unwrap();

        assert!(merchant_id.starts_with("merchant_"));

        let merchant = service.get_merchant(&merchant_id).unwrap();
        assert_eq!(merchant.name, "Test Coffee Shop");
        assert_eq!(merchant.category, MerchantCategory::CoffeeShop);
        assert_eq!(merchant.wallet_address, "0x123456789");
        assert_eq!(merchant.description, Some("A cozy coffee shop".to_string()));
        assert_eq!(merchant.location, Some("123 Main St".to_string()));
        assert_eq!(
            merchant.contact_info,
            Some("contact@testcoffee.com".to_string())
        );
        assert!(!merchant.is_verified);
    }

    #[test]
    fn test_merchant_verification() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);

        let merchant_id = service
            .register_merchant(
                "Test Restaurant".to_string(),
                MerchantCategory::Restaurant,
                "0x987654321".to_string(),
                None,
                None,
                None,
            )
            .unwrap();

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

        let merchant_id = service
            .register_merchant(
                "Test Bookstore".to_string(),
                MerchantCategory::Bookstore,
                "0x111111111".to_string(),
                None,
                None,
                None,
            )
            .unwrap();

        // Verify merchant first
        service.verify_merchant(&merchant_id).unwrap();

        // Create QR code
        let qr_id = service
            .create_payment_qr(
                merchant_id.clone(),
                25.99,
                "P".to_string(),
                Some("Book purchase".to_string()),
                Some(3600), // Expires in 1 hour
            )
            .unwrap();

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

        let merchant_id = service
            .register_merchant(
                "Test Clinic".to_string(),
                MerchantCategory::Clinic,
                "0x222222222".to_string(),
                None,
                None,
                None,
            )
            .unwrap();

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

        let merchant_id = service
            .register_merchant(
                "Test Repair Shop".to_string(),
                MerchantCategory::RepairShop,
                "0x444444444".to_string(),
                None,
                None,
                None,
            )
            .unwrap();

        // Verify merchant
        service.verify_merchant(&merchant_id).unwrap();

        // Create QR code
        let qr_id = service
            .create_payment_qr(
                merchant_id.clone(),
                120.00,
                "P".to_string(),
                Some("Laptop repair".to_string()),
                Some(3600),
            )
            .unwrap();

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

        let merchant_id = service
            .register_merchant(
                "Unverified Shop".to_string(),
                MerchantCategory::CoffeeShop,
                "0x777777777".to_string(),
                None,
                None,
                None,
            )
            .unwrap();

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
        service
            .register_merchant(
                "Coffee Shop".to_string(),
                MerchantCategory::CoffeeShop,
                "0x111111111".to_string(),
                None,
                None,
                None,
            )
            .unwrap();

        service
            .register_merchant(
                "Restaurant".to_string(),
                MerchantCategory::Restaurant,
                "0x222222222".to_string(),
                None,
                None,
                None,
            )
            .unwrap();

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
        let coffee_shop_id = service
            .register_merchant(
                "Coffee Shop".to_string(),
                MerchantCategory::CoffeeShop,
                "0x111111111".to_string(),
                None,
                None,
                None,
            )
            .unwrap();

        let restaurant_id = service
            .register_merchant(
                "Restaurant".to_string(),
                MerchantCategory::Restaurant,
                "0x222222222".to_string(),
                None,
                None,
                None,
            )
            .unwrap();

        let bookstore_id = service
            .register_merchant(
                "Bookstore".to_string(),
                MerchantCategory::Bookstore,
                "0x333333333".to_string(),
                None,
                None,
                None,
            )
            .unwrap();

        let clinic_id = service
            .register_merchant(
                "Clinic".to_string(),
                MerchantCategory::Clinic,
                "0x444444444".to_string(),
                None,
                None,
                None,
            )
            .unwrap();

        let repair_shop_id = service
            .register_merchant(
                "Repair Shop".to_string(),
                MerchantCategory::RepairShop,
                "0x555555555".to_string(),
                None,
                None,
                None,
            )
            .unwrap();

        // Verify all merchants exist
        assert!(service.get_merchant(&coffee_shop_id).is_some());
        assert!(service.get_merchant(&restaurant_id).is_some());
        assert!(service.get_merchant(&bookstore_id).is_some());
        assert!(service.get_merchant(&clinic_id).is_some());
        assert!(service.get_merchant(&repair_shop_id).is_some());

        // Check categories
        assert_eq!(
            service.get_merchant(&coffee_shop_id).unwrap().category,
            MerchantCategory::CoffeeShop
        );
        assert_eq!(
            service.get_merchant(&restaurant_id).unwrap().category,
            MerchantCategory::Restaurant
        );
        assert_eq!(
            service.get_merchant(&bookstore_id).unwrap().category,
            MerchantCategory::Bookstore
        );
        assert_eq!(
            service.get_merchant(&clinic_id).unwrap().category,
            MerchantCategory::Clinic
        );
        assert_eq!(
            service.get_merchant(&repair_shop_id).unwrap().category,
            MerchantCategory::RepairShop
        );
    }

    // Add tests for digital goods functionality
    #[test]
    fn test_add_digital_goods_product() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);

        // Register a merchant first
        let merchant_id = service
            .register_merchant(
                "Test Digital Store".to_string(),
                MerchantCategory::Other("digital".to_string()),
                "0x123456789".to_string(),
                Some("A digital goods store".to_string()),
                Some("Online".to_string()),
                Some("contact@digitalstore.com".to_string()),
            )
            .unwrap();

        // Verify merchant
        service.verify_merchant(&merchant_id).unwrap();

        // Create a digital goods product
        let product = DigitalGoodsProduct {
            product_id: "product_123".to_string(),
            name: "Test E-book".to_string(),
            description: "A test e-book for digital goods".to_string(),
            price: 9.99,
            currency: "P".to_string(),
            category: "ebook".to_string(),
            country_restrictions: Some(vec!["CN".to_string(), "RU".to_string()]),
            language_availability: vec!["en".to_string(), "es".to_string()],
            digital_delivery_method: "download".to_string(),
            download_url: Some("https://example.com/download/test-ebook.pdf".to_string()),
            license_key_template: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        let result = service.add_digital_goods_product(&merchant_id, product);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_purchase_digital_good() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);

        // Register a merchant first
        let merchant_id = service
            .register_merchant(
                "Test Digital Store".to_string(),
                MerchantCategory::Other("digital".to_string()),
                "0x123456789".to_string(),
                Some("A digital goods store".to_string()),
                Some("Online".to_string()),
                Some("contact@digitalstore.com".to_string()),
            )
            .unwrap();

        // Verify merchant
        service.verify_merchant(&merchant_id).unwrap();

        // Create a digital goods product
        let product = DigitalGoodsProduct {
            product_id: "product_123".to_string(),
            name: "Test E-book".to_string(),
            description: "A test e-book for digital goods".to_string(),
            price: 9.99,
            currency: "P".to_string(),
            category: "ebook".to_string(),
            country_restrictions: Some(vec!["CN".to_string(), "RU".to_string()]),
            language_availability: vec!["en".to_string(), "es".to_string()],
            digital_delivery_method: "download".to_string(),
            download_url: Some("https://example.com/download/test-ebook.pdf".to_string()),
            license_key_template: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        service
            .add_digital_goods_product(&merchant_id, product)
            .unwrap();

        // Purchase the digital good
        let transaction = service
            .purchase_digital_good(
                "product_123".to_string(),
                "customer_123".to_string(),
                "US".to_string(), // Not restricted
                "en".to_string(), // Available in this language
            )
            .await
            .unwrap();

        assert_eq!(transaction.product_id, "product_123");
        assert_eq!(transaction.customer_id, "customer_123");
        assert_eq!(transaction.amount, 9.99);
        assert_eq!(transaction.status, "completed");
        assert_eq!(transaction.delivery_status, "delivered");
        assert!(transaction.delivery_data.is_some());
    }

    #[tokio::test]
    async fn test_purchase_restricted_digital_good() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);

        // Register a merchant first
        let merchant_id = service
            .register_merchant(
                "Test Digital Store".to_string(),
                MerchantCategory::Other("digital".to_string()),
                "0x123456789".to_string(),
                Some("A digital goods store".to_string()),
                Some("Online".to_string()),
                Some("contact@digitalstore.com".to_string()),
            )
            .unwrap();

        // Verify merchant
        service.verify_merchant(&merchant_id).unwrap();

        // Create a digital goods product with country restrictions
        let product = DigitalGoodsProduct {
            product_id: "product_456".to_string(),
            name: "Restricted Software".to_string(),
            description: "Software restricted in certain countries".to_string(),
            price: 29.99,
            currency: "P".to_string(),
            category: "software".to_string(),
            country_restrictions: Some(vec!["CN".to_string(), "RU".to_string()]),
            language_availability: vec!["en".to_string()],
            digital_delivery_method: "license_key".to_string(),
            download_url: None,
            license_key_template: Some("LICENSE".to_string()),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };

        service
            .add_digital_goods_product(&merchant_id, product)
            .unwrap();

        // Try to purchase from a restricted country
        let result = service
            .purchase_digital_good(
                "product_456".to_string(),
                "customer_456".to_string(),
                "CN".to_string(), // Restricted country
                "en".to_string(),
            )
            .await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Product is not available in your country"
        );
    }

    #[test]
    fn test_get_digital_goods_transaction() {
        // This would require setting up a transaction first, which is complex in a test
        // For now, we'll just verify the method exists and returns None for non-existent transactions
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let service = MerchantService::new(config);

        let transaction = service.get_digital_goods_transaction("nonexistent_tx");
        assert!(transaction.is_none());
    }

    #[test]
    fn test_get_customer_digital_goods_transactions() {
        // This would require setting up transactions first, which is complex in a test
        // For now, we'll just verify the method exists and returns an empty vector for non-existent customers
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let service = MerchantService::new(config);

        let transactions = service.get_customer_digital_goods_transactions("nonexistent_customer");
        assert!(transactions.is_empty());
    }

    // Add tests for cashback and loyalty functionality
    #[test]
    fn test_configure_cashback() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);

        // Register a merchant first
        let merchant_id = service
            .register_merchant(
                "Test Store".to_string(),
                MerchantCategory::Other("retail".to_string()),
                "0x123456789".to_string(),
                Some("A test store".to_string()),
                Some("Online".to_string()),
                Some("contact@teststore.com".to_string()),
            )
            .unwrap();

        // Verify merchant
        service.verify_merchant(&merchant_id).unwrap();

        // Configure cashback
        let result = service.configure_cashback(
            merchant_id.clone(),
            5.0,  // 5% cashback
            10.0, // Minimum $10 purchase
            50.0, // Maximum $50 cashback
            1.0,  // 1 loyalty point per P-Coin spent
        );

        assert!(result.is_ok());

        // Check that the configuration was stored
        let cashback_config = service.get_cashback_config(&merchant_id).unwrap();
        assert_eq!(cashback_config.cashback_percentage, 5.0);
        assert_eq!(cashback_config.min_purchase_amount, 10.0);
        assert_eq!(cashback_config.max_cashback_amount, 50.0);
        assert_eq!(cashback_config.loyalty_points_per_coin, 1.0);
        assert!(cashback_config.is_active);
    }

    #[tokio::test]
    async fn test_calculate_cashback() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);

        // Register a merchant first
        let merchant_id = service
            .register_merchant(
                "Test Store".to_string(),
                MerchantCategory::Other("retail".to_string()),
                "0x123456789".to_string(),
                Some("A test store".to_string()),
                Some("Online".to_string()),
                Some("contact@teststore.com".to_string()),
            )
            .unwrap();

        // Verify merchant
        service.verify_merchant(&merchant_id).unwrap();

        // Configure cashback
        service
            .configure_cashback(
                merchant_id.clone(),
                5.0,  // 5% cashback
                10.0, // Minimum $10 purchase
                50.0, // Maximum $50 cashback
                1.0,  // 1 loyalty point per P-Coin spent
            )
            .unwrap();

        // Test purchase below minimum amount
        let (cashback, loyalty) = service.calculate_cashback(&merchant_id, 5.0).unwrap();
        assert_eq!(cashback, 0.0);
        assert_eq!(loyalty, 0.0);

        // Test purchase at minimum amount
        let (cashback, loyalty) = service.calculate_cashback(&merchant_id, 10.0).unwrap();
        assert_eq!(cashback, 0.5); // 5% of 10.0
        assert_eq!(loyalty, 10.0); // 1 point per P-Coin

        // Test purchase with normal amount
        let (cashback, loyalty) = service.calculate_cashback(&merchant_id, 100.0).unwrap();
        assert_eq!(cashback, 5.0); // 5% of 100.0
        assert_eq!(loyalty, 100.0); // 1 point per P-Coin

        // Test purchase that exceeds maximum cashback
        let (cashback, loyalty) = service.calculate_cashback(&merchant_id, 2000.0).unwrap();
        assert_eq!(cashback, 50.0); // Capped at maximum of 50.0
        assert_eq!(loyalty, 2000.0); // 1 point per P-Coin
    }

    #[tokio::test]
    async fn test_process_purchase_with_cashback() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);

        // Register a merchant first
        let merchant_id = service
            .register_merchant(
                "Test Store".to_string(),
                MerchantCategory::Other("retail".to_string()),
                "0x123456789".to_string(),
                Some("A test store".to_string()),
                Some("Online".to_string()),
                Some("contact@teststore.com".to_string()),
            )
            .unwrap();

        // Verify merchant
        service.verify_merchant(&merchant_id).unwrap();

        // Configure cashback
        service
            .configure_cashback(
                merchant_id.clone(),
                5.0,  // 5% cashback
                10.0, // Minimum $10 purchase
                50.0, // Maximum $50 cashback
                1.0,  // 1 loyalty point per P-Coin spent
            )
            .unwrap();

        // Process a purchase
        let cashback_tx = service
            .process_purchase_with_cashback(
                merchant_id.clone(),
                "customer_123".to_string(),
                100.0, // $100 purchase
                "0x987654321".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(cashback_tx.customer_id, "customer_123");
        assert_eq!(cashback_tx.merchant_id, merchant_id);
        assert_eq!(cashback_tx.original_amount, 100.0);
        assert_eq!(cashback_tx.cashback_amount, 5.0); // 5% of 100.0
        assert_eq!(cashback_tx.cashback_percentage, 5.0);
        assert_eq!(cashback_tx.loyalty_points_earned, 100.0); // 1 point per P-Coin
        assert_eq!(cashback_tx.status, "completed");
        assert!(cashback_tx.tx_hash.is_some());

        // Check that loyalty points were updated
        let loyalty_points = service
            .get_customer_loyalty_points("customer_123", &merchant_id)
            .unwrap();
        assert_eq!(loyalty_points.points, 100.0);
        assert_eq!(loyalty_points.total_earned, 100.0);
        assert_eq!(loyalty_points.total_spent, 0.0);
    }

    #[test]
    fn test_get_customer_loyalty_points() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);

        // Register a merchant first
        let merchant_id = service
            .register_merchant(
                "Test Store".to_string(),
                MerchantCategory::Other("retail".to_string()),
                "0x123456789".to_string(),
                Some("A test store".to_string()),
                Some("Online".to_string()),
                Some("contact@teststore.com".to_string()),
            )
            .unwrap();

        // Verify merchant
        service.verify_merchant(&merchant_id).unwrap();

        // Configure cashback
        service
            .configure_cashback(
                merchant_id.clone(),
                5.0,  // 5% cashback
                10.0, // Minimum $10 purchase
                50.0, // Maximum $50 cashback
                1.0,  // 1 loyalty point per P-Coin spent
            )
            .unwrap();

        // Initially, customer should have no loyalty points
        let loyalty_points = service.get_customer_loyalty_points("customer_123", &merchant_id);
        assert!(loyalty_points.is_none());

        // Process a purchase to earn points
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            service
                .process_purchase_with_cashback(
                    merchant_id.clone(),
                    "customer_123".to_string(),
                    100.0,
                    "0x987654321".to_string(),
                )
                .await
                .unwrap();
        });

        // Now customer should have loyalty points
        let loyalty_points = service
            .get_customer_loyalty_points("customer_123", &merchant_id)
            .unwrap();
        assert_eq!(loyalty_points.points, 100.0);
        assert_eq!(loyalty_points.customer_id, "customer_123");
        assert_eq!(loyalty_points.merchant_id, merchant_id);
    }

    #[test]
    fn test_redeem_loyalty_points() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);

        // Register a merchant first
        let merchant_id = service
            .register_merchant(
                "Test Store".to_string(),
                MerchantCategory::Other("retail".to_string()),
                "0x123456789".to_string(),
                Some("A test store".to_string()),
                Some("Online".to_string()),
                Some("contact@teststore.com".to_string()),
            )
            .unwrap();

        // Verify merchant
        service.verify_merchant(&merchant_id).unwrap();

        // Configure cashback
        service
            .configure_cashback(
                merchant_id.clone(),
                5.0,  // 5% cashback
                10.0, // Minimum $10 purchase
                50.0, // Maximum $50 cashback
                1.0,  // 1 loyalty point per P-Coin spent
            )
            .unwrap();

        // Process a purchase to earn points
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            service
                .process_purchase_with_cashback(
                    merchant_id.clone(),
                    "customer_123".to_string(),
                    100.0,
                    "0x987654321".to_string(),
                )
                .await
                .unwrap();
        });

        // Redeem loyalty points
        let coin_value = service
            .redeem_loyalty_points("customer_123", &merchant_id, 50.0)
            .unwrap();
        assert_eq!(coin_value, 0.5); // 50 points * 0.01 P-Coin per point

        // Check that points were deducted
        let loyalty_points = service
            .get_customer_loyalty_points("customer_123", &merchant_id)
            .unwrap();
        assert_eq!(loyalty_points.points, 50.0); // 100 - 50 = 50
        assert_eq!(loyalty_points.total_spent, 50.0);
    }

    #[tokio::test]
    async fn test_redeem_insufficient_loyalty_points() {
        let config = MerchantServiceConfig {
            fee_percentage: 0.01,
            max_transaction_amount: 10000.0,
        };
        let mut service = MerchantService::new(config);

        // Register a merchant first
        let merchant_id = service
            .register_merchant(
                "Test Store".to_string(),
                MerchantCategory::Other("retail".to_string()),
                "0x123456789".to_string(),
                Some("A test store".to_string()),
                Some("Online".to_string()),
                Some("contact@teststore.com".to_string()),
            )
            .unwrap();

        // Verify merchant
        service.verify_merchant(&merchant_id).unwrap();

        // Try to redeem points without having any
        let result = service.redeem_loyalty_points("customer_123", &merchant_id, 50.0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No loyalty points found for this customer and merchant"
        );
    }
}
