use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Supply chain categories for tracked donation items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SupplyCategory {
    Medicine,
    Food,
    Supplies,
    Water,
    Equipment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonationItem {
    pub id: String,
    pub category: SupplyCategory,
    pub description: String,
    pub quantity: f64,
    pub unit: String,
    pub expiration: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogisticsEvent {
    pub timestamp: NaiveDateTime,
    pub location: String,
    pub status: String,
    pub handler: String,
    pub notes: Option<String>,
}

impl LogisticsEvent {
    pub fn new(location: &str, status: &str, handler: &str, notes: Option<String>) -> Self {
        Self {
            timestamp: Utc::now().naive_utc(),
            location: location.to_string(),
            status: status.to_string(),
            handler: handler.to_string(),
            notes,
        }
    }
}

/// Severity levels for anti-corruption alerts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiCorruptionAlert {
    pub timestamp: NaiveDateTime,
    pub reporter: String,
    pub description: String,
    pub severity: AlertSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SupplyChainError {
    ShipmentAlreadyExists,
    ShipmentNotFound,
    AlreadyDelivered,
    UnauthorizedHandler,
    RouteDeviation,
    IntegrityMismatch,
}

impl fmt::Display for SupplyChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SupplyChainError::ShipmentAlreadyExists => write!(f, "Shipment already exists"),
            SupplyChainError::ShipmentNotFound => write!(f, "Shipment not found"),
            SupplyChainError::AlreadyDelivered => write!(f, "Shipment already delivered"),
            SupplyChainError::UnauthorizedHandler => write!(f, "Unauthorized logistics handler"),
            SupplyChainError::RouteDeviation => write!(f, "Event deviates from expected route"),
            SupplyChainError::IntegrityMismatch => {
                write!(f, "Shipment integrity verification failed")
            }
        }
    }
}

impl std::error::Error for SupplyChainError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidShipment {
    pub shipment_id: String,
    pub donor_id: String,
    pub ngo_address: String,
    pub origin: String,
    pub destination: String,
    pub carrier: String,
    pub items: Vec<DonationItem>,
    pub expected_route: Vec<String>,
    pub authorized_handlers: HashSet<String>,
    pub events: Vec<LogisticsEvent>,
    pub chain_hash: String,
    pub is_delivered: bool,
    pub flags: Vec<String>,
    pub theft_alerts: Vec<AntiCorruptionAlert>,
}

impl AidShipment {
    pub fn new(
        shipment_id: String,
        donor_id: String,
        ngo_address: String,
        origin: String,
        destination: String,
        carrier: String,
        items: Vec<DonationItem>,
        expected_route: Vec<String>,
        authorized_handlers: HashSet<String>,
    ) -> Self {
        let base_hash =
            Self::initial_hash(&shipment_id, &origin, &destination, &carrier, &donor_id);
        Self {
            shipment_id,
            donor_id,
            ngo_address,
            origin,
            destination,
            carrier,
            items,
            expected_route,
            authorized_handlers,
            events: Vec::new(),
            chain_hash: base_hash,
            is_delivered: false,
            flags: Vec::new(),
            theft_alerts: Vec::new(),
        }
    }

