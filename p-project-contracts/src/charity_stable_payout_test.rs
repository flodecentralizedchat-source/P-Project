#[cfg(test)]
mod tests {
    use super::super::charity::{CharityAllocator, CharityError};
    use super::super::stable_liquidity_pool::StableLiquidityPool;

    #[test]
    fn test_ngo_payout_via_stable_pool() {
        let dao = "dao123".to_string();
        let mut allocator = CharityAllocator::new(dao.clone(), 1_000_000.0);

        // Register and verify NGO
        allocator
            .register_ngo(
                &dao,
                "ngo_addr".to_string(),
                "Good NGO".to_string(),
                "Helping people".to_string(),
                "relief".to_string(),
                None,
                None,
            )
            .unwrap();
        allocator.verify_ngo(&dao, "ngo_addr").unwrap();

        // Create stable pool P-COIN/USDC and add liquidity
        let mut sp = StableLiquidityPool::new(
            "pcoin_usdc_stable".to_string(),
            "P-COIN".to_string(),
            "USDC".to_string(),
            0.0005,
            100.0,
            "REWARD".to_string(),
            100000.0,
            0.10,
        )
        .unwrap();
        sp.add_liquidity("lp1".to_string(), 500_000.0, 500_000.0, 90)
            .unwrap();

        // Create allocation for NGO in P-COIN terms
        let alloc_id = allocator
            .allocate(
                &dao,
                "ngo_addr".to_string(),
                100_000.0,
                "Disaster relief tranche".to_string(),
            )
            .unwrap();

        let (reserves_a_before, reserves_b_before) = sp.get_reserves();

        // Disburse using stable pool (P-COIN -> USDC)
        let (tx_hash, usdc_out) = allocator
            .disburse_with_stable_pool(&dao, &alloc_id, &mut sp)
            .unwrap();

        assert!(!tx_hash.is_empty());
        assert!(usdc_out > 0.0);

        // Fund balance reduced by P-COIN allocation amount
        assert_eq!(allocator.get_fund_balance(), 900_000.0);

        // Reserves updated as expected
        let (ra, rb) = sp.get_reserves();
        assert!(ra > reserves_a_before); // P-COIN increased in pool
        assert!(rb < reserves_b_before); // USDC decreased in pool

        // Second disburse should fail (already disbursed)
        let err = allocator
            .disburse_with_stable_pool(&dao, &alloc_id, &mut sp)
            .unwrap_err();
        assert_eq!(err, CharityError::AllocationAlreadyDisbursed);
    }
}
