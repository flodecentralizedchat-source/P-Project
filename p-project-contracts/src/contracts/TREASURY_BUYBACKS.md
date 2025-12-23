# Treasury Buyback Programs in P-Project

This document explains the treasury buyback functionality implemented in the Treasury contract, which allows the project to automatically or manually buy back tokens from the market to support the token price.

## Overview

The Treasury contract manages funds and implements various buyback mechanisms to reduce token supply and support the token price. These include manual buybacks, scheduled buybacks, and trigger-based buybacks.

## Key Components

### Treasury Management

The treasury manages funds in different assets (primarily USDT) and tracks allocations for various purposes.

### Buyback Records

All buyback operations are recorded with timestamps, amounts spent, tokens bought, and prices per token.

### Multi-sig Configuration

The treasury supports multi-signature operations for enhanced security.

## Buyback Mechanisms

### 1. Manual Buybacks

Direct buybacks executed by the owner when market conditions are favorable.

**Key Function**: `executeBuyback(uint256 amountToSpend, uint256 currentTokenPrice)`
- Executes an immediate buyback using treasury funds
- Only callable by the contract owner
- Parameters:
  - `amountToSpend`: Amount of USDT to spend on the buyback
  - `currentTokenPrice`: Current price of the token in USDT

### 2. Scheduled Buybacks

Automated buybacks that execute at predetermined times.

**Key Functions**:
1. `addScheduledBuyback(uint256 timestamp, uint256 amount, uint256 targetPrice)`
   - Adds a new scheduled buyback
   - Only callable by the contract owner

2. `executeScheduledBuybacks(uint256 currentTokenPrice)`
   - Executes all scheduled buybacks that are due
   - Only callable by the contract owner
   - Only executes when autoBuybackEnabled is true

### 3. Trigger-based Buybacks

Conditional buybacks that execute when specific market conditions are met.

**Key Functions**:
1. `addBuybackTrigger(string memory triggerName, string memory condition, uint256 threshold, uint256 amount)`
   - Adds a new buyback trigger
   - Only callable by the contract owner
   - Supported conditions: "price_drop", "volume_spike"

2. `checkBuybackTriggers(uint256 currentPrice, string memory marketCondition, uint256 conditionValue)`
   - Checks and executes buyback triggers based on market conditions
   - Only callable by the contract owner
   - Only executes when autoBuybackEnabled is true

## How Buybacks Work

### Manual Buybacks
1. Owner calls executeBuyback() with amount and current price
2. Contract calculates tokens to buy based on price
3. Contract spends USDT from reserves
4. Contract burns the purchased tokens
5. Buyback is recorded in buybackRecords

### Scheduled Buybacks
1. Owner adds scheduled buybacks with future timestamps
2. When timestamp is reached, owner calls executeScheduledBuybacks()
3. Contract checks each scheduled buyback:
   - If not executed AND timestamp is in the past
   - If sufficient funds are available
4. Contract executes buyback and burns tokens
5. Buyback is recorded and marked as executed

### Trigger-based Buybacks
1. Owner adds buyback triggers with conditions and thresholds
2. When market conditions change, owner calls checkBuybackTriggers()
3. Contract checks each trigger:
   - If not executed AND condition is met
   - If sufficient funds are available
4. Contract executes buyback and burns tokens
5. Buyback is recorded and trigger is marked as executed

## Key Functions

1. **addFunds(string memory asset, uint256 amount)**
   - Adds funds to the treasury
   - Only callable by the contract owner

2. **allocateFunds(string memory name, uint256 amount, string memory purpose)**
   - Allocates funds for specific purposes
   - Only callable by the contract owner

3. **setAutoBuybackEnabled(bool enabled)**
   - Enables or disables automatic buybacks
   - Only callable by the contract owner

4. **getBalance(string memory asset)**
   - Returns the treasury balance for an asset

5. **getBuybackRecords()**
   - Returns all buyback records

## Test Scenarios

The following test scenarios should be implemented to verify the treasury buyback functionality:

### Manual Buyback Tests

1. **Successful Buyback Test**
   - Add funds to treasury
   - Execute buyback with valid parameters
   - Verify tokens are bought and burned
   - Verify funds are deducted from treasury

2. **Insufficient Funds Test**
   - Attempt to execute buyback with insufficient funds
   - Verify transaction is reverted

3. **Zero Amount Test**
   - Attempt to execute buyback with zero amount
   - Verify transaction is reverted

### Scheduled Buyback Tests

1. **Future Timestamp Test**
   - Add scheduled buyback with future timestamp
   - Enable auto buyback
   - Call executeScheduledBuybacks() before timestamp
   - Verify no tokens are bought

2. **Due Timestamp Test**
   - Add scheduled buyback with past timestamp
   - Enable auto buyback
   - Call executeScheduledBuybacks()
   - Verify tokens are bought and burned

3. **Disabled Auto Buyback Test**
   - Add scheduled buyback with past timestamp
   - Keep auto buyback disabled
   - Call executeScheduledBuybacks()
   - Verify no tokens are bought

### Trigger-based Buyback Tests

1. **Price Drop Trigger Test**
   - Add price drop trigger
   - Call checkBuybackTriggers() with price below threshold
   - Verify buyback is executed

2. **Volume Spike Trigger Test**
   - Add volume spike trigger
   - Call checkBuybackTriggers() with volume above threshold
   - Verify buyback is executed

3. **Non-triggering Condition Test**
   - Add price drop trigger
   - Call checkBuybackTriggers() with price above threshold
   - Verify no buyback is executed

## Security Considerations

1. All buyback operations can only be initiated by the contract owner
2. Treasury funds are protected by access controls
3. Multi-sig configuration provides additional security for large transactions
4. Insufficient funds are checked before executing buybacks
5. Automatic execution must be explicitly enabled by the owner
6. All buyback operations are recorded for transparency

## Best Practices

1. Maintain adequate treasury reserves for buyback programs
2. Set realistic targets for scheduled buybacks
3. Monitor market conditions and adjust trigger thresholds accordingly
4. Use multi-sig for large buyback operations
5. Keep detailed records of all buyback activities
6. Communicate buyback programs to the community for transparency
7. Consider market impact when executing large buybacks
8. Regularly review and adjust buyback strategies based on market performance