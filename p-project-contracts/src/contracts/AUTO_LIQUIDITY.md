# Auto-Liquidity Features in P-Project Token

This document explains the auto-liquidity functionality implemented in the PProjectToken contract, which automatically adds liquidity to decentralized exchanges based on transaction fees.

## Overview

The auto-liquidity mechanism in PProjectToken automatically converts a portion of transaction fees into liquidity pool tokens, which are then added to a Uniswap-style liquidity pool. This helps ensure there's always sufficient liquidity for trading the token.

## Key Components

### Liquidity Fees

The contract charges two types of liquidity fees on each transaction when trading is enabled:

1. **Liquidity Fee**: A percentage of tokens collected as liquidity fees
2. **Marketing Fee**: A percentage of tokens collected as marketing fees

These fees are configurable by the owner and are subject to a maximum limit.

### Key Functions

1. **setLiquidityFees(uint256 _liquidityFee, uint256 _marketingFee)**
   - Updates the liquidity and marketing fees
   - Only callable by the contract owner
   - Fees are specified in scaled format (1e18 = 100%)

2. **setMinTokensBeforeSwap(uint256 _minTokensBeforeSwap)**
   - Sets the minimum number of tokens that must be collected before swapping
   - Only callable by the contract owner

3. **setMarketingWallet(address _marketingWallet)**
   - Sets the wallet that receives marketing fees
   - Only callable by the contract owner

4. **setSwapAndLiquifyEnabled(bool _enabled)**
   - Enables or disables the automatic swap and liquify functionality
   - Only callable by the contract owner

5. **setUniswapRouter(address newAddress)**
   - Updates the Uniswap V2 router address
   - Only callable by the contract owner

### Internal Functions

1. **swapAndLiquify(uint256 contractTokenBalance)**
   - Automatically swaps tokens for ETH and adds liquidity
   - Called internally when the token balance exceeds the minimum threshold

2. **swapTokensForEth(uint256 tokenAmount)**
   - Swaps tokens for ETH using the Uniswap router

3. **addLiquidity(uint256 tokenAmount, uint256 ethAmount)**
   - Adds liquidity to the Uniswap pair

## How Auto-Liquidity Works

1. **Fee Collection**: When trading is enabled, each token transfer collects liquidity and marketing fees
2. **Fee Accumulation**: Fees are accumulated in the contract's token balance
3. **Threshold Check**: When the contract's token balance exceeds the minimum threshold, swapAndLiquify is triggered
4. **Token Swap**: Half of the accumulated tokens are swapped for ETH
5. **Liquidity Addition**: The ETH and remaining tokens are added as liquidity to the Uniswap pair
6. **Marketing Distribution**: Marketing fees are sent to the designated marketing wallet

## Fee Distribution

When a token transfer occurs with trading enabled:

1. Liquidity fee is calculated and added to contract balance
2. Marketing fee is calculated and added to contract balance
3. Remaining tokens are transferred to the recipient
4. Rewards are distributed to token holders based on reward rate

## Test Scenarios

The following test scenarios should be implemented to verify the auto-liquidity functionality:

### Liquidity Settings Tests

1. **Default Fee Test**
   - Verify default liquidity and marketing fees are set correctly
   - Check that fees can be updated by the owner

2. **Fee Limit Test**
   - Attempt to set fees above the maximum limit
   - Verify the transaction is reverted

3. **Marketing Wallet Test**
   - Update the marketing wallet address
   - Verify the new address is set correctly

### Liquidity Functionality Tests

1. **Fee Collection Test**
   - Enable trading
   - Transfer tokens between accounts
   - Verify fees are collected by the contract

2. **No Fee Collection Test**
   - Disable trading
   - Transfer tokens between accounts
   - Verify no fees are collected

3. **Swap and Liquify Test**
   - Set a low threshold for swap and liquify
   - Transfer enough tokens to trigger the threshold
   - Verify the SwapAndLiquify event is emitted

4. **Disabled Swap Test**
   - Disable swap and liquify
   - Transfer enough tokens to normally trigger the threshold
   - Verify the SwapAndLiquify event is NOT emitted

5. **Marketing Fee Distribution Test**
   - Set a marketing wallet
   - Transfer tokens to generate marketing fees
   - Verify marketing fees are sent to the marketing wallet

## Security Considerations

1. All liquidity settings can only be modified by the contract owner
2. Fee limits prevent excessive fee collection
3. Swap and liquify must be explicitly enabled
4. Marketing wallet can be updated by the owner for flexibility
5. Router address can be updated to support different DEXes

## Best Practices

1. Set reasonable liquidity and marketing fees (typically 1-5% each)
2. Ensure the minimum swap threshold is appropriate for the token's trading volume
3. Regularly monitor the marketing wallet balance
4. Consider market conditions when adjusting fees
5. Keep the Uniswap router address updated to the latest version
6. Test all fee changes thoroughly before implementing on mainnet