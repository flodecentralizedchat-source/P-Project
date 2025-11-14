#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;
    use p_project_web::*;

    #[wasm_bindgen_test]
    fn test_proposal_creation() {
        let proposal = create_proposal(
            "Test Title",
            "Test Description",
            "creator123",
        );
        
        assert_eq!(proposal.title(), "Test Title");
        assert_eq!(proposal.description(), "Test Description");
        assert_eq!(proposal.creator_id(), "creator123");
        assert_eq!(proposal.status(), "Active");
    }

    #[wasm_bindgen_test]
    fn test_voting() {
        let result = vote_on_proposal("proposal123", "user456", true);
        assert_eq!(result, true);
    }

    #[wasm_bindgen_test]
    fn test_delegation() {
        let result = delegate_vote("user1", "user2");
        assert_eq!(result, true);
    }

    #[wasm_bindgen_test]
    fn test_staking_yield_calculation() {
        let result = calculate_staking_yield(1000.0, 365);
        
        assert_eq!(result.amount(), 1000.0);
        assert_eq!(result.duration_days(), 365);
        assert_eq!(result.apy_rate(), 0.1); // 10% APY
        assert!(result.projected_rewards() > 0.0);
        assert!(result.total_return() > result.amount());
    }

    #[wasm_bindgen_test]
    fn test_airdrop_status() {
        let status = get_airdrop_status();
        
        assert_eq!(status.airdrop_id(), "airdrop-1");
        assert_eq!(status.total_amount(), 1000000.0);
        assert_eq!(status.distributed_amount(), 250000.0);
        assert_eq!(status.total_recipients(), 10000);
        assert_eq!(status.claimed_recipients(), 2500);
        assert_eq!(status.is_paused(), false);
        assert_eq!(status.progress_percentage(), 25.0);
    }

    #[wasm_bindgen_test]
    fn test_user_creation() {
        let user = WebUser::new(
            "user123".to_string(),
            "testuser".to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
        );
        
        assert_eq!(user.id(), "user123");
        assert_eq!(user.username(), "testuser");
        assert_eq!(user.wallet_address(), "0x1234567890123456789012345678901234567890");
        assert_eq!(user.short_wallet_address(), "0x1234...7890");
    }
}