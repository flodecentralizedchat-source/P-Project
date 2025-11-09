//! Tests for airdrop contract with database integration

#[cfg(test)]
mod tests {
    use crate::airdrop::{AirdropContract, AirdropError};
    use chrono::Utc;

    #[test]
    fn test_airdrop_creation() {
        let airdrop = AirdropContract::new(1000000.0);
        assert_eq!(airdrop.get_status().total_amount, 1000000.0);
        assert!(!airdrop.is_paused());
    }

    #[test]
    fn test_add_recipients_success() {
        let mut airdrop = AirdropContract::new(1000000.0);
        let recipients = vec![("user1".to_string(), 1000.0), ("user2".to_string(), 500.0)];
        
        let result = airdrop.add_recipients(recipients);
        assert!(result.is_ok());
        
        let status = airdrop.get_status();
        assert_eq!(status.distributed_amount, 1500.0);
    }

    #[test]
    fn test_add_recipients_insufficient_tokens() {
        let mut airdrop = AirdropContract::new(1000.0);
        let recipients = vec![("user1".to_string(), 10000.0)];
        
        let result = airdrop.add_recipients(recipients);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AirdropError::InsufficientTokens);
    }

    #[test]
    fn test_claim_airdrop_success() {
        let mut airdrop = AirdropContract::new(1000000.0);
        let recipients = vec![("user1".to_string(), 1000.0)];
        airdrop.add_recipients(recipients).unwrap();
        
        let result = airdrop.claim("user1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1000.0);
        assert!(airdrop.is_claimed("user1"));
    }

    #[test]
    fn test_claim_airdrop_not_eligible() {
        let mut airdrop = AirdropContract::new(1000000.0);
        
        let result = airdrop.claim("user1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AirdropError::UserNotEligible);
    }

    #[test]
    fn test_claim_airdrop_already_claimed() {
        let mut airdrop = AirdropContract::new(1000000.0);
        let recipients = vec![("user1".to_string(), 1000.0)];
        airdrop.add_recipients(recipients).unwrap();
        airdrop.claim("user1").unwrap();
        
        let result = airdrop.claim("user1");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AirdropError::AirdropAlreadyClaimed);
    }

    #[test]
    fn test_pause_resume_airdrop() {
        let mut airdrop = AirdropContract::new(1000000.0);
        assert!(!airdrop.is_paused());
        
        airdrop.pause();
        assert!(airdrop.is_paused());
        
        airdrop.resume();
        assert!(!airdrop.is_paused());
    }

    #[test]
    fn test_timed_airdrop() {
        let start_time = Utc::now().naive_utc();
        let end_time = start_time + chrono::Duration::days(1);
        let airdrop = AirdropContract::new_timed(1000000.0, start_time, end_time);
        
        assert!(airdrop.get_start_time().is_some());
        assert!(airdrop.get_end_time().is_some());
    }

    #[test]
    fn test_batch_claim() {
        let mut airdrop = AirdropContract::new(1000000.0);
        let recipients = vec![
            ("user1".to_string(), 1000.0),
            ("user2".to_string(), 500.0),
            ("user3".to_string(), 250.0)
        ];
        airdrop.add_recipients(recipients).unwrap();
        
        let user_ids = vec!["user1".to_string(), "user2".to_string()];
        let result = airdrop.batch_claim(user_ids);
        assert!(result.is_ok());
        
        let claimed = result.unwrap();
        assert_eq!(claimed.len(), 2);
        assert_eq!(claimed[0].0, "user1");
        assert_eq!(claimed[0].1, 1000.0);
        assert_eq!(claimed[1].0, "user2");
        assert_eq!(claimed[1].1, 500.0);
    }

    #[test]
    fn test_referral_bonus() {
        let mut airdrop = AirdropContract::new(1000000.0);
        let recipients = vec![("user1".to_string(), 1000.0)];
        airdrop.add_recipients(recipients).unwrap();
        airdrop.add_referral("user1", "referrer1".to_string());
        
        let result = airdrop.claim("user1");
        assert!(result.is_ok());
        
        // User should get 1000 + 50 (5% referral bonus)
        assert_eq!(result.unwrap(), 1050.0);
    }
}