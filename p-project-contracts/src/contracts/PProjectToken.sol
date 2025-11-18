// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./interfaces/IUniswapV2Factory.sol";
import "./interfaces/IUniswapV2Router02.sol";

/**
 * @title P-Project Token
 * @dev ERC-20 token with deflationary mechanisms including dynamic burn rates,
 * transaction-based burns, scheduled burns, milestone burns, revenue-linked burns,
 * and auto-liquidity features.
 */
contract PProjectToken {
    // Token information
    string public constant name = "P-Project Token";
    string public constant symbol = "P";
    uint8 public constant decimals = 18;
    uint256 public totalSupply;
    
    // Addresses
    address public owner;
    mapping(address => bool) public restrictedWallets;
    
    // Balances and allowances
    mapping(address => uint256) private _balances;
    mapping(address => mapping(address => uint256)) private _allowances;
    
    // Activity tracking for dynamic burn rates
    mapping(address => uint256) public userActivityCount;
    uint256 public totalTransactions;
    
    // Anti-whale mechanisms
    uint256 public maxTransferLimit; // 5% of total supply
    uint256 public maxDailyTransferPercent; // 3% of total supply
    mapping(address => uint256) public dailyTransferAmount;
    mapping(address => uint256) public lastTransferReset;
    bool public botProtectionEnabled;
    uint256 public botCooldownPeriod; // in seconds
    mapping(address => uint256) public userLastTransaction;
    
    // Burn mechanism parameters
    uint256 public baseBurnRate; // Base percentage to burn on each transaction (scaled by 1e18)
    uint256 public rewardRate; // Percentage to distribute to holders (scaled by 1e18)
    
    // Scheduled burns
    struct ScheduledBurn {
        uint256 timestamp;
        uint256 amount;
        bool executed;
    }
    ScheduledBurn[] public scheduledBurns;
    bool public burnScheduleEnabled;
    
    // Milestone burns
    struct MilestoneBurn {
        string milestoneName;
        string target; // "holders_count", "transactions_count", "supply_reduction"
        uint256 targetValue;
        uint256 burnAmount;
        bool executed;
    }
    MilestoneBurn[] public milestoneBurns;
    address[] public holders;
    mapping(address => bool) public isHolder;
    
    // Revenue-linked burns
    struct RevenueLinkedBurn {
        string revenueSource; // "staking_rewards", "transaction_fees", "partnerships"
        uint256 revenueAmount;
        uint256 burnPercentage; // Percentage of revenue to burn (scaled by 1e18)
        bool executed;
    }
    RevenueLinkedBurn[] public revenueLinkedBurns;
    
    // Liquidity mechanisms
    mapping(string => uint256) public liquidityPools;
    mapping(string => bool) public liquidityLocked;
    
    // Auto-liquidity mechanisms
    IUniswapV2Router02 public uniswapV2Router;
    address public uniswapV2Pair;
    bool public tradingEnabled;
    bool public inSwapAndLiquify;
    
    // Auto-liquidity settings
    uint256 public liquidityFee; // Percentage of liquidity fee (scaled by 1e18)
    uint256 public marketingFee; // Percentage of marketing fee (scaled by 1e18)
    uint256 public maxLiquidityFee; // Maximum liquidity fee (scaled by 1e18)
    uint256 public minTokensBeforeSwap; // Minimum tokens before swap
    address public marketingWallet;
    bool public swapAndLiquifyEnabled;
    
    // Events
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event TokensBurned(address indexed executor, uint256 amount);
    event RewardsDistributed(address indexed recipient, uint256 amount);
    event ScheduledBurnAdded(uint256 timestamp, uint256 amount);
    event ScheduledBurnExecuted(uint256 amount);
    event MilestoneBurnAdded(string milestoneName, uint256 burnAmount);
    event MilestoneBurnExecuted(string milestoneName, uint256 burnAmount);
    event RevenueBurnAdded(string revenueSource, uint256 burnAmount);
    event RevenueBurnExecuted(string revenueSource, uint256 burnAmount);
    event LiquidityAdded(string poolId, address user, uint256 amount);
    event LiquidityRemoved(string poolId, address user, uint256 amount);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    
    // Auto-liquidity events
    event SwapAndLiquifyEnabledUpdated(bool enabled);
    event LiquidityFeesUpdated(uint256 liquidityFee, uint256 marketingFee);
    event MinTokensBeforeSwapUpdated(uint256 minTokensBeforeSwap);
    event MarketingWalletUpdated(address marketingWallet);
    event SwapAndLiquify(
        uint256 tokensSwapped,
        uint256 ethReceived,
        uint256 tokensIntoLiqudity
    );
    event SendToMarketing(uint256 amount);
    
    // Modifiers
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    modifier lockTheSwap() {
        inSwapAndLiquify = true;
        _;
        inSwapAndLiquify = false;
    }
    
    /**
     * @dev Constructor to initialize the token
     * @param _totalSupply Total supply of tokens
     * @param _burnRate Base burn rate (scaled by 1e18, e.g., 1% = 1e16)
     * @param _rewardRate Reward rate for holders (scaled by 1e18)
     */
    constructor(uint256 _totalSupply, uint256 _burnRate, uint256 _rewardRate) {
        owner = msg.sender;
        totalSupply = _totalSupply;
        baseBurnRate = _burnRate;
        rewardRate = _rewardRate;
        maxTransferLimit = (_totalSupply * 5) / 100; // 5% of total supply
        maxDailyTransferPercent = 3; // 3% of total supply
        botProtectionEnabled = true;
        botCooldownPeriod = 60; // 60 seconds
        burnScheduleEnabled = false;
        
        // Auto-liquidity settings
        liquidityFee = 3e16; // 3% liquidity fee
        marketingFee = 2e16; // 2% marketing fee
        maxLiquidityFee = 10e16; // 10% maximum fee
        minTokensBeforeSwap = 1000 * 10**decimals; // 1000 tokens
        marketingWallet = owner;
        swapAndLiquifyEnabled = true;
        tradingEnabled = false;
        
        // Mint initial supply to owner
        _balances[owner] = _totalSupply;
        emit Transfer(address(0), owner, _totalSupply);
    }
    
    /**
     * @dev Returns the balance of an account
     * @param account Address to check balance for
     * @return Balance of the account
     */
    function balanceOf(address account) public view returns (uint256) {
        return _balances[account];
    }
    
    /**
     * @dev Returns the allowance of a spender for an owner
     * @param owner Address that owns the tokens
     * @param spender Address that is allowed to spend tokens
     * @return Allowance amount
     */
    function allowance(address owner, address spender) public view returns (uint256) {
        return _allowances[owner][spender];
    }
    
    /**
     * @dev Transfers tokens to an address
     * @param to Address to transfer to
     * @param amount Amount to transfer
     * @return Success status
     */
    function transfer(address to, uint256 amount) public returns (bool) {
        _transfer(msg.sender, to, amount);
        return true;
    }
    
    /**
     * @dev Approves a spender to spend tokens
     * @param spender Address to approve
     * @param amount Amount to approve
     * @return Success status
     */
    function approve(address spender, uint256 amount) public returns (bool) {
        _approve(msg.sender, spender, amount);
        return true;
    }
    
    /**
     * @dev Transfers tokens on behalf of an address
     * @param from Address to transfer from
     * @param to Address to transfer to
     * @param amount Amount to transfer
     * @return Success status
     */
    function transferFrom(address from, address to, uint256 amount) public returns (bool) {
        uint256 currentAllowance = _allowances[from][msg.sender];
        require(currentAllowance >= amount, "Insufficient allowance");
        
        _transfer(from, to, amount);
        _approve(from, msg.sender, currentAllowance - amount);
        
        return true;
    }
    
    /**
     * @dev Internal transfer function with deflationary mechanisms and auto-liquidity
     * @param sender Address sending tokens
     * @param recipient Address receiving tokens
     * @param amount Amount to transfer
     */
    function _transfer(address sender, address recipient, uint256 amount) internal {
        require(sender != address(0), "Transfer from zero address");
        require(recipient != address(0), "Transfer to zero address");
        require(amount > 0, "Transfer amount must be greater than zero");
        require(_balances[sender] >= amount, "Insufficient balance");
        
        // Check if sender wallet is restricted
        require(!restrictedWallets[sender], "Wallet is restricted");
        
        // Anti-whale check
        require(amount <= maxTransferLimit, "Transfer amount exceeds maximum limit");
        
        // Daily transfer limit check
        _checkDailyTransferLimit(sender, amount);
        
        // Bot protection check
        _checkBotProtection(sender);
        
        // Track activity for dynamic burn rate
        userActivityCount[sender] += 1;
        totalTransactions += 1;
        
        // Update last transaction time for bot protection
        userLastTransaction[sender] = block.timestamp;
        
        // Calculate fees if trading is enabled and not in swap
        uint256 liquidityFeeAmount = 0;
        uint256 marketingFeeAmount = 0;
        uint256 burnAmount = 0;
        uint256 transferAmount = amount;
        
        if (tradingEnabled && !inSwapAndLiquify) {
            // Calculate liquidity fee
            liquidityFeeAmount = (amount * liquidityFee) / 1e18;
            
            // Calculate marketing fee
            marketingFeeAmount = (amount * marketingFee) / 1e18;
            
            // Calculate dynamic burn amount
            uint256 dynamicBurnRate = _getDynamicBurnRate(sender);
            burnAmount = (amount * dynamicBurnRate) / 1e18;
            
            // Calculate transfer amount
            transferAmount = amount - liquidityFeeAmount - marketingFeeAmount - burnAmount;
        }
        
        // Update balances
        _balances[sender] -= amount;
        
        // Add to recipient balance
        _balances[recipient] += transferAmount;
        
        // Update total supply (burn)
        totalSupply -= burnAmount;
        
        // Add recipient to holders if not already present
        if (_balances[recipient] > 0 && !isHolder[recipient]) {
            isHolder[recipient] = true;
            holders.push(recipient);
        }
        
        // Handle liquidity fees
        if (liquidityFeeAmount > 0) {
            _balances[address(this)] += liquidityFeeAmount;
            emit Transfer(sender, address(this), liquidityFeeAmount);
        }
        
        // Handle marketing fees
        if (marketingFeeAmount > 0) {
            _balances[address(this)] += marketingFeeAmount;
            emit Transfer(sender, address(this), marketingFeeAmount);
        }
        
        // Distribute rewards to holders
        _distributeRewards((burnAmount * rewardRate) / 1e18);
        
        // Swap and liquify if needed
        if (swapAndLiquifyEnabled && 
            !inSwapAndLiquify && 
            address(uniswapV2Router) != address(0) &&
            _balances[address(this)] >= minTokensBeforeSwap) {
            swapAndLiquify(minTokensBeforeSwap);
        }
        
        // Send marketing fees if needed
        if (marketingFeeAmount > 0 && marketingWallet != address(0)) {
            sendToMarketing(marketingFeeAmount);
        }
        
        emit Transfer(sender, recipient, transferAmount);
        if (burnAmount > 0) {
            emit TokensBurned(sender, burnAmount);
        }
    }
    
    /**
     * @dev Internal approve function
     * @param owner Address that owns the tokens
     * @param spender Address that is allowed to spend tokens
     * @param amount Amount to approve
     */
    function _approve(address owner, address spender, uint256 amount) internal {
        require(owner != address(0), "Approve from zero address");
        require(spender != address(0), "Approve to zero address");
        
        _allowances[owner][spender] = amount;
        emit Approval(owner, spender, amount);
    }
    
    /**
     * @dev Get dynamic burn rate based on network activity
     * @param user Address of the user
     * @return Dynamic burn rate (scaled by 1e18)
     */
    function _getDynamicBurnRate(address user) internal view returns (uint256) {
        // Base burn rate
        uint256 burnRate = baseBurnRate;
        
        // Increase burn rate during high activity periods
        // Check if this user has been very active (more than 10 transactions)
        if (userActivityCount[user] > 10) {
            // Increase burn rate by up to 50% for highly active users
            uint256 activityMultiplier = (userActivityCount[user] * 1e18) / 100;
            if (activityMultiplier > 5e17) {
                activityMultiplier = 5e17; // Cap at 50%
            }
            burnRate += (baseBurnRate * activityMultiplier) / 1e18;
        }
        
        // Global activity factor - increase burn rate when network is very active
        if (totalTransactions > 10000) {
            // Increase burn rate by up to 30% during high network activity
            uint256 networkActivityMultiplier = (totalTransactions * 1e18) / 100000;
            if (networkActivityMultiplier > 3e17) {
                networkActivityMultiplier = 3e17; // Cap at 30%
            }
            burnRate += (baseBurnRate * networkActivityMultiplier) / 1e18;
        }
        
        // Cap the burn rate at a maximum of 5%
        if (burnRate > 5e16) {
            burnRate = 5e16;
        }
        
        return burnRate;
    }
    
    /**
     * @dev Check daily transfer limit for a user
     * @param user Address of the user
     * @param amount Amount to transfer
     */
    function _checkDailyTransferLimit(address user, uint256 amount) internal {
        uint256 maxDailyLimit = (totalSupply * maxDailyTransferPercent) / 100;
        
        // Reset daily limit if it's a new day
        if (block.timestamp - lastTransferReset[user] >= 1 days) {
            dailyTransferAmount[user] = 0;
            lastTransferReset[user] = block.timestamp;
        }
        
        // Check if adding this amount would exceed daily limit
        require(dailyTransferAmount[user] + amount <= maxDailyLimit, "Daily transfer limit exceeded");
        
        // Update daily amount
        dailyTransferAmount[user] += amount;
    }
    
    /**
     * @dev Check bot protection cooldown
     * @param user Address of the user
     */
    function _checkBotProtection(address user) internal view {
        if (!botProtectionEnabled) {
            return;
        }
        
        if (userLastTransaction[user] > 0) {
            require(block.timestamp - userLastTransaction[user] >= botCooldownPeriod, "Bot protection cooldown");
        }
    }
    
    /**
     * @dev Distribute rewards to all holders
     * @param rewardPool Amount of rewards to distribute
     */
    function _distributeRewards(uint256 rewardPool) internal {
        if (holders.length == 0 || rewardPool == 0) {
            return;
        }
        
        // Calculate total holdings for weighted distribution
        uint256 totalHoldings = 0;
        for (uint256 i = 0; i < holders.length; i++) {
            totalHoldings += _balances[holders[i]];
        }
        
        if (totalHoldings == 0) {
            return;
        }
        
        // Distribute rewards
        for (uint256 i = 0; i < holders.length; i++) {
            uint256 holderBalance = _balances[holders[i]];
            if (holderBalance > 0) {
                uint256 rewardAmount = (rewardPool * holderBalance) / totalHoldings;
                if (rewardAmount > 0) {
                    _balances[holders[i]] += rewardAmount;
                    emit RewardsDistributed(holders[i], rewardAmount);
                }
            }
        }
    }
    
    /**
     * @dev Set maximum transfer limit for anti-whale mechanism
     * @param limit New maximum transfer limit
     */
    function setMaxTransferLimit(uint256 limit) external onlyOwner {
        maxTransferLimit = limit;
    }
    
    /**
     * @dev Set maximum daily transfer percent
     * @param percent New maximum daily transfer percent
     */
    function setMaxDailyTransferPercent(uint256 percent) external onlyOwner {
        maxDailyTransferPercent = percent;
    }
    
    /**
     * @dev Enable or disable bot protection
     * @param enabled Whether bot protection is enabled
     */
    function setBotProtection(bool enabled) external onlyOwner {
        botProtectionEnabled = enabled;
    }
    
    /**
     * @dev Set bot cooldown period in seconds
     * @param seconds Cooldown period in seconds
     */
    function setBotCooldownPeriod(uint256 seconds) external onlyOwner {
        botCooldownPeriod = seconds;
    }
    
    /**
     * @dev Restrict a wallet (e.g., team wallets)
     * @param user Address to restrict
     * @param restricted Whether the wallet is restricted
     */
    function restrictWallet(address user, bool restricted) external onlyOwner {
        restrictedWallets[user] = restricted;
    }
    
    /**
     * @dev Add liquidity to a pool
     * @param poolId ID of the pool
     * @param user Address of the user
     * @param amount Amount to add
     */
    function addLiquidity(string memory poolId, address user, uint256 amount) external onlyOwner {
        require(_balances[user] >= amount, "Insufficient balance");
        
        // Update user balance
        _balances[user] -= amount;
        
        // Update liquidity pool
        liquidityPools[poolId] += amount;
        
        emit LiquidityAdded(poolId, user, amount);
    }
    
    /**
     * @dev Remove liquidity from a pool
     * @param poolId ID of the pool
     * @param user Address of the user
     * @param amount Amount to remove
     */
    function removeLiquidity(string memory poolId, address user, uint256 amount) external onlyOwner {
        require(liquidityPools[poolId] >= amount, "Insufficient liquidity");
        require(!liquidityLocked[poolId], "Liquidity is locked");
        
        // Update liquidity pool
        liquidityPools[poolId] -= amount;
        
        // Update user balance
        _balances[user] += amount;
        
        emit LiquidityRemoved(poolId, user, amount);
    }
    
    /**
     * @dev Lock liquidity for a pool
     * @param poolId ID of the pool
     */
    function lockLiquidity(string memory poolId) external onlyOwner {
        liquidityLocked[poolId] = true;
    }
    
    /**
     * @dev Unlock liquidity for a pool
     * @param poolId ID of the pool
     */
    function unlockLiquidity(string memory poolId) external onlyOwner {
        liquidityLocked[poolId] = false;
    }
    
    /**
     * @dev Burn tokens directly (for buyback programs)
     * @param amount Amount to burn
     */
    function burnTokens(uint256 amount) external onlyOwner {
        require(amount > 0, "Amount must be positive");
        require(amount <= totalSupply, "Insufficient supply");
        
        totalSupply -= amount;
        emit TokensBurned(msg.sender, amount);
    }
    
    /**
     * @dev Add a scheduled burn
     * @param timestamp Timestamp when burn should occur
     * @param amount Amount to burn
     */
    function addScheduledBurn(uint256 timestamp, uint256 amount) external onlyOwner {
        scheduledBurns.push(ScheduledBurn({
            timestamp: timestamp,
            amount: amount,
            executed: false
        }));
        burnScheduleEnabled = true;
        emit ScheduledBurnAdded(timestamp, amount);
    }
    
    /**
     * @dev Execute scheduled burns that are due
     * @return Total amount burned
     */
    function executeScheduledBurns() external onlyOwner returns (uint256) {
        if (!burnScheduleEnabled) {
            return 0;
        }
        
        uint256 totalBurned = 0;
        uint256 currentTime = block.timestamp;
        
        for (uint256 i = 0; i < scheduledBurns.length; i++) {
            if (!scheduledBurns[i].executed && scheduledBurns[i].timestamp <= currentTime) {
                if (scheduledBurns[i].amount > 0 && scheduledBurns[i].amount <= totalSupply) {
                    totalSupply -= scheduledBurns[i].amount;
                    totalBurned += scheduledBurns[i].amount;
                    scheduledBurns[i].executed = true;
                    emit ScheduledBurnExecuted(scheduledBurns[i].amount);
                }
            }
        }
        
        return totalBurned;
    }
    
    /**
     * @dev Add a milestone-based burn
     * @param milestoneName Name of the milestone
     * @param target Target type ("holders_count", "transactions_count", "supply_reduction")
     * @param targetValue Target value to trigger burn
     * @param burnAmount Amount to burn when milestone is reached
     */
    function addMilestoneBurn(
        string memory milestoneName,
        string memory target,
        uint256 targetValue,
        uint256 burnAmount
    ) external onlyOwner {
        milestoneBurns.push(MilestoneBurn({
            milestoneName: milestoneName,
            target: target,
            targetValue: targetValue,
            burnAmount: burnAmount,
            executed: false
        }));
        emit MilestoneBurnAdded(milestoneName, burnAmount);
    }
    
    /**
     * @dev Check and execute milestone-based burns
     * @return Total amount burned
     */
    function checkMilestoneBurns() external onlyOwner returns (uint256) {
        uint256 totalBurned = 0;
        uint256 initialSupply = 350000000 * 10**uint256(decimals); // Assuming initial supply
        
        for (uint256 i = 0; i < milestoneBurns.length; i++) {
            if (!milestoneBurns[i].executed) {
                bool shouldExecute = false;
                
                if (keccak256(bytes(milestoneBurns[i].target)) == keccak256(bytes("holders_count"))) {
                    shouldExecute = holders.length >= milestoneBurns[i].targetValue;
                } else if (keccak256(bytes(milestoneBurns[i].target)) == keccak256(bytes("transactions_count"))) {
                    shouldExecute = totalTransactions >= milestoneBurns[i].targetValue;
                } else if (keccak256(bytes(milestoneBurns[i].target)) == keccak256(bytes("supply_reduction"))) {
                    // This would be based on percentage reduction from initial supply
                    uint256 reductionPercentage = ((initialSupply - totalSupply) * 100) / initialSupply;
                    shouldExecute = reductionPercentage >= milestoneBurns[i].targetValue;
                }
                
                if (shouldExecute) {
                    if (milestoneBurns[i].burnAmount > 0 && milestoneBurns[i].burnAmount <= totalSupply) {
                        totalSupply -= milestoneBurns[i].burnAmount;
                        totalBurned += milestoneBurns[i].burnAmount;
                        milestoneBurns[i].executed = true;
                        emit MilestoneBurnExecuted(milestoneBurns[i].milestoneName, milestoneBurns[i].burnAmount);
                    }
                }
            }
        }
        
        return totalBurned;
    }
    
    /**
     * @dev Add a revenue-linked burn
     * @param revenueSource Source of revenue
     * @param revenueAmount Amount of revenue
     * @param burnPercentage Percentage of revenue to burn (scaled by 1e18)
     */
    function addRevenueLinkedBurn(
        string memory revenueSource,
        uint256 revenueAmount,
        uint256 burnPercentage
    ) external onlyOwner {
        revenueLinkedBurns.push(RevenueLinkedBurn({
            revenueSource: revenueSource,
            revenueAmount: revenueAmount,
            burnPercentage: burnPercentage,
            executed: false
        }));
        
        uint256 burnAmount = (revenueAmount * burnPercentage) / 1e18;
        emit RevenueBurnAdded(revenueSource, burnAmount);
    }
    
    /**
     * @dev Execute revenue-linked burns
     * @return Total amount burned
     */
    function executeRevenueLinkedBurns() external onlyOwner returns (uint256) {
        uint256 totalBurned = 0;
        
        for (uint256 i = 0; i < revenueLinkedBurns.length; i++) {
            if (!revenueLinkedBurns[i].executed) {
                uint256 burnAmount = (revenueLinkedBurns[i].revenueAmount * revenueLinkedBurns[i].burnPercentage) / 1e18;
                
                if (burnAmount > 0 && burnAmount <= totalSupply) {
                    totalSupply -= burnAmount;
                    totalBurned += burnAmount;
                    revenueLinkedBurns[i].executed = true;
                    emit RevenueBurnExecuted(revenueLinkedBurns[i].revenueSource, burnAmount);
                }
            }
        }
        
        return totalBurned;
    }
    
    /**
     * @dev Enable or disable burn schedule
     * @param enabled Whether burn schedule is enabled
     */
    function setBurnScheduleEnabled(bool enabled) external onlyOwner {
        burnScheduleEnabled = enabled;
    }
    
    /**
     * @dev Update the Uniswap V2 router address
     * @param newAddress New router address
     */
    function setUniswapRouter(address newAddress) external onlyOwner {
        uniswapV2Router = IUniswapV2Router02(newAddress);
        uniswapV2Pair = IUniswapV2Factory(uniswapV2Router.factory())
            .getPair(address(this), uniswapV2Router.WETH());
        
        // If pair doesn't exist, create it
        if (uniswapV2Pair == address(0)) {
            uniswapV2Pair = IUniswapV2Factory(uniswapV2Router.factory())
                .createPair(address(this), uniswapV2Router.WETH());
        }
    }
    
    /**
     * @dev Enable or disable trading
     * @param _enabled Whether trading is enabled
     */
    function setTradingEnabled(bool _enabled) external onlyOwner {
        tradingEnabled = _enabled;
    }
    
    /**
     * @dev Update liquidity and marketing fees
     * @param _liquidityFee New liquidity fee (scaled by 1e18)
     * @param _marketingFee New marketing fee (scaled by 1e18)
     */
    function setLiquidityFees(uint256 _liquidityFee, uint256 _marketingFee) external onlyOwner {
        require(_liquidityFee + _marketingFee <= maxLiquidityFee, "Fees exceed maximum");
        liquidityFee = _liquidityFee;
        marketingFee = _marketingFee;
        emit LiquidityFeesUpdated(_liquidityFee, _marketingFee);
    }
    
    /**
     * @dev Update minimum tokens before swap
     * @param _minTokensBeforeSwap New minimum tokens before swap
     */
    function setMinTokensBeforeSwap(uint256 _minTokensBeforeSwap) external onlyOwner {
        minTokensBeforeSwap = _minTokensBeforeSwap;
        emit MinTokensBeforeSwapUpdated(_minTokensBeforeSwap);
    }
    
    /**
     * @dev Update marketing wallet address
     * @param _marketingWallet New marketing wallet address
     */
    function setMarketingWallet(address _marketingWallet) external onlyOwner {
        marketingWallet = _marketingWallet;
        emit MarketingWalletUpdated(_marketingWallet);
    }
    
    /**
     * @dev Enable or disable swap and liquify
     * @param _enabled Whether swap and liquify is enabled
     */
    function setSwapAndLiquifyEnabled(bool _enabled) external onlyOwner {
        swapAndLiquifyEnabled = _enabled;
        emit SwapAndLiquifyEnabledUpdated(_enabled);
    }
    
    /**
     * @dev Swap tokens for ETH and add liquidity
     */
    function swapAndLiquify(uint256 contractTokenBalance) private lockTheSwap {
        // Split the contract balance into halves
        uint256 half = contractTokenBalance / 2;
        uint256 otherHalf = contractTokenBalance - half;
        
        // Capture the contract's current ETH balance.
        uint256 initialBalance = address(this).balance;
        
        // Swap tokens for ETH
        swapTokensForEth(half);
        
        // How much ETH did we just swap into?
        uint256 newBalance = address(this).balance - initialBalance;
        
        // Add liquidity to uniswap
        addLiquidity(otherHalf, newBalance);
        
        emit SwapAndLiquify(half, newBalance, otherHalf);
    }
    
    /**
     * @dev Swap tokens for ETH
     * @param tokenAmount Amount of tokens to swap
     */
    function swapTokensForEth(uint256 tokenAmount) private {
        // Generate the uniswap pair path of token -> weth
        address[] memory path = new address[](2);
        path[0] = address(this);
        path[1] = uniswapV2Router.WETH();
        
        _approve(address(this), address(uniswapV2Router), tokenAmount);
        
        // Make the swap
        uniswapV2Router.swapExactTokensForETHSupportingFeeOnTransferTokens(
            tokenAmount,
            0, // accept any amount of ETH
            path,
            address(this),
            block.timestamp
        );
    }
    
    /**
     * @dev Add liquidity to Uniswap
     * @param tokenAmount Amount of tokens to add
     * @param ethAmount Amount of ETH to add
     */
    function addLiquidity(uint256 tokenAmount, uint256 ethAmount) private {
        // Approve token for uniswap router
        _approve(address(this), address(uniswapV2Router), tokenAmount);
        
        // Add the liquidity
        uniswapV2Router.addLiquidityETH{value: ethAmount}(
            address(this),
            tokenAmount,
            0, // slippage is unavoidable
            0, // slippage is unavoidable
            owner,
            block.timestamp
        );
    }
    
    /**
     * @dev Send tokens to marketing wallet
     * @param amount Amount of tokens to send
     */
    function sendToMarketing(uint256 amount) private {
        _transfer(address(this), marketingWallet, amount);
        emit SendToMarketing(amount);
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
    
    /**
     * @dev Receive ETH from Uniswap swaps
     */
    receive() external payable {}
}