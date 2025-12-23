// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./PProjectToken.sol";

/**
 * @title Treasury Contract
 * @dev Manages treasury funds and buyback programs for P-Project Token
 */
contract Treasury {
    PProjectToken public immutable token;
    address public owner;
    
    // Treasury reserves
    mapping(string => uint256) public reserves;
    
    // Treasury allocations
    struct TreasuryAllocation {
        string name;
        uint256 amount;
        string purpose;
    }
    TreasuryAllocation[] public allocations;
    
    // Buyback records
    struct BuybackRecord {
        uint256 timestamp;
        uint256 amountSpent;
        uint256 tokensBought;
        uint256 pricePerToken;
    }
    BuybackRecord[] public buybackRecords;
    uint256 public totalBuybacks;
    
    // Multi-sig configuration
    address[] public multisigSigners;
    uint256 public multisigRequired;
    mapping(string => bool) public executedTransactions;
    
    // Scheduled buybacks
    struct BuybackSchedule {
        uint256 timestamp;
        uint256 amount;
        uint256 targetPrice;
        bool executed;
    }
    BuybackSchedule[] public scheduledBuybacks;
    bool public autoBuybackEnabled;
    
    // Trigger-based buybacks
    struct BuybackTrigger {
        string triggerName;
        string condition; // "price_drop", "volume_spike"
        uint256 threshold; // Threshold value for the condition
        uint256 amount; // Amount to spend on buyback
        bool executed;
    }
    BuybackTrigger[] public buybackTriggers;
    
    // Events
    event FundsAdded(string asset, uint256 amount);
    event AllocationCreated(string name, uint256 amount, string purpose);
    event BuybackExecuted(uint256 amountSpent, uint256 tokensBought, uint256 pricePerToken);
    event ScheduledBuybackAdded(uint256 timestamp, uint256 amount, uint256 targetPrice);
    event ScheduledBuybackExecuted(uint256 amountSpent, uint256 tokensBought, uint256 pricePerToken);
    event BuybackTriggerAdded(string triggerName, string condition, uint256 threshold, uint256 amount);
    event TriggerBuybackExecuted(string triggerName, uint256 amountSpent, uint256 tokensBought, uint256 pricePerToken);
    event AutoBuybackEnabled(bool enabled);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    constructor(address _token, address[] memory _signers, uint256 _required) {
        token = PProjectToken(payable(_token));
        owner = msg.sender;
        
        // Initialize multi-sig signers
        for (uint256 i = 0; i < _signers.length; i++) {
            multisigSigners.push(_signers[i]);
        }
        multisigRequired = _required;
        autoBuybackEnabled = false;
        
        emit OwnershipTransferred(address(0), owner);
    }
    
    /**
     * @dev Add funds to treasury
     * @param asset Asset type (e.g., "USDT", "ETH")
     * @param amount Amount to add
     */
    function addFunds(string memory asset, uint256 amount) external onlyOwner {
        require(amount > 0, "Amount must be positive");
        reserves[asset] += amount;
        emit FundsAdded(asset, amount);
    }
    
    /**
     * @dev Get treasury balance for an asset
     * @param asset Asset type
     * @return Balance of the asset
     */
    function getBalance(string memory asset) public view returns (uint256) {
        return reserves[asset];
    }
    
    /**
     * @dev Allocate funds for specific purposes
     * @param name Name of the allocation
     * @param amount Amount to allocate
     * @param purpose Purpose of the allocation
     */
    function allocateFunds(string memory name, uint256 amount, string memory purpose) external onlyOwner {
        require(amount > 0, "Amount must be positive");
        require(amount <= getBalance("USDT"), "Insufficient funds");
        
        allocations.push(TreasuryAllocation({
            name: name,
            amount: amount,
            purpose: purpose
        }));
        
        reserves["USDT"] -= amount;
        emit AllocationCreated(name, amount, purpose);
    }
    
    /**
     * @dev Execute token buyback program
     * @param amountToSpend Amount of USDT to spend
     * @param currentTokenPrice Current price of token in USDT
     * @return Tokens bought and burned
     */
    function executeBuyback(uint256 amountToSpend, uint256 currentTokenPrice) external onlyOwner returns (uint256) {
        require(amountToSpend > 0, "Amount must be positive");
        require(currentTokenPrice > 0, "Price must be positive");
        require(amountToSpend <= getBalance("USDT"), "Insufficient funds");
        
        // Calculate how many tokens we can buy
        uint256 tokensToBuy = (amountToSpend * 1e18) / currentTokenPrice;
        
        // Record the buyback
        buybackRecords.push(BuybackRecord({
            timestamp: block.timestamp,
            amountSpent: amountToSpend,
            tokensBought: tokensToBuy,
            pricePerToken: currentTokenPrice
        }));
        
        totalBuybacks += amountToSpend;
        
        // Deduct funds from treasury
        reserves["USDT"] -= amountToSpend;
        
        // Burn the tokens to reduce supply (deflationary mechanism)
        token.burnTokens(tokensToBuy);
        
        emit BuybackExecuted(amountToSpend, tokensToBuy, currentTokenPrice);
        
        return tokensToBuy;
    }
    
    /**
     * @dev Add a scheduled buyback
     * @param timestamp Timestamp when buyback should occur
     * @param amount Amount to spend on buyback
     * @param targetPrice Target price for buyback
     */
    function addScheduledBuyback(uint256 timestamp, uint256 amount, uint256 targetPrice) external onlyOwner {
        require(amount > 0, "Amount must be positive");
        require(amount <= getBalance("USDT"), "Insufficient funds");
        
        scheduledBuybacks.push(BuybackSchedule({
            timestamp: timestamp,
            amount: amount,
            targetPrice: targetPrice,
            executed: false
        }));
        
        emit ScheduledBuybackAdded(timestamp, amount, targetPrice);
    }
    
    /**
     * @dev Execute scheduled buybacks that are due
     * @param currentTokenPrice Current price of token in USDT
     * @return Total tokens bought
     */
    function executeScheduledBuybacks(uint256 currentTokenPrice) external onlyOwner returns (uint256) {
        if (!autoBuybackEnabled) {
            return 0;
        }
        
        uint256 totalTokensBought = 0;
        uint256 currentTime = block.timestamp;
        
        for (uint256 i = 0; i < scheduledBuybacks.length; i++) {
            if (!scheduledBuybacks[i].executed && scheduledBuybacks[i].timestamp <= currentTime) {
                if (scheduledBuybacks[i].amount > 0 && scheduledBuybacks[i].amount <= getBalance("USDT")) {
                    // Calculate how many tokens we can buy
                    uint256 tokensToBuy = (scheduledBuybacks[i].amount * 1e18) / scheduledBuybacks[i].targetPrice;
                    
                    // Record the buyback
                    buybackRecords.push(BuybackRecord({
                        timestamp: block.timestamp,
                        amountSpent: scheduledBuybacks[i].amount,
                        tokensBought: tokensToBuy,
                        pricePerToken: scheduledBuybacks[i].targetPrice
                    }));
                    
                    totalBuybacks += scheduledBuybacks[i].amount;
                    totalTokensBought += tokensToBuy;
                    scheduledBuybacks[i].executed = true;
                    
                    // Deduct funds from treasury
                    reserves["USDT"] -= scheduledBuybacks[i].amount;
                    
                    // Burn the tokens to reduce supply (deflationary mechanism)
                    token.burnTokens(tokensToBuy);
                    
                    emit ScheduledBuybackExecuted(scheduledBuybacks[i].amount, tokensToBuy, scheduledBuybacks[i].targetPrice);
                }
            }
        }
        
        return totalTokensBought;
    }
    
    /**
     * @dev Add a buyback trigger
     * @param triggerName Name of the trigger
     * @param condition Condition type ("price_drop", "volume_spike")
     * @param threshold Threshold value for the condition
     * @param amount Amount to spend on buyback
     */
    function addBuybackTrigger(
        string memory triggerName,
        string memory condition,
        uint256 threshold,
        uint256 amount
    ) external onlyOwner {
        require(amount > 0, "Amount must be positive");
        require(amount <= getBalance("USDT"), "Insufficient funds");
        
        buybackTriggers.push(BuybackTrigger({
            triggerName: triggerName,
            condition: condition,
            threshold: threshold,
            amount: amount,
            executed: false
        }));
        
        emit BuybackTriggerAdded(triggerName, condition, threshold, amount);
    }
    
    /**
     * @dev Check and execute buyback triggers based on market conditions
     * @param currentPrice Current token price
     * @param marketCondition Market condition type
     * @param conditionValue Condition value
     * @return Total tokens bought
     */
    function checkBuybackTriggers(
        uint256 currentPrice,
        string memory marketCondition,
        uint256 conditionValue
    ) external onlyOwner returns (uint256) {
        if (!autoBuybackEnabled) {
            return 0;
        }
        
        uint256 totalTokensBought = 0;
        
        for (uint256 i = 0; i < buybackTriggers.length; i++) {
            if (!buybackTriggers[i].executed) {
                bool shouldExecute = false;
                
                if (keccak256(bytes(buybackTriggers[i].condition)) == keccak256(bytes("price_drop"))) {
                    shouldExecute = currentPrice <= buybackTriggers[i].threshold;
                } else if (keccak256(bytes(buybackTriggers[i].condition)) == keccak256(bytes("volume_spike"))) {
                    shouldExecute = conditionValue >= buybackTriggers[i].threshold;
                }
                
                if (shouldExecute) {
                    if (buybackTriggers[i].amount > 0 && buybackTriggers[i].amount <= getBalance("USDT")) {
                        // Calculate how many tokens we can buy
                        uint256 tokensToBuy = (buybackTriggers[i].amount * 1e18) / currentPrice;
                        
                        // Record the buyback
                        buybackRecords.push(BuybackRecord({
                            timestamp: block.timestamp,
                            amountSpent: buybackTriggers[i].amount,
                            tokensBought: tokensToBuy,
                            pricePerToken: currentPrice
                        }));
                        
                        totalBuybacks += buybackTriggers[i].amount;
                        totalTokensBought += tokensToBuy;
                        buybackTriggers[i].executed = true;
                        
                        // Deduct funds from treasury
                        reserves["USDT"] -= buybackTriggers[i].amount;
                        
                        // Burn the tokens to reduce supply (deflationary mechanism)
                        token.burnTokens(tokensToBuy);
                        
                        emit TriggerBuybackExecuted(buybackTriggers[i].triggerName, buybackTriggers[i].amount, tokensToBuy, currentPrice);
                    }
                }
            }
        }
        
        return totalTokensBought;
    }
    
    /**
     * @dev Enable or disable auto buybacks
     * @param enabled Whether auto buybacks are enabled
     */
    function setAutoBuybackEnabled(bool enabled) external onlyOwner {
        autoBuybackEnabled = enabled;
        emit AutoBuybackEnabled(enabled);
    }
    
    /**
     * @dev Get all allocations
     * @return Array of treasury allocations
     */
    function getAllocations() external view returns (TreasuryAllocation[] memory) {
        return allocations;
    }
    
    /**
     * @dev Get all buyback records
     * @return Array of buyback records
     */
    function getBuybackRecords() external view returns (BuybackRecord[] memory) {
        return buybackRecords;
    }
    
    /**
     * @dev Get all scheduled buybacks
     * @return Array of scheduled buybacks
     */
    function getScheduledBuybacks() external view returns (BuybackSchedule[] memory) {
        return scheduledBuybacks;
    }
    
    /**
     * @dev Get all buyback triggers
     * @return Array of buyback triggers
     */
    function getBuybackTriggers() external view returns (BuybackTrigger[] memory) {
        return buybackTriggers;
    }
    
    /**
     * @dev Transfer ownership of the contract
     * @param newOwner Address of the new owner
     */
    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "New owner is zero address");
        address oldOwner = owner;
        owner = newOwner;
        emit OwnershipTransferred(oldOwner, newOwner);
    }
    
    /**
     * @dev Renounce ownership of the contract
     */
    function renounceOwnership() external onlyOwner {
        emit OwnershipTransferred(owner, address(0));
        owner = address(0);
    }
}