# Quick Start Guide

This guide will help you deploy the P-Project token contracts quickly.

## Prerequisites

1. Install Node.js (v14 or higher)
2. Install npm (comes with Node.js)

## Quick Deployment Steps

### 1. Setup
```bash
# Navigate to the contracts directory
cd d:\p-project\p-project-contracts\src\contracts

# Install dependencies
npm install

# Copy and configure environment file
cp .env.example .env
# Edit .env with your private key and RPC endpoints
```

### 2. Test Deployment
```bash
# Compile contracts
npm run compile

# Test deployment scripts
npm run test:deployment
```

### 3. Deploy to Testnet
```bash
# Deploy contracts to BSC Testnet
npm run deploy:testnet

# Verify deployment
npm run checklist -- --network bscTestnet

# Initialize contracts
npm run initialize -- --network bscTestnet
```

### 4. Deploy to Mainnet
```bash
# Deploy contracts to BSC Mainnet
npm run deploy:mainnet

# Verify contracts on BscScan
npm run verify
```

## Next Steps

After deployment:
1. Set up initial liquidity
2. Submit DEX listing request
3. Announce to community

## Need Help?

- Check the [Deployment Guide](deploy.md) for detailed instructions
- See the [Deployment FAQ](DEPLOYMENT_FAQ.md) for common questions
- Review the [Deployment Summary](DEPLOYMENT_SUMMARY.md) for a quick overview