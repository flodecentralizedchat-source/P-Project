//! Lightweight helpers for token price modeling.
//!
//! These utilities capture the starting price and market cap formulas described in
//! the product documents so they can be reused by services and tooling.
use std::fmt;

/// Possible errors when computing price-model outputs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PriceModelError {
    /// Starting price input contained a negative liquidity amount.
    NegativeUsdtLiquidity(f64),
    /// The token-side of the pool must be strictly positive to derive a price.
    NonPositiveTokenLiquidity(f64),
    /// Circulating supply cannot be negative.
    NegativeCirculatingSupply(f64),
    /// Token price cannot be negative.
    NegativeTokenPrice(f64),
}

impl fmt::Display for PriceModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PriceModelError::NegativeUsdtLiquidity(value) => {
                write!(f, "USDT liquidity cannot be negative ({})", value)
            }
            PriceModelError::NonPositiveTokenLiquidity(value) => {
                write!(f, "token liquidity must be positive ({})", value)
            }
            PriceModelError::NegativeCirculatingSupply(value) => {
                write!(f, "circulating supply cannot be negative ({})", value)
            }
            PriceModelError::NegativeTokenPrice(value) => {
                write!(f, "token price cannot be negative ({})", value)
            }
        }
    }
}

impl std::error::Error for PriceModelError {}

/// Starting price formula derived from the LP pair.
///
/// ```
/// Starting Price = USDT Liquidity / Token Liquidity
/// ```
pub fn starting_price(usdt_liquidity: f64, token_liquidity: f64) -> Result<f64, PriceModelError> {
    if usdt_liquidity < 0.0 {
        return Err(PriceModelError::NegativeUsdtLiquidity(usdt_liquidity));
    }

    if token_liquidity <= 0.0 {
        return Err(PriceModelError::NonPositiveTokenLiquidity(token_liquidity));
    }

    Ok(usdt_liquidity / token_liquidity)
}

/// Market cap formula driven by circulating supply.
///
/// ```
/// Market Cap = Circulating Supply × Token Price
/// ```
pub fn market_cap(circulating_supply: f64, token_price: f64) -> Result<f64, PriceModelError> {
    if circulating_supply < 0.0 {
        return Err(PriceModelError::NegativeCirculatingSupply(
            circulating_supply,
        ));
    }

    if token_price < 0.0 {
        return Err(PriceModelError::NegativeTokenPrice(token_price));
    }

    Ok(circulating_supply * token_price)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_price_formula_example() {
        let price = starting_price(50_000.0, 5_000_000.0).expect("valid inputs");
        assert_eq!(price, 0.01);
    }

    #[test]
    fn starting_price_rejects_zero_token_liquidity() {
        let err = starting_price(10_000.0, 0.0).unwrap_err();
        assert_eq!(err, PriceModelError::NonPositiveTokenLiquidity(0.0));
    }

    #[test]
    fn market_cap_formula_example() {
        let market_cap = market_cap(70_000_000.0, 0.20).expect("valid inputs");
        assert_eq!(market_cap, 14_000_000.0);
    }

    #[test]
    fn market_cap_rejects_negative_price() {
        let err = market_cap(1_000_000.0, -0.5).unwrap_err();
        assert_eq!(err, PriceModelError::NegativeTokenPrice(-0.5));
    }
}
