# Tokenomics & Price Model

The repository ships `tokenomics_master_price.csv`, which records every major metric:

- **TOKENOMICS section** – supply, circulating amounts, and allocations for the team, ecosystem, treasury, staking rewards, etc.
- **VESTING section** – durations, cliffs, and emission patterns for team/advisors/investors.
- **LP_RATIOS and LAUNCH_STRATEGY** – initial liquidity levels, DEX/CEX plans, presale percentages, and lock durations.
- **PRICE_MODEL** – aspirational target prices plus the conditions required (market cap, staking, burns, exchanges).

The API exposes `/tokenomics/summary` (public). It parses the CSV into structured JSON, so you can:

1. Display token distribution data in a dashboard.
2. Show the launch strategy checklist next to the token release schedule.
3. Feed the price targets into visualizations (`market_cap` is calculated automatically from the total supply).

Set `TOKENOMICS_CSV_PATH` to point at a different CSV if you maintain updated variants.
