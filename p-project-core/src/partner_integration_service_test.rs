#[cfg(test)]
mod tests {
    use crate::{models::PartnerIntegrationType, partner_integration_service::PartnerRegistry};
    use serde_json::json;

    #[test]
    fn register_and_fetch_partner() {
        let mut reg = PartnerRegistry::new();
        let p = reg.register_partner(
            "ChainLink",
            PartnerIntegrationType::Oracle,
            json!({"network": "Ethereum"}),
            Some("secret123".to_string()),
            true,
        );
        assert!(reg.get_partner(&p.id).is_some());
        assert_eq!(p.name, "ChainLink");
        assert!(p.active);
        assert_eq!(p.integration_type, PartnerIntegrationType::Oracle);
    }

    #[test]
    fn webhook_sign_and_verify() {
        let payload = "{\"event\":\"update\"}";
        let secret = "s3cr3t";
        let sig = PartnerRegistry::sign_webhook(payload, secret);
        assert!(PartnerRegistry::verify_webhook(payload, secret, &sig));
        assert!(!PartnerRegistry::verify_webhook(payload, "wrong", &sig));
    }
}
