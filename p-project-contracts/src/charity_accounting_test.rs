#[cfg(test)]
mod tests {
    use crate::charity::{CharityAllocator, DistributionRule};

    fn setup_allocator() -> CharityAllocator {
        let dao = "dao-admin".to_string();
        let mut allocator = CharityAllocator::new(dao.clone(), 0.0);

        allocator
            .register_ngo(
                &dao,
                "ngo-1".to_string(),
                "Health NGO".to_string(),
                "Medical supplies provider".to_string(),
                "health".to_string(),
                None,
                None,
            )
            .unwrap();
        allocator.verify_ngo(&dao, "ngo-1").unwrap();

        allocator
            .register_ngo(
                &dao,
                "ngo-2".to_string(),
                "Education NGO".to_string(),
                "School builder".to_string(),
                "education".to_string(),
                None,
                None,
            )
            .unwrap();
        allocator.verify_ngo(&dao, "ngo-2").unwrap();

        allocator
    }

    #[test]
    fn test_set_distribution_rules_and_auto_distribution() {
        let dao = "dao-admin".to_string();
        let mut allocator = setup_allocator();

        let rules = vec![
            DistributionRule {
                ngo_address: "ngo-1".to_string(),
                weight: 1.0,
                purpose: "Medical supplies".to_string(),
            },
            DistributionRule {
                ngo_address: "ngo-2".to_string(),
                weight: 2.0,
                purpose: "Education rescue".to_string(),
            },
        ];

        allocator.set_distribution_rules(rules).unwrap();

        let result = allocator
            .automate_donation_distribution(
                "donor-1".to_string(),
                1500.0,
                "tx-donation".to_string(),
                Some("Automated relief".to_string()),
            )
            .unwrap();

        assert_eq!(result.len(), 2);
        assert!((result[0].1 - 500.0).abs() < 1e-6);
        assert!((result[1].1 - 1000.0).abs() < 1e-6);

        let summary = allocator.get_dashboard_summary(2);
        assert_eq!(summary.total_donations, 1500.0);
        assert_eq!(summary.total_allocations, 1500.0);
        assert_eq!(summary.top_donors.len(), 1);
        assert_eq!(summary.top_donors[0].0, "donor-1");
        assert_eq!(summary.ngo_allocations.len(), 2);
        assert!(summary
            .ngo_allocations
            .iter()
            .any(|(ngo, amount)| ngo == "ngo-1" && (*amount - 500.0).abs() < 1e-6));

        let audit_events = allocator.get_recent_audit_events(5);
        assert!(audit_events
            .iter()
            .any(|event| event.action == "auto_distribution"));
        assert!(audit_events
            .iter()
            .any(|event| event.action == "donation_recorded"));
    }
}
