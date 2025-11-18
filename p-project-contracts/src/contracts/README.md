# P-Project Smart Contracts

This directory contains the Solidity smart contracts for the P-Project token ecosystem, designed for deployment on EVM-compatible blockchains.

## Contract Overview

### 1. PProjectToken.sol
The main ERC-20 token contract with comprehensive deflationary mechanisms:
- Dynamic burn rates based on network activity
- Transaction-based burns
- Scheduled burns
- Milestone-based burns
- Revenue-linked burns
- Anti-whale protections
- Holder reward distribution

### 2. Vesting.sol
Vesting contract for team, advisor, and investor allocations:
- Linear vesting with configurable cliff periods
- Support for different vesting schedules
- Beneficiary-specific release functionality

### 3. Treasury.sol
Treasury management contract with buyback capabilities:
- Multi-asset fund management
- Treasury allocations
- Buyback execution
- Scheduled and trigger-based buybacks

### 4. LiquidityPool.sol
Liquidity pool contract for DEX integration:
- Constant product formula (x * y = k)
- Liquidity provision and removal
- Swap functionality with fee collection
- Liquidity locking mechanisms

### 5. deploy.js
Deployment script for all contracts using Hardhat:
- Automated deployment of all contracts
- Contract address tracking
- Initial setup and configuration

## Deployment Instructions

For detailed deployment instructions, see [GETTING_STARTED.md](GETTING_STARTED.md) and [deploy.md](../../../deploy.md).

### Quick Start

1. Install dependencies:
```bash
npm install
```

2. Compile contracts:
```bash
npx hardhat compile
```

3. Deploy to testnet:
```bash
npx hardhat run deploy.js --network bscTestnet
```

4. Verify deployment:
```bash
npx hardhat run scripts/deployment-checklist.js --network bscTestnet
```

5. Run tests:
```bash
npx hardhat test
```

## Contract Addresses

After deployment, contract addresses will be saved to `deployment-info.json`.

## Security

These contracts implement standard security practices:
- Access control modifiers
- Reentrancy protection
- Integer overflow/underflow protection
- Proper error handling

## Auditing

Before mainnet deployment, these contracts should be audited by a professional security firm.