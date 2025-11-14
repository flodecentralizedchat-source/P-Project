Goal: migrate monetary amounts from f64 to rust_decimal::Decimal for correctness and safety.

Scope and approach
- Phase 1 (done): API edge accepts/returns Decimal in JSON for amounts; converts to f64 with rounding when calling DB/core using existing f64 types.
- Phase 2: Introduce a central Amount alias in core and refactor models/functions to use it.
- Phase 3: Update DB schemas/queries to ensure DECIMAL columns are used consistently and enforce scale/precision.

Compatibility plan
- Use an Amount type alias to ease refactors:
  - Default: type Amount = f64
  - Feature `decimal-amount`: type Amount = rust_decimal::Decimal
- Gradually replace direct f64 in core with `amount_migration::Amount`.

Rounding policy
- Use 8 decimal places when converting Decimal→f64 at the API boundary for now.
- Final policy should be set per domain (token vs fiat) and enforced consistently in DB and business logic.

Next steps
1) Replace f64 fields in core models (StakingInfo, TokenTransaction, BridgeTx, etc.) with `amount_migration::Amount`.
2) Update DB read/write code to use DECIMAL consistently and avoid implicit float conversions.
3) Expand tests to cover corner cases (very small/large amounts, rounding edges).

