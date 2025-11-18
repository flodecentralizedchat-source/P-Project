# P-Project Token Deployment FAQ

## General Questions

### What are the prerequisites for deploying the P-Project contracts?
You need:
- Node.js v14 or higher
- npm (comes with Node.js)
- A wallet with sufficient funds for deployment
- RPC endpoint access for your target network
- Private key for the deployment wallet

### Which networks are supported?
The contracts are designed for EVM-compatible networks:
- Binance Smart Chain (BSC)
- Ethereum
- Polygon
- Other EVM-compatible chains

### How much does deployment cost?
Deployment costs vary by network:
- BSC Testnet: Free (testnet BNB)
- BSC Mainnet: ~0.1-0.3 BNB
- Ethereum: Varies based on gas prices
- Polygon: Generally much cheaper than Ethereum

## Environment Setup

### How do I configure my environment?
1. Copy `.env.example` to `.env`
2. Fill in your private key and RPC endpoints
3. Add your Etherscan API key for contract verification

### Why do I need an Etherscan API key?
The Etherscan API key is used to verify your contracts on block explorers, making them transparent and trusted by users.

## Deployment Process

### What happens during deployment?
The deployment script:
1. Deploys PProjectToken contract
2. Deploys Vesting contract
3. Deploys Treasury contract
4. Deploys LiquidityPool contract
5. Saves contract addresses to deployment-info.json

### Can I deploy to testnet first?
Yes, it's highly recommended. Deploy to testnet first to verify everything works correctly before deploying to mainnet.

### How do I verify my contracts?
After deployment, use the verification script:
```bash
npx hardhat verify --network bsc CONTRACT_ADDRESS
```

## Troubleshooting

### Deployment fails with "insufficient funds"
Ensure your deployment wallet has sufficient funds for gas fees. Check the network's current gas prices and fund your wallet accordingly.

### Compilation errors
Make sure you're using a compatible Solidity version. Check the pragma directive in the contract files.

### Contract verification fails
Ensure your Etherscan API key is correct and that the contract addresses match those in deployment-info.json.

## Post-Deployment

### How do I set up initial liquidity?
1. Transfer P tokens to the liquidity pool
2. Transfer USDT to the liquidity pool
3. Call `addLiquidity()` on the LiquidityPool contract

### How do I initialize contracts?
Run the initialization script:
```bash
npx hardhat run scripts/initialize-contracts.js --network bscTestnet
```

### How do I prepare for DEX listing?
Run the DEX preparation script:
```bash
npx hardhat run scripts/prepare-dex-listing.js --network bscTestnet
```

## Security

### Should I audit the contracts?
Yes, it's highly recommended to have your contracts audited by a professional security firm before mainnet deployment.

### How do I secure my private key?
Never commit your private key to version control. Use environment variables and ensure your .env file is in .gitignore.

## Maintenance

### How do I update contract configurations?
Only the contract owner can update configurations. Use the appropriate functions in each contract.

### How do I monitor contract activity?
Use block explorers like BscScan or Etherscan to monitor contract activity and transactions.