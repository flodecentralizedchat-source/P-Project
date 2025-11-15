#[cfg(test)]
mod tests {
    use super::super::web2_service::{
        EcommerceConfig, EcommercePaymentRequest, Web2Service, Web2ServiceConfig,
    };
    use tokio;

    #[test]
    fn test_create_ecommerce_integration() {
        let config = Web2ServiceConfig {
            api_keys: std::collections::HashMap::new(),
            webhook_url: "".to_string(),
        };
        let mut service = Web2Service::new(config);

        let ecommerce_config = EcommerceConfig {
            platform: "shopify".to_string(),
            api_key: "test_key".to_string(),
            api_secret: "test_secret".to_string(),
            store_url: "https://test-shop.myshopify.com".to_string(),
            webhook_secret: Some("webhook_secret".to_string()),
        };

        let integration_id = service
            .create_ecommerce_integration(ecommerce_config)
            .unwrap();
        assert!(integration_id.starts_with("ecom_"));
    }

    #[tokio::test]
    async fn test_process_ecommerce_payment() {
        let config = Web2ServiceConfig {
            api_keys: std::collections::HashMap::new(),
            webhook_url: "".to_string(),
        };
        let service = Web2Service::new(config);

        let payment_request = EcommercePaymentRequest {
            order_id: "order_123".to_string(),
            customer_wallet: "0x123456789".to_string(),
            amount: 25.99,
            currency: "P".to_string(),
            platform: "shopify".to_string(),
            webhook_data: None,
        };

        let response = service
            .process_ecommerce_payment(payment_request)
            .await
            .unwrap();
        assert_eq!(response.order_id, "order_123");
        assert_eq!(response.status, "success");
        assert!(response.tx_hash.is_some());
    }

    #[test]
    fn test_verify_shopify_webhook() {
        let config = Web2ServiceConfig {
            api_keys: std::collections::HashMap::new(),
            webhook_url: "".to_string(),
        };
        let service = Web2Service::new(config);

        // This is a mock test - in a real scenario, we would have actual webhook data
        let body = b"test body";
        let signature = "invalid_signature";
        let secret = "test_secret";

        let result = service.verify_shopify_webhook(body, signature, secret);
        // This should fail with invalid signature
        assert!(!result);
    }

    #[test]
    fn test_verify_woocommerce_webhook() {
        let config = Web2ServiceConfig {
            api_keys: std::collections::HashMap::new(),
            webhook_url: "".to_string(),
        };
        let service = Web2Service::new(config);

        // This is a mock test - in a real scenario, we would have actual webhook data
        let body = b"test body";
        let signature = "invalid_signature";
        let secret = "test_secret";

        let result = service.verify_woocommerce_webhook(body, signature, secret);
        // This should fail with invalid signature
        assert!(!result);
    }
}
