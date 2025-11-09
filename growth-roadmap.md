# P‑Coin Growth Roadmap & KPIs

## Phases

### Phase 0 — Proof & Trust (Weeks 0–2)
- Publish audits or open-source test suite
- Lock LP for 24 months; doxx multisig signers
- Launch Impact Dashboard (airdrops, donations, NGO receipts)
**KPIs:** 2,000 holders • $500k LP lock • 2 NGOs onboarded

### Phase 1 — Ignition (Months 1–2)
- Airdrop claim + sybil-resistant quests (Zealy/Galxe)
- Campus & dev-guild partnerships; “Tip in Peace” widgets
**KPIs:** 10,000 holders • 50 merchants accept $P • 10M P staked

### Phase 2 — Utility Flywheel (Months 3–6)
- Ship Staking, Tip‑bot (Telegram/Discord), Donation widgets
- Peace Art NFT drops (royalties → charity)
**KPIs:** $1M cumulative donations • 100 NGOs • 100k MAW

### Phase 3 — Scale & Governance (Months 7–12)
- CEX listings, L2 bridge, matching‑donation programs
- Quarterly Quadratic Grants for peace builders
**KPIs:** 500 NGOs • $5M donations • 300k MAW • 40% staking ratio

---

## Always‑on Growth Loops
- **Give → Share → Earn:** Donation mints Proof‑of‑Peace badge (soulbound/NFT). Share to join raffles in $P.
- **Merchant Cashback:** Merchants accepting $P receive capped cashback in $P.
- **Creator Tips:** “Tip in Peace” buttons on blogs/streams; tiny on‑chain fees.
- **Referral (capped):** On‑chain ref codes with sybil checks; bonus <10%.

## Transparency & Reporting
- Monthly on‑chain treasury and allocator reports (Dune/Flipside dashboards)
- NGO receipts & impact docs pinned to IPFS/Arweave with hashes
- Public proposal history via PGovernor + Timelock

## Practical Parameters
- **Tx Tax (optional):** 2% — 1% CharityAllocator, 1% burn (DAO can disable)
- **Staking APR Target:** 15–25% year 1, declining via halvings
- **DAO Quorum/Threshold:** Quorum 4%; proposal threshold 0.1%; timelock 72h
- **Claim Gas Target:** L2 (Base/Polygon) to keep claim < $0.10

## Security & Compliance Guardrails
- Multisig + Timelock on all treasury/allocator actions
- Emergency pause on airdrop/allocator
- NGO KYC/verification off‑chain; on‑chain NGO registry (address + metadata hash)
- Unit/property/invariant tests; public audits when available
- No admin keys for token after setup (or DAO‑controlled only)

## Integration Checklists

### Smart Contracts
- PToken (ERC‑20) • PGovernor + Timelock • MultiSig Treasury
- CharityAllocator • AirdropDistributor (Merkle) • VestingVault
- StakingRewards • LiquidityManager • BridgeAdapter (optional)

### DApp
- WalletConnect + Ethers.js hooks
- Pages: Home • Donate • Stake • Airdrop Claim • DAO • Grants
- Indexing: The Graph (transfers, votes, claims, allocations, staking)

### Payments
- Crypto gateway (NowPayments/Coinbase Commerce)
- QR/PoS flows for merchants
- Web2 donation portal: donate.peacecoin.org

---

*P‑Coin — Spend Peacefully.*