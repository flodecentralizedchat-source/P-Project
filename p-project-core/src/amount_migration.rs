//! Transitional amount type alias to support migrating from f64 to Decimal.
//! Switch with `--features decimal-amount` during the migration.

#[cfg(feature = "decimal-amount")]
pub type Amount = rust_decimal::Decimal;

#[cfg(not(feature = "decimal-amount"))]
pub type Amount = f64;

