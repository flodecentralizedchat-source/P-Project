#[cfg(test)]
mod tests {
    use super::super::charity::{CharityAllocator, NGO};

    #[test]
    fn test_proof_of_peace_badge_creation() {
        let dao_address = "dao1".to_string();
        let mut allocator = CharityAllocator::new(dao_address.clone(), 10000.0);

        // Register and verify an NGO
        let ngo_result = allocator.register_ngo(
            &dao_address,
            "ngo1".to_string(),
            "Test NGO".to_string(),
            "A test NGO".to_string(),
            "education".to_string(),
            Some("https://testngo.org".to_string()),
            Some("contact@testngo.org".to_string()),
        );
        assert!(ngo_result.is_ok());

        let verify_result = allocator.verify_ngo(&dao_address, "ngo1");
        assert!(verify_result.is_ok());

        // Record a donation which should create a Proof-of-Peace badge
        let donation_result = allocator.record_donation(
            "donor1".to_string(),
            "ngo1".to_string(),
            150.0,
            "tx123".to_string(),
            None,
            Some("Test donation".to_string()),
        );
        assert!(donation_result.is_ok());

        // Check that a badge was created
        let donor_badges = allocator.get_donor_badges("donor1");
        assert_eq!(donor_badges.len(), 1);

        let badge = donor_badges[0];
        assert_eq!(badge.donor_address, "donor1");
        assert_eq!(badge.ngo_address, "ngo1");
        assert_eq!(badge.donation_amount, 150.0);
        assert_eq!(badge.badge_type, "silver"); // 150 is between 100-500 so silver
        assert_eq!(badge.tx_hash, "tx123");
    }

    #[test]
    fn test_donor_reputation_scoring() {
        let dao_address = "dao1".to_string();
        let mut allocator = CharityAllocator::new(dao_address.clone(), 10000.0);

        // Register and verify an NGO
        let ngo_result = allocator.register_ngo(
            &dao_address,
            "ngo1".to_string(),
            "Test NGO".to_string(),
            "A test NGO".to_string(),
            "education".to_string(),
            Some("https://testngo.org".to_string()),
            Some("contact@testngo.org".to_string()),
        );
        assert!(ngo_result.is_ok());

        let verify_result = allocator.verify_ngo(&dao_address, "ngo1");
        assert!(verify_result.is_ok());

        // Record multiple donations to test reputation scoring
        let donation1_result = allocator.record_donation(
            "donor1".to_string(),
            "ngo1".to_string(),
            50.0,
            "tx1".to_string(),
            None,
            None,
        );
        assert!(donation1_result.is_ok());

        // Check donor reputation after first donation
        let donor_rep = allocator.get_donor_reputation("donor1").unwrap();
        assert_eq!(donor_rep.donation_count, 1);
        assert_eq!(donor_rep.total_donations, 50.0);
        // Score should be: base(10) + amount(50/10=5) + consistency(0) = 15
        assert_eq!(donor_rep.score, 15.0);
        assert_eq!(donor_rep.badges_earned.len(), 1);

        // Check first badge type
        let badges1 = allocator.get_donor_badges("donor1");
        assert_eq!(badges1.len(), 1);
        assert_eq!(badges1[0].badge_type, "bronze"); // 50 < 100

        // Record second donation
        let donation2_result = allocator.record_donation(
            "donor1".to_string(),
            "ngo1".to_string(),
            200.0,
            "tx2".to_string(),
            None,
            None,
        );
        assert!(donation2_result.is_ok());

        // Check donor reputation after second donation
        let donor_rep = allocator.get_donor_reputation("donor1").unwrap();
        assert_eq!(donor_rep.donation_count, 2);
        assert_eq!(donor_rep.total_donations, 250.0);
        // Score should be: previous(15) + base(10) + amount(200/10=20) + consistency(5) = 50
        assert_eq!(donor_rep.score, 50.0);
        assert_eq!(donor_rep.badges_earned.len(), 2);

        // Check badge types
        let badges = allocator.get_donor_badges("donor1");
        assert_eq!(badges.len(), 2);
        // Find the badge for the 50.0 donation
        let badge_50 = badges.iter().find(|b| b.donation_amount == 50.0).unwrap();
        // Find the badge for the 200.0 donation
        let badge_200 = badges.iter().find(|b| b.donation_amount == 200.0).unwrap();
        assert_eq!(badge_50.badge_type, "bronze"); // First donation was 50 < 100
        assert_eq!(badge_200.badge_type, "silver"); // Second donation was 200 between 100-500
    }

    #[test]
    fn test_ngo_impact_tracking() {
        let dao_address = "dao1".to_string();
        let mut allocator = CharityAllocator::new(dao_address.clone(), 10000.0);

        // Register and verify an NGO
        let ngo_result = allocator.register_ngo(
            &dao_address,
            "ngo1".to_string(),
            "Test NGO".to_string(),
            "A test NGO".to_string(),
            "education".to_string(),
            Some("https://testngo.org".to_string()),
            Some("contact@testngo.org".to_string()),
        );
        assert!(ngo_result.is_ok());

        let verify_result = allocator.verify_ngo(&dao_address, "ngo1");
        assert!(verify_result.is_ok());

        // Record donations to the NGO
        let donation1_result = allocator.record_donation(
            "donor1".to_string(),
            "ngo1".to_string(),
            100.0,
            "tx1".to_string(),
            None,
            None,
        );
        assert!(donation1_result.is_ok());

        let donation2_result = allocator.record_donation(
            "donor2".to_string(),
            "ngo1".to_string(),
            300.0,
            "tx2".to_string(),
            None,
            None,
        );
        assert!(donation2_result.is_ok());

        // Check NGO impact record
        let ngo_impact = allocator.get_ngo_impact_record("ngo1").unwrap();
        assert_eq!(ngo_impact.total_received, 400.0);
        assert_eq!(ngo_impact.donor_count, 2);
        assert_eq!(ngo_impact.badges_issued, 2);
    }

