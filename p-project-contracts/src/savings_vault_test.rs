#[cfg(test)]
mod tests {
    use super::super::charity::CharityAllocator;
    use super::super::savings_vault::{SavingsError, SavingsVault};
    use chrono::Duration;

    #[test]
    fn test_savings_vault_interest_and_charity_share() {
        let mut vault = SavingsVault::new(
            "interest_vault".to_string(),
            0.25, // 25% APR
            0.15, // 15% of interest goes to charities
            30,
            5000.0,
        )
        .unwrap();

        vault.deposit("user1".to_string(), 2000.0, 30).unwrap();
        vault
            .fast_forward_position("user1", Duration::days(31))
            .unwrap();

        let (principal, user_interest, charity_share) = vault.withdraw("user1").unwrap();
        assert_eq!(principal, 2000.0);

        let expected_interest = vault.calculate_interest(2000.0, 30.0);
        assert!((user_interest - expected_interest * 0.85).abs() < 1e-6);
        assert!((charity_share - expected_interest * 0.15).abs() < 1e-6);
        assert!((vault.charity_yield_pool - charity_share).abs() < 1e-8);

        let mut allocator = CharityAllocator::new("dao".to_string(), 0.0);
        let distributed = vault.claim_charity_yield(&mut allocator).unwrap();
        assert!((distributed - charity_share).abs() < 1e-8);
        assert!((allocator.get_fund_balance() - charity_share).abs() < 1e-8);
    }

    #[test]
    fn test_fast_forward_position_fails_for_unknown_user() {
        let mut vault =
            SavingsVault::new("interest_vault".to_string(), 0.1, 0.1, 7, 1000.0).unwrap();

        let err = vault
            .fast_forward_position("missing", Duration::days(1))
            .unwrap_err();
        assert_eq!(err, SavingsError::PositionNotFound);
    }
}
