# P-Project Multi-Chain Deployment Guide

This guide covers the step-by-step process for deploying the P-Project token and supporting contracts to the primary chains in scope: Ethereum (EVM), Solana (SPL), and Sui (Move). Each section assumes the Solidity/Rust work is already compiled (see `cargo fmt`/`cargo test` for Rust, and `npx hardhat compile` for EVM artifacts).

## Prerequisites

1. **Environment**  
   - Install Rust toolchain (stable) and run `cargo test` in the workspace to ensure the off-chain services build.
   - Install Node.js (>=18) for Hardhat scripts (`package.json` already declares dependencies).
   - Install `solana` CLI and `sui` CLI for SPL / Move deployments.

2. **Secrets/config**  
   - Provide RPC endpoints as environment variables (e.g., `ETHEREUM_RPC_URL`, `SOLANA_RPC_URL`, `SUI_RPC_URL`).
   - Store deployer private keys securely (e.g., `.env` with `DEPLOYER_PRIVATE_KEY` for Ethereum and corresponding keypairs for Solana/Sui).
   - Maintain the DAO multisig addresses for owner roles and bridging authorities.

3. **Build outputs**
   - Hardhat artifacts: `npx hardhat compile`.
   - Solidity tests: `npx hardhat test`.
   - Rust services: `cargo test -p p-project-core marketing` (run locally outside sandbox).

## 1. EVM Deployment (Ethereum)

1. **Compile & Test**
   - Run `npx hardhat compile` from the contracts directory (`p-project-contracts` or root if scripts/config there).
   - Execute `npx hardhat test` to validate contracts (`PProjectToken.sol`, `Vesting.sol`, `Treasury.sol`, `LiquidityPool.sol`).

2. **Configure network**
   - Create `.env` or export:
     ```
     ETHEREUM_RPC_URL=https://eth-mainnet.g.alchemy.com/v2/<key>
     DEPLOYER_PRIVATE_KEY=0xabc...
     ETHERSCAN_API_KEY=XXXX
     ```
   - Ensure `hardhat.config.js` references these env vars and that `network` entries exist for `mainnet`, `base`, `polygon` if desired.

3. **Run deployment script**
   - Execute `npx hardhat run scripts/deploy.js --network mainnet`.
   - Confirm the script deploys the five essential contracts and prints their addresses. Capture those addresses for the cross-chain bridge config.

4. **Verify on explorer**
   - Verify each contract on Etherscan via `npx hardhat verify --network mainnet <address>`.
   - For LiquidityPool/Treasury, ensure constructor arguments are recorded.

5. **Funding liquidity**
   - Mint/transfer tokens to the owner multisig.
   - Approve router/pair contract and add liquidity (e.g., `ethers.js` script or DeFi UI).
   - Lock liquidity for 24 months using the mechanisms in `PProjectToken.sol`.

6. **Bridge prep**
   - Use `Bridge.sol` to lock the initial Ethereum supply that will back Solana/Sui markets.
   - Record the bridge contract address for the SPL/Move wrappers.

7. **Monitoring**
   - Set up notifications for the DAO multisig and key events (burns, liquidity changes, treasury actions) via Etherscan alerts or Dune dashboards.

## 2. Solana Deployment (SPL)

1. **Mint a wrapped SPL token**
   - Use the `solana` CLI: `solana-keygen new` for the bridge authority, then `spl-token create-token --decimals 18`.
   - Create an SPL account with `spl-token create-account <mint>`.
   - Mint supply equal to the locked Ethereum tokens through the bridge authority: `spl-token mint <mint> <amount> <account>`.

2. **Bridge synchronization**
   - Use `Bridge.sol` or the p-project-bridge service to lock a matching amount on Ethereum and emit the `Mint` event consumed by your Solana relayer.
   - Configure your relayer (e.g., Wormhole, Axelar) to listen for the lock event and trigger SPL minting.
   - Store metadata (name: PProject, symbol: P) in the Solana mint info.

