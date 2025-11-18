use crate::models::{Partner, PartnerIntegrationType};
use chrono::Utc;
use std::collections::HashMap;

pub struct PartnerRegistry {
    partners: HashMap<String, Partner>,
}

impl PartnerRegistry {
    pub fn new() -> Self {
        Self {
            partners: HashMap::new(),
        }
    }

    pub fn register_partner(
        &mut self,
        name: &str,
        integration_type: PartnerIntegrationType,
        metadata: serde_json::Value,
        webhook_secret: Option<String>,
        active: bool,
    ) -> Partner {
        let now = Utc::now().naive_utc();
        let raw = format!(
            "{}:{}:{}",
            name,
            now.timestamp_millis(),
            webhook_secret.clone().unwrap_or_default()
        );
        let digest = format!("{:x}", md5::compute(raw.as_bytes()));
        let id = format!("partner_{}", &digest[..12]);

        let partner = Partner {
            id: id.clone(),
            name: name.to_string(),
            integration_type,
            metadata,
            webhook_secret,
            active,
            created_at: now,
        };

        self.partners.insert(id.clone(), partner.clone());
        partner
    }

    pub fn get_partner(&self, id: &str) -> Option<&Partner> {
        self.partners.get(id)
    }

    pub fn list_partners(&self) -> Vec<&Partner> {
        self.partners.values().collect()
    }

    // Simple MD5-based signature: md5_hex(secret + ":" + payload)
    pub fn sign_webhook(payload: &str, secret: &str) -> String {
        let raw = format!("{}:{}", secret, payload);
        format!("{:x}", md5::compute(raw.as_bytes()))
    }

    pub fn verify_webhook(payload: &str, secret: &str, signature_hex: &str) -> bool {
        Self::sign_webhook(payload, secret) == signature_hex
    }
}
