# Scheduled Operations in P-Project Contracts

This document explains the scheduled operations functionality implemented in the P-Project smart contracts, including scheduled burns and scheduled buybacks.

## Overview

Scheduled operations allow the P-Project token ecosystem to automatically execute deflationary mechanisms and treasury buybacks at predetermined times. This creates predictable tokenomics events that can help maintain or increase token value over time.

## Scheduled Burns (PProjectToken.sol)

The PProjectToken contract implements scheduled burns that can be executed automatically when their timestamp is reached.

### Key Functions

1. **addScheduledBurn(uint256 timestamp, uint256 amount)**
   - Adds a new scheduled burn to the contract
   - Only callable by the contract owner
   - Parameters:
     - `timestamp`: When the burn should be executed (Unix timestamp)
     - `amount`: How many tokens to burn

2. **executeScheduledBurns()**
   - Executes all scheduled burns that are due (timestamp <= current time)
   - Only callable by the contract owner
   - Returns the total amount of tokens burned
   - Only executes when burnScheduleEnabled is true

3. **setBurnScheduleEnabled(bool enabled)**
   - Enables or disables the automatic execution of scheduled burns
   - Only callable by the contract owner

### How It Works

1. Owner adds scheduled burns with future timestamps
2. When the timestamp is reached, the owner (or an automated process) calls executeScheduledBurns()
3. The function checks each scheduled burn:
   - If it hasn't been executed yet AND the timestamp is in the past
   - If the burn amount is valid (greater than 0 and less than total supply)
   - Then it executes the burn by reducing the total supply
   - Marks the burn as executed to prevent double execution

## Scheduled Buybacks (Treasury.sol)

The Treasury contract implements scheduled buybacks that automatically purchase tokens from the market when their timestamp is reached.

### Key Functions

1. **addScheduledBuyback(uint256 timestamp, uint256 amount, uint256 targetPrice)**
   - Adds a new scheduled buyback to the treasury
   - Only callable by the contract owner
   - Parameters:
     - `timestamp`: When the buyback should be executed (Unix timestamp)
     - `amount`: How much USDT to spend on the buyback
     - `targetPrice`: Target price for the buyback calculation

2. **executeScheduledBuybacks(uint256 currentTokenPrice)**
   - Executes all scheduled buybacks that are due (timestamp <= current time)
   - Only callable by the contract owner
   - Returns the total number of tokens bought
   - Only executes when autoBuybackEnabled is true

3. **setAutoBuybackEnabled(bool enabled)**
   - Enables or disables the automatic execution of scheduled buybacks
   - Only callable by the contract owner

### How It Works

1. Owner adds scheduled buybacks with future timestamps and funding
2. Treasury must have sufficient USDT reserves to cover the buyback amounts
3. When the timestamp is reached, the owner (or an automated process) calls executeScheduledBuybacks()
4. The function checks each scheduled buyback:
   - If it hasn't been executed yet AND the timestamp is in the past
   - If the buyback amount is valid and covered by treasury reserves
   - Then it calculates how many tokens can be bought at the target price
   - Executes the buyback by spending USDT and burning the purchased tokens
   - Records the buyback in the buybackRecords array
   - Marks the buyback as executed to prevent double execution

## Test Scenarios

The following test scenarios should be implemented to verify the scheduled operations functionality:

### Scheduled Burn Tests

1. **Future Timestamp Test**
   - Add a scheduled burn with a future timestamp
   - Enable burn schedule
   - Call executeScheduledBurns() before the timestamp
   - Verify no tokens are burned

2. **Due Timestamp Test**
   - Add a scheduled burn with a past timestamp
   - Enable burn schedule
   - Call executeScheduledBurns()
   - Verify tokens are burned and supply is reduced

3. **Disabled Schedule Test**
   - Add a scheduled burn with a past timestamp
   - Keep burn schedule disabled
   - Call executeScheduledBurns()
   - Verify no tokens are burned

### Scheduled Buyback Tests

1. **Future Timestamp Test**
   - Add a scheduled buyback with a future timestamp
   - Enable auto buyback
   - Call executeScheduledBuybacks() before the timestamp
   - Verify no tokens are bought

2. **Due Timestamp Test**
   - Add a scheduled buyback with a past timestamp
   - Enable auto buyback
   - Call executeScheduledBuybacks()
   - Verify tokens are bought and burned

3. **Disabled Auto Buyback Test**
   - Add a scheduled buyback with a past timestamp
   - Keep auto buyback disabled
   - Call executeScheduledBuybacks()
   - Verify no tokens are bought

## Security Considerations

1. All scheduled operations can only be added and executed by the contract owner
2. Double execution is prevented by marking operations as "executed"
3. Insufficient funds are checked before executing buybacks
4. Invalid burn amounts (zero or exceeding supply) are rejected
5. Automatic execution must be explicitly enabled by the owner

## Best Practices

1. Schedule burns and buybacks at regular intervals to create predictable deflation
2. Ensure treasury has sufficient funds before scheduling buybacks
3. Monitor market conditions and adjust target prices accordingly
4. Consider using automated scripts or oracles to trigger execution at the right time
5. Keep detailed records of all scheduled operations for transparency