3. **Deployment of SPL utilities**
   - If you expose wrapped token functionality (staking/treasury), deploy Anchor or Solana programs that reference the mint.
   - Ensure the on-chain token metadata is verified and the mint authority matches the bridge.

4. **Listing on Solana DEX**
   - Provide liquidity on Serum/Raydium by supplying the SPL token paired with USDC/USDT.
   - Optionally, integrate with Jupiter for aggregated swaps.

## 3. Sui Deployment (Move)

1. **Design the Move module**
   - Create a Move package (e.g., `p_project_coin`) with a `struct PProject` that implements the `Coin` interface.
   - Include capabilities for mint/burn controlled by the Sui bridge guardian or a permissioned account.
   - Reference the locked Ethereum bridge supply (the same total minted + SPL counterpart should not exceed Ethereum supply).

2. **Compile & Publish**
   - Use `sui move build` to compile the package.
   - Publish to the desired Sui network with `sui client publish --gas-budget 10000000 --gas-budget <budget>`.

3. **Bridge Integration**
   - Connect the Sui move mint authority to the cross-chain relayer that listens to `Bridge.sol` events (lock/mint, burn/release).
   - Implement verification logic to ensure each mint call corresponds to a validated Ethereum lock.

4. **Liquidity and listing**
   - Pair `PProject` with Sui-native stable assets (ETH-native or USDC analog) on Sui AMMs (e.g., Suiswap).
   - Provide initial liquidity and register the pairing metadata with the Sui DEX.

## 4. Ongoing Synchronization & Security

1. **Bridge checks**
   - Maintain watch scripts to ensure total minted tokens (EVM + SPL + Move) never exceed the Ethereum locked supply.
   - Automate reconciliation via dashboards or on-chain proofs.

2. **Governance triggers**
   - Use the DAO multisig (owner) to trigger scheduled burns, buybacks, and liquidity unlocks as outlined in the `PProjectToken.sol`.
   - Ensure each action is logged and verified in the monitoring stack (Dune, Flipside, explorers).

3. **Audits**
   - Before each chain launch, run `solc`/`move` auditing, and keep audit reports updated in the docs.

## 5. Bridge Relayer & Operations Checklist

1. **Bridge relayer wiring**
   - Implement a relayer that listens to the `Bridge.sol` lock/burn events and triggers the corresponding SPL `mint` or Sui `mint/burn` entrypoints.
   - Document the commands used by these scripts (e.g., `node scripts/relay-to-spl.js --lock-event <tx>` and `node scripts/relay-to-sui.js --mint <amount>`).
   - Log and monitor relayer activity so mint/burn counts always balance with the Ethereum locked supply before releasing tokens to user endpoints.

2. **Deployment checklists**
   - **Ethereum (EVM)**
     - [ ] `npx hardhat compile`
     - [ ] `npx hardhat test`
     - [ ] `npx hardhat run scripts/deploy.js --network mainnet`
     - [ ] `npx hardhat verify --network mainnet <contract>`
     - [ ] `node scripts/fund-liquidity.js --amount <eth> --pair P/ETH`
   - **Solana (SPL)**
     - [ ] `spl-token create-token --decimals 18`
     - [ ] `spl-token create-account <mint>`
     - [ ] `node scripts/relay-to-spl.js --lock-tx <tx>`
     - [ ] `node scripts/setup-serum-market.js --mint <mint>`
   - **Sui (Move)**
     - [ ] `sui move build`
     - [ ] `sui client publish --gas-budget 10000000`
     - [ ] `node scripts/relay-to-sui.js --lock-tx <tx>`
     - [ ] `node scripts/list-sui-pair.js --coin <PProject>`

3. **Next steps**
   - Wire the relayer logic (events → SPL/Move mint calls) into production scripts and document each command.
   - Keep the checklists in sync with your automation so operators can follow them securely every release cycle.
