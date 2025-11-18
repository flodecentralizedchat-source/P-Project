# Getting Started with P-Project Token Deployment

This guide will help you deploy the P-Project token contracts to a blockchain network.

## Prerequisites

1. Install Node.js (v14 or higher)
2. Install npm (comes with Node.js)
3. Have a wallet with sufficient funds for deployment

## Installation

1. Navigate to the contracts directory:
   ```bash
   cd d:\p-project\p-project-contracts\src\contracts
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

## Configuration

1. Create a `.env` file in the contracts directory with your configuration:
   ```env
   PRIVATE_KEY=your_private_key_here
   BSC_TESTNET_URL=https://data-seed-prebsc-1-s1.binance.org:8545
   BSC_MAINNET_URL=https://bsc-dataseed.binance.org/
   ```

## Deployment Process

1. Compile the contracts:
   ```bash
   npx hardhat compile
   ```

2. Deploy to testnet:
   ```bash
   npx hardhat run deploy.js --network bscTestnet
   ```

3. Verify deployment:
   ```bash
   npx hardhat run scripts/deployment-checklist.js --network bscTestnet
   ```

4. Initialize contracts:
   ```bash
   npx hardhat run scripts/initialize-contracts.js --network bscTestnet
   ```

5. Prepare for DEX listing:
   ```bash
   npx hardhat run scripts/prepare-dex-listing.js --network bscTestnet
   ```

## Next Steps

After successful testnet deployment:
1. Audit your contracts
2. Deploy to mainnet
3. Verify contracts on block explorer
4. Set up initial liquidity
5. Submit DEX listing request