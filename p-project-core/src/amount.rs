use rust_decimal::prelude::*;
use rust_decimal::Decimal;

/// Utilities for safe monetary math. This module introduces Decimal-based helpers
/// as a first step toward migrating f64 amounts to rust_decimal::Decimal.
/// Callers can convert f64 inputs to Decimal with sane rounding.
pub fn to_decimal(amount: f64) -> Decimal {
    // Convert via string to avoid binary float surprises for common values
    Decimal::from_f64(amount).unwrap_or_else(|| Decimal::ZERO)
}

pub fn to_f64(d: Decimal) -> f64 {
    d.to_f64().unwrap_or(0.0)
}

pub fn round_dp(d: Decimal, dp: u32) -> Decimal {
    d.round_dp(dp)
}
