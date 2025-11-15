#[cfg(test)]
mod tests {
    use crate::supply_chain::{
        AidShipment, AlertSeverity, DonationItem, LogisticsEvent, SupplyCategory,
        SupplyChainTracker,
    };
    use std::collections::HashSet;

    #[test]
    fn test_shipment_event_sequence_and_integrity() {
        let mut tracker = SupplyChainTracker::new();
        tracker.authorize_provider("carrier-A".to_string());

        let items = vec![DonationItem {
            id: "batch-001".to_string(),
            category: SupplyCategory::Medicine,
            description: "Antimalarial kits".to_string(),
            quantity: 300.0,
            unit: "boxes".to_string(),
            expiration: None,
        }];

        let mut handlers = HashSet::new();
        handlers.insert("warehouse-ops".to_string());

        let mut shipment = AidShipment::new(
            "shipment-001".to_string(),
            "donor-xyz".to_string(),
            "ngo-relief".to_string(),
            "Warehouse A".to_string(),
            "NGO HQ".to_string(),
            "P-Logistics".to_string(),
            items,
            vec![
                "Warehouse A".to_string(),
                "Checkpoint B".to_string(),
                "NGO HQ".to_string(),
            ],
            handlers,
        );

        tracker.register_shipment(shipment).unwrap();

        tracker
            .log_event(
                "shipment-001",
                LogisticsEvent::new("Warehouse A", "dispatched", "warehouse-ops", None),
            )
            .unwrap();
        tracker
            .log_event(
                "shipment-001",
                LogisticsEvent::new("Checkpoint B", "in transit", "carrier-A", None),
            )
            .unwrap();
        tracker
            .log_event(
                "shipment-001",
                LogisticsEvent::new("NGO HQ", "delivered", "carrier-A", Some("Signed".into())),
            )
            .unwrap();

        let integrity = tracker.verify_shipment_integrity("shipment-001").unwrap();
        assert!(integrity);

        let shipment = tracker.get_shipment("shipment-001").unwrap();
        assert!(shipment.flags.is_empty());
        assert!(shipment.is_delivered);
    }

    #[test]
    fn test_logistics_flags_for_unauthorized_activity() {
        let mut tracker = SupplyChainTracker::new();
        let mut handlers = HashSet::new();
        handlers.insert("warehouse-ops".to_string());

        let shipment = AidShipment::new(
            "shipment-002".to_string(),
            "donor-abc".to_string(),
            "ngo-med".to_string(),
            "Port".to_string(),
            "Clinic".to_string(),
            "GlobalCarrier".to_string(),
            vec![DonationItem {
                id: "batch-007".to_string(),
                category: SupplyCategory::Supplies,
                description: "Shelter tents".to_string(),
                quantity: 150.0,
                unit: "sets".to_string(),
                expiration: None,
            }],
            vec!["Port".to_string(), "Clinic".to_string()],
            handlers,
        );

        tracker.register_shipment(shipment).unwrap();

        tracker
            .log_event(
                "shipment-002",
                LogisticsEvent::new("Port", "dispatched", "warehouse-ops", None),
            )
            .unwrap();

        tracker
            .log_event(
                "shipment-002",
                LogisticsEvent::new(
                    "Unknown Yard",
                    "handover",
                    "malicious",
                    Some("Route spoof".into()),
                ),
            )
            .unwrap();

        let flagged = tracker.flagged_shipments();
        assert_eq!(flagged.len(), 1);
        let shipment = flagged.first().unwrap();
        assert!(shipment
            .flags
            .iter()
            .any(|f| f.contains("Unauthorized handler")));
        assert!(shipment.flags.iter().any(|f| f.contains("Route deviation")));
    }

    #[test]
    fn test_theft_alert_tracking() {
        let mut tracker = SupplyChainTracker::new();
        let shipment = AidShipment::new(
            "shipment-003".to_string(),
            "donor-123".to_string(),
            "ngo-thrive".to_string(),
            "Depot".to_string(),
            "Camp".to_string(),
            "SecureCarriers".to_string(),
            Vec::new(),
            vec!["Depot".to_string(), "Camp".to_string()],
            HashSet::new(),
        );

        tracker.register_shipment(shipment).unwrap();

        tracker
            .report_theft(
                "shipment-003",
                "inspector-1".to_string(),
                "Seal broken before storage".to_string(),
                AlertSeverity::High,
            )
            .unwrap();

        let flagged = tracker.flagged_shipments();
        assert_eq!(flagged.len(), 1);
        let shipment = flagged.first().unwrap();
        assert!(!shipment.theft_alerts.is_empty());
        assert!(shipment
            .flags
            .iter()
            .any(|f| f.contains("Theft alert filed")));
    }
}