    #[test]
    fn test_leaderboard_functionality() {
        let dao_address = "dao1".to_string();
        let mut allocator = CharityAllocator::new(dao_address.clone(), 10000.0);

        // Register and verify NGOs
        let ngo1_result = allocator.register_ngo(
            &dao_address,
            "ngo1".to_string(),
            "High Impact NGO".to_string(),
            "A high impact NGO".to_string(),
            "education".to_string(),
            None,
            None,
        );
        assert!(ngo1_result.is_ok());

        let ngo2_result = allocator.register_ngo(
            &dao_address,
            "ngo2".to_string(),
            "Low Impact NGO".to_string(),
            "A low impact NGO".to_string(),
            "health".to_string(),
            None,
            None,
        );
        assert!(ngo2_result.is_ok());

        let verify1_result = allocator.verify_ngo(&dao_address, "ngo1");
        assert!(verify1_result.is_ok());

        let verify2_result = allocator.verify_ngo(&dao_address, "ngo2");
        assert!(verify2_result.is_ok());

        // Create donors with different donation patterns
        let donation1_result = allocator.record_donation(
            "generous_donor".to_string(),
            "ngo1".to_string(),
            1000.0,
            "tx1".to_string(),
            None,
            None,
        );
        assert!(donation1_result.is_ok());

        let donation2_result = allocator.record_donation(
            "frequent_donor".to_string(),
            "ngo1".to_string(),
            50.0,
            "tx2".to_string(),
            None,
            None,
        );
        assert!(donation2_result.is_ok());

        let donation3_result = allocator.record_donation(
            "frequent_donor".to_string(),
            "ngo2".to_string(),
            75.0,
            "tx3".to_string(),
            None,
            None,
        );
        assert!(donation3_result.is_ok());

        // Check donor leaderboard
        let donor_leaderboard = allocator.get_donor_leaderboard(Some(10));
        assert!(!donor_leaderboard.is_empty());

        // Check NGO leaderboard
        let ngo_leaderboard = allocator.get_ngo_leaderboard(Some(10));
        assert_eq!(ngo_leaderboard.len(), 2);

        // The NGO with more donations should rank higher
        // ngo1 received 1050 total (1000 + 50)
        // ngo2 received 75 total
        assert_eq!(ngo_leaderboard[0].address, "ngo1");
        assert_eq!(ngo_leaderboard[1].address, "ngo2");

        // Check combined leaderboard
        let combined_leaderboard = allocator.get_combined_leaderboard(Some(10));
        assert!(!combined_leaderboard.is_empty());
    }

    #[test]
    fn test_badge_tier_system() {
        let dao_address = "dao1".to_string();
        let mut allocator = CharityAllocator::new(dao_address.clone(), 10000.0);

        // Register and verify an NGO
        let ngo_result = allocator.register_ngo(
            &dao_address,
            "ngo1".to_string(),
            "Test NGO".to_string(),
            "A test NGO".to_string(),
            "education".to_string(),
            None,
            None,
        );
        assert!(ngo_result.is_ok());

        let verify_result = allocator.verify_ngo(&dao_address, "ngo1");
        assert!(verify_result.is_ok());

        // Test bronze badge (donation < 100)
        let donation1_result = allocator.record_donation(
            "donor1".to_string(),
            "ngo1".to_string(),
            50.0,
            "tx1".to_string(),
            None,
            None,
        );
        assert!(donation1_result.is_ok());

        let badges = allocator.get_donor_badges("donor1");
        assert_eq!(badges[0].badge_type, "bronze");

        // Test silver badge (donation 100-499)
        let donation2_result = allocator.record_donation(
            "donor2".to_string(),
            "ngo1".to_string(),
            250.0,
            "tx2".to_string(),
            None,
            None,
        );
        assert!(donation2_result.is_ok());

        let badges = allocator.get_donor_badges("donor2");
        assert_eq!(badges[0].badge_type, "silver");

        // Test gold badge (donation 500-999)
        let donation3_result = allocator.record_donation(
            "donor3".to_string(),
            "ngo1".to_string(),
            750.0,
            "tx3".to_string(),
            None,
            None,
        );
        assert!(donation3_result.is_ok());

        let badges = allocator.get_donor_badges("donor3");
        assert_eq!(badges[0].badge_type, "gold");

        // Test platinum badge (donation >= 1000)
        let donation4_result = allocator.record_donation(
            "donor4".to_string(),
            "ngo1".to_string(),
            1500.0,
            "tx4".to_string(),
            None,
            None,
        );
        assert!(donation4_result.is_ok());

        let badges = allocator.get_donor_badges("donor4");
        assert_eq!(badges[0].badge_type, "platinum");
    }
}
