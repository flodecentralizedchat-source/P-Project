# EVM vs Non‑EVM Chain Targets

## Counts (by individual networks)
- EVM networks: 5 (Ethereum, Polygon, BSC, Arbitrum, Optimism)
- Non‑EVM networks: 2 (Solana, Sui)

Note: The EVM list is configurable via `EVM_NETWORKS`. The above are the default targets we plan to support/document.

## Breakdown
- EVM
  - Ethereum — Solidity contracts; Bridge `p-project-bridge/contracts/Bridge.sol`
  - Polygon — uses same Solidity/ABI and relayer as Ethereum
  - BSC — uses same Solidity/ABI and relayer as Ethereum
  - Arbitrum — uses same Solidity/ABI and relayer as Ethereum
  - Optimism — uses same Solidity/ABI and relayer as Ethereum
    - Contracts path (shared): `p-project-contracts/src/contracts/*.sol`, `p-project-bridge/contracts/Bridge.sol`
- Non‑EVM
  - Solana — adapter: `p-project-bridge/src/solana.rs`
  - Sui — adapter: `p-project-bridge/src/sui.rs`

## Configuration

Set `EVM_NETWORKS` to a comma‑separated list of EVM network names (case‑insensitive, no spaces):
- Example: `EVM_NETWORKS=ethereum,polygon,bsc,arbitrum,optimism`

For each name `<NAME>`, set:
- `EVM_<NAME>_RPC_URL`
- `EVM_<NAME>_BRIDGE_ADDRESS`
- `EVM_<NAME>_TOKEN_ADDRESS`
- `EVM_<NAME>_PRIVATE_KEY_ENV` (optional; defaults to `ETH_PRIVATE_KEY`)
- `EVM_<NAME>_CONFIRMATIONS` (optional; defaults per chain/policy)

Concrete examples:
- Ethereum: `EVM_ETHEREUM_RPC_URL`, `EVM_ETHEREUM_BRIDGE_ADDRESS`, `EVM_ETHEREUM_TOKEN_ADDRESS`
- Polygon: `EVM_POLYGON_RPC_URL`, `EVM_POLYGON_BRIDGE_ADDRESS`, `EVM_POLYGON_TOKEN_ADDRESS`
- BSC: `EVM_BSC_RPC_URL`, `EVM_BSC_BRIDGE_ADDRESS`, `EVM_BSC_TOKEN_ADDRESS`
- Arbitrum: `EVM_ARBITRUM_RPC_URL`, `EVM_ARBITRUM_BRIDGE_ADDRESS`, `EVM_ARBITRUM_TOKEN_ADDRESS`
- Optimism: `EVM_OPTIMISM_RPC_URL`, `EVM_OPTIMISM_BRIDGE_ADDRESS`, `EVM_OPTIMISM_TOKEN_ADDRESS`

Backward‑compatibility:
- If `EVM_NETWORKS` is not set, the bridge will read legacy single‑network vars: `ETH_RPC_URL`, `ETH_BRIDGE_ADDRESS`, `ETH_TOKEN_ADDRESS`, `ETH_PRIVATE_KEY_ENV`, `ETH_CONFIRMATIONS`.

## Source
- `p-docs/multi-chain-deployment-features.md` — defines deployment across Ethereum (EVM), Solana, and Sui.

## Notes
- The EVM relayer/adapter logic is shared across EVM networks using the same ABI. Add new EVM networks by adding them to `EVM_NETWORKS` and setting the corresponding environment variables.
