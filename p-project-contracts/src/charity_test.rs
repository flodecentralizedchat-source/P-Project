#[cfg(test)]
mod tests {
    use super::super::charity::{CharityAllocator, CharityError};
    use super::super::token::PProjectToken;

    #[test]
    fn test_charity_allocator_creation() {
        let dao_address = "dao123".to_string();
        let initial_balance = 1000000.0;
        let allocator = CharityAllocator::new(dao_address.clone(), initial_balance);

        assert_eq!(allocator.get_dao_address(), &dao_address);
        assert_eq!(allocator.get_fund_balance(), initial_balance);
    }

    #[test]
    fn test_add_funds() {
        let mut allocator = CharityAllocator::new("dao123".to_string(), 1000000.0);
        let initial_balance = allocator.get_fund_balance();

        // Add funds successfully
        let result = allocator.add_funds(500000.0);
        assert!(result.is_ok());
        assert_eq!(allocator.get_fund_balance(), initial_balance + 500000.0);

        // Try to add negative amount
        let result = allocator.add_funds(-1000.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CharityError::InvalidAmount);
    }
}