    fn initial_hash(
        shipment_id: &str,
        origin: &str,
        destination: &str,
        carrier: &str,
        donor_id: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(shipment_id.as_bytes());
        hasher.update(origin.as_bytes());
        hasher.update(destination.as_bytes());
        hasher.update(carrier.as_bytes());
        hasher.update(donor_id.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn fingerprint(prev_hash: &str, event: &LogisticsEvent) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prev_hash.as_bytes());
        hasher.update(event.location.as_bytes());
        hasher.update(event.status.as_bytes());
        hasher.update(event.handler.as_bytes());
        if let Some(notes) = &event.notes {
            hasher.update(notes.as_bytes());
        }
        hasher.update(event.timestamp.timestamp().to_string().as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn append_event(&mut self, event: LogisticsEvent) -> Result<(), SupplyChainError> {
        if self.is_delivered {
            return Err(SupplyChainError::AlreadyDelivered);
        }
        let next_hash = Self::fingerprint(&self.chain_hash, &event);
        self.chain_hash = next_hash;
        if event.status.eq_ignore_ascii_case("delivered") {
            self.is_delivered = true;
        }
        self.events.push(event);
        Ok(())
    }

    pub fn recalc_hash(&self) -> String {
        let mut current = Self::initial_hash(
            &self.shipment_id,
            &self.origin,
            &self.destination,
            &self.carrier,
            &self.donor_id,
        );
        for event in &self.events {
            current = Self::fingerprint(&current, event);
        }
        current
    }

    pub fn flag_issue(&mut self, reason: String) {
        self.flags.push(reason);
    }

    pub fn report_theft(&mut self, reporter: String, description: String, severity: AlertSeverity) {
        self.theft_alerts.push(AntiCorruptionAlert {
            timestamp: Utc::now().naive_utc(),
            reporter,
            description,
            severity,
        });
    }

    pub fn latest_event(&self) -> Option<&LogisticsEvent> {
        self.events.last()
    }
}

#[derive(Debug, Default)]
pub struct SupplyChainTracker {
    shipments: HashMap<String, AidShipment>,
    authorized_providers: HashSet<String>,
}

impl SupplyChainTracker {
    pub fn new() -> Self {
        Self {
            shipments: HashMap::new(),
            authorized_providers: HashSet::new(),
        }
    }

    pub fn authorize_provider(&mut self, provider_id: String) {
        self.authorized_providers.insert(provider_id);
    }

    pub fn register_shipment(&mut self, shipment: AidShipment) -> Result<(), SupplyChainError> {
        if self.shipments.contains_key(&shipment.shipment_id) {
            return Err(SupplyChainError::ShipmentAlreadyExists);
        }
        self.shipments
            .insert(shipment.shipment_id.clone(), shipment);
        Ok(())
    }

    pub fn log_event(
        &mut self,
        shipment_id: &str,
        event: LogisticsEvent,
    ) -> Result<(), SupplyChainError> {
        let shipment = self
            .shipments
            .get_mut(shipment_id)
            .ok_or(SupplyChainError::ShipmentNotFound)?;

        if !shipment.authorized_handlers.contains(&event.handler)
            && !self.authorized_providers.contains(&event.handler)
        {
            shipment.flag_issue(format!("Unauthorized handler recorded: {}", event.handler));
        }

        if !shipment
            .expected_route
            .iter()
            .any(|location| location == &event.location)
        {
            shipment.flag_issue(format!("Route deviation reported: {}", event.location));
        }

        shipment.append_event(event)
    }

    pub fn verify_shipment_integrity(&self, shipment_id: &str) -> Result<bool, SupplyChainError> {
        let shipment = self
            .shipments
            .get(shipment_id)
            .ok_or(SupplyChainError::ShipmentNotFound)?;
        Ok(shipment.chain_hash == shipment.recalc_hash())
    }

    pub fn report_theft(
        &mut self,
        shipment_id: &str,
        reporter: String,
        description: String,
        severity: AlertSeverity,
    ) -> Result<(), SupplyChainError> {
        let shipment = self
            .shipments
            .get_mut(shipment_id)
            .ok_or(SupplyChainError::ShipmentNotFound)?;
        shipment.report_theft(reporter, description, severity);
        shipment.flag_issue("Theft alert filed".to_string());
        Ok(())
    }

    pub fn flagged_shipments(&self) -> Vec<&AidShipment> {
        self.shipments
            .values()
            .filter(|shipment| !shipment.flags.is_empty() || !shipment.theft_alerts.is_empty())
            .collect()
    }

    pub fn get_shipment(&self, shipment_id: &str) -> Option<&AidShipment> {
        self.shipments.get(shipment_id)
    }
}
