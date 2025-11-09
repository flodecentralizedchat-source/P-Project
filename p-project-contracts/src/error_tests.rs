//! Comprehensive tests for custom error types in all contracts

#[cfg(test)]
mod tests {
    use crate::airdrop::AirdropError;
    use crate::staking::StakingError;
    use crate::token::TokenError;
    use std::error::Error;

    #[test]
    fn test_token_error_display() {
        let error = TokenError::InsufficientBalance;
        assert_eq!(format!("{}", error), "Insufficient balance");

        let error = TokenError::InsufficientFrozenBalance;
        assert_eq!(format!("{}", error), "Insufficient frozen balance");

        let error = TokenError::TransferLimitExceeded(100.0);
        assert_eq!(
            format!("{}", error),
            "Transfer amount exceeds maximum limit of 100"
        );

        let error = TokenError::InvalidAmount;
        assert_eq!(format!("{}", error), "Amount must be positive");

        let error = TokenError::DatabaseError("Connection failed".to_string());
        assert_eq!(format!("{}", error), "Database error: Connection failed");

        let error = TokenError::SerializationError("JSON parse error".to_string());
        assert_eq!(
            format!("{}", error),
            "Serialization error: JSON parse error"
        );
    }

    #[test]
    fn test_token_error_traits() {
        let error = TokenError::InsufficientBalance;
        assert!(error.source().is_none());
        assert_eq!(format!("{:?}", error), "InsufficientBalance");
    }

    #[test]
    fn test_staking_error_display() {
        let error = StakingError::InvalidAmount;
        assert_eq!(format!("{}", error), "Amount must be positive");

        let error = StakingError::NoStakingInfo;
        assert_eq!(format!("{}", error), "No staking info found for user");

        let error = StakingError::StakingPositionExists;
        assert_eq!(
            format!("{}", error),
            "Target user already has a staking position"
        );

        let error = StakingError::EmergencyWithdrawalsDisabled;
        assert_eq!(
            format!("{}", error),
            "Emergency withdrawals are currently disabled"
        );

        let error = StakingError::DatabaseError("Connection failed".to_string());
        assert_eq!(format!("{}", error), "Database error: Connection failed");

        let error = StakingError::SerializationError("JSON parse error".to_string());
        assert_eq!(
            format!("{}", error),
            "Serialization error: JSON parse error"
        );
    }

    #[test]
    fn test_staking_error_traits() {
        let error = StakingError::InvalidAmount;
        assert!(error.source().is_none());
        assert_eq!(format!("{:?}", error), "InvalidAmount");
    }

    #[test]
    fn test_airdrop_error_display() {
        let error = AirdropError::InsufficientTokens;
        assert_eq!(format!("{}", error), "Not enough tokens for airdrop");

        let error = AirdropError::UserNotEligible;
        assert_eq!(format!("{}", error), "User not eligible for airdrop");

        let error = AirdropError::AirdropAlreadyClaimed;
        assert_eq!(format!("{}", error), "Airdrop already claimed");

        let error = AirdropError::InvalidMerkleProof;
        assert_eq!(format!("{}", error), "Invalid merkle proof");

        let error = AirdropError::InvalidSignature;
        assert_eq!(format!("{}", error), "Invalid signature");

        let error = AirdropError::AirdropNotActive;
        assert_eq!(format!("{}", error), "Airdrop is not currently active");

        let error = AirdropError::EmergencyWithdrawalsDisabled;
        assert_eq!(
            format!("{}", error),
            "Emergency withdrawals are currently disabled"
        );

        let error = AirdropError::DatabaseError("Connection failed".to_string());
        assert_eq!(format!("{}", error), "Database error: Connection failed");

        let error = AirdropError::SerializationError("JSON parse error".to_string());
        assert_eq!(
            format!("{}", error),
            "Serialization error: JSON parse error"
        );
    }

    #[test]
    fn test_airdrop_error_traits() {
        let error = AirdropError::InvalidMerkleProof;
        assert!(error.source().is_none());
        assert_eq!(format!("{:?}", error), "InvalidMerkleProof");
    }

    #[test]
    fn test_error_comparisons() {
        let token_error1 = TokenError::InsufficientBalance;
        let token_error2 = TokenError::InsufficientBalance;
        let token_error3 = TokenError::InvalidAmount;

        assert_eq!(token_error1, token_error2);
        assert_ne!(token_error1, token_error3);

        let staking_error1 = StakingError::InvalidAmount;
        let staking_error2 = StakingError::InvalidAmount;
        let staking_error3 = StakingError::NoStakingInfo;

        assert_eq!(staking_error1, staking_error2);
        assert_ne!(staking_error1, staking_error3);

        let airdrop_error1 = AirdropError::InvalidMerkleProof;
        let airdrop_error2 = AirdropError::InvalidMerkleProof;
        let airdrop_error3 = AirdropError::UserNotEligible;

        assert_eq!(airdrop_error1, airdrop_error2);
        assert_ne!(airdrop_error1, airdrop_error3);
    }
}
