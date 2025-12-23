// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "./PProjectToken.sol";

/**
 * @title Liquidity Pool Contract
 * @dev Implements a constant product liquidity pool for P-Project Token
 */
contract LiquidityPool {
    PProjectToken public immutable token;
    address public immutable usdt;
    address public owner;
    
    string public poolId;
    uint256 public feeTier; // Fee tier scaled by 1e18 (e.g., 0.3% = 3e15)
    uint256 public totalLiquidity;
    uint256 public totalToken;
    uint256 public totalUSDT;
    uint256 public kConstant; // Constant product formula: x * y = k
    uint256 public totalVolume;
    uint256 public totalFees;
    
    // Liquidity provider positions
    struct LiquidityPosition {
        address user;
        uint256 liquidityAmount;
        uint256 tokenAmount;
        uint256 usdtAmount;
        uint256 startTime;
        uint256 durationDays;
    }
    mapping(address => LiquidityPosition) public liquidityPositions;
    address[] public liquidityProviders;
    
    // Liquidity lock
    bool public liquidityLocked;
    uint256 public lockStartTime;
    uint256 public lockDuration; // in days
    
    // Events
    event LiquidityAdded(address indexed user, uint256 tokenAmount, uint256 usdtAmount, uint256 liquidityAmount);
    event LiquidityRemoved(address indexed user, uint256 tokenAmount, uint256 usdtAmount, uint256 liquidityAmount);
    event SwapExecuted(address indexed user, string tokenIn, uint256 amountIn, uint256 amountOut, uint256 fee);
    event LiquidityLocked(uint256 durationDays);
    event LiquidityUnlocked();
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    modifier liquidityNotLocked() {
        require(!liquidityLocked || block.timestamp >= lockStartTime + (lockDuration * 1 days), "Liquidity is locked");
        _;
    }
    
    constructor(
        address _token,
        address _usdt,
        string memory _poolId,
        uint256 _feeTier,
        uint256 _lockDuration
    ) {
        token = PProjectToken(payable(_token));
        usdt = _usdt;
        poolId = _poolId;
        feeTier = _feeTier;
        owner = msg.sender;
        lockDuration = _lockDuration;
        liquidityLocked = false;
        
        emit OwnershipTransferred(address(0), owner);
    }
    
    /**
     * @dev Add liquidity to the pool
     * @param user Address of the liquidity provider
     * @param tokenAmount Amount of P tokens to add
     * @param usdtAmount Amount of USDT to add
     * @param durationDays Duration to lock liquidity in days
     * @return Liquidity amount received
     */
    function addLiquidity(
        address user,
        uint256 tokenAmount,
        uint256 usdtAmount,
        uint256 durationDays
    ) external onlyOwner returns (uint256) {
        require(tokenAmount > 0 && usdtAmount > 0, "Amounts must be positive");
        require(durationDays > 0, "Duration must be positive");
        
        // If this is the first liquidity added, initialize the k constant
        if (totalLiquidity == 0) {
            kConstant = tokenAmount * usdtAmount;
        }
        
        uint256 liquidityAmount = sqrt(tokenAmount * usdtAmount);
        
        // Update pool totals
        totalToken += tokenAmount;
        totalUSDT += usdtAmount;
        totalLiquidity += liquidityAmount;
        
        // Update or create liquidity position
        if (liquidityPositions[user].liquidityAmount == 0) {
            liquidityPositions[user] = LiquidityPosition({
                user: user,
                liquidityAmount: liquidityAmount,
                tokenAmount: tokenAmount,
                usdtAmount: usdtAmount,
                startTime: block.timestamp,
                durationDays: durationDays
            });
            liquidityProviders.push(user);
        } else {
            LiquidityPosition storage position = liquidityPositions[user];
            position.liquidityAmount += liquidityAmount;
            position.tokenAmount += tokenAmount;
            position.usdtAmount += usdtAmount;
            position.durationDays = position.durationDays > durationDays ? position.durationDays : durationDays;
        }
        
        emit LiquidityAdded(user, tokenAmount, usdtAmount, liquidityAmount);
        
        return liquidityAmount;
    }
    
    /**
     * @dev Remove liquidity from the pool
     * @param user Address of the liquidity provider
     * @param liquidityAmount Amount of liquidity to remove
     * @return Amount of P tokens and USDT received
     */
    function removeLiquidity(address user, uint256 liquidityAmount) external onlyOwner liquidityNotLocked returns (uint256, uint256) {
        require(liquidityAmount > 0, "Amount must be positive");
        require(liquidityPositions[user].liquidityAmount >= liquidityAmount, "Insufficient liquidity");
        
        LiquidityPosition storage position = liquidityPositions[user];
        
        // Calculate proportional amounts to return
        uint256 tokenAmount = (position.tokenAmount * liquidityAmount) / position.liquidityAmount;
        uint256 usdtAmount = (position.usdtAmount * liquidityAmount) / position.liquidityAmount;
        
        // Update position
        position.liquidityAmount -= liquidityAmount;
        position.tokenAmount -= tokenAmount;
        position.usdtAmount -= usdtAmount;
        
        // If position is empty, remove from providers list
        if (position.liquidityAmount == 0) {
            delete liquidityPositions[user];
            // Remove from providers array
            for (uint256 i = 0; i < liquidityProviders.length; i++) {
                if (liquidityProviders[i] == user) {
                    liquidityProviders[i] = liquidityProviders[liquidityProviders.length - 1];
                    liquidityProviders.pop();
                    break;
                }
            }
        }
        
        // Update pool totals
        totalToken -= tokenAmount;
        totalUSDT -= usdtAmount;
        totalLiquidity -= liquidityAmount;
        
        emit LiquidityRemoved(user, tokenAmount, usdtAmount, liquidityAmount);
        
        return (tokenAmount, usdtAmount);
    }
    
    /**
     * @dev Calculate swap output amount
     * @param tokenIn Token to swap ("P" or "USDT")
     * @param amountIn Amount to swap in
     * @return Amount out and fee
     */
    function calculateSwapOutput(string memory tokenIn, uint256 amountIn) public view returns (uint256, uint256) {
        require(amountIn > 0, "Amount must be positive");
        
        uint256 fee = (amountIn * feeTier) / 1e18;
        uint256 amountInAfterFee = amountIn - fee;
        
        uint256 amountOut;
        if (keccak256(bytes(tokenIn)) == keccak256(bytes("P"))) {
            // Swapping P for USDT
            uint256 newTokenReserve = totalToken + amountInAfterFee;
            uint256 newUSDTReserve = kConstant / newTokenReserve;
            amountOut = totalUSDT - newUSDTReserve;
        } else if (keccak256(bytes(tokenIn)) == keccak256(bytes("USDT"))) {
            // Swapping USDT for P
            uint256 newUSDTReserve = totalUSDT + amountInAfterFee;
            uint256 newTokenReserve = kConstant / newUSDTReserve;
            amountOut = totalToken - newTokenReserve;
        } else {
            revert("Invalid token");
        }
        
        return (amountOut, fee);
    }
    
    /**
     * @dev Execute a swap
     * @param user Address of the user
     * @param tokenIn Token to swap ("P" or "USDT")
     * @param amountIn Amount to swap in
     * @return Amount out
     */
    function swap(address user, string memory tokenIn, uint256 amountIn) external onlyOwner returns (uint256) {
        (uint256 amountOut, uint256 fee) = calculateSwapOutput(tokenIn, amountIn);
        require(amountOut > 0, "Insufficient liquidity");
        
        // Update volume and fees
        totalVolume += amountIn;
        totalFees += fee;
        
        emit SwapExecuted(user, tokenIn, amountIn, amountOut, fee);
        
        return amountOut;
    }
    
    /**
     * @dev Lock liquidity for a specified duration
     * @param durationDays Duration to lock liquidity in days
     */
    function lockLiquidity(uint256 durationDays) external onlyOwner {
        require(durationDays > 0, "Duration must be positive");
        liquidityLocked = true;
        lockStartTime = block.timestamp;
        lockDuration = durationDays;
        emit LiquidityLocked(durationDays);
    }
    
    /**
     * @dev Unlock liquidity
     */
    function unlockLiquidity() external onlyOwner {
        liquidityLocked = false;
        emit LiquidityUnlocked();
    }
    
    /**
     * @dev Get all liquidity providers
     * @return Array of liquidity provider addresses
     */
    function getLiquidityProviders() external view returns (address[] memory) {
        return liquidityProviders;
    }
    
    /**
     * @dev Calculate square root (using Babylonian method)
     * @param x Number to calculate square root of
     * @return Square root of x
     */
    function sqrt(uint256 x) internal pure returns (uint256) {
        if (x == 0) return 0;
        uint256 z = (x + 1) / 2;
        uint256 y = x;
        while (z < y) {
            y = z;
            z = (x / z + z) / 2;
        }
        return y;
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