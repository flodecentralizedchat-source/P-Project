// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../interfaces/IUniswapV2Router02.sol";
import "../interfaces/IUniswapV2Factory.sol";
import "./MockUniswapV2Factory.sol";

contract MockUniswapV2Router is IUniswapV2Router02 {
    address private _factoryAddress;
    address private _wethAddress;
    address private _mockPairAddress;
    
    constructor(address mockPairAddress) {
        _factoryAddress = address(new MockUniswapV2Factory()); // Create a mock factory
        _wethAddress = address(this); // Use this contract as WETH for simplicity
        _mockPairAddress = mockPairAddress;
    }
    
    function factory() external pure override returns (address) {
        return address(0x1234); // Return a fixed address for testing
    }
    
    function WETH() external pure override returns (address) {
        return address(0x5678); // Return a fixed address for testing
    }
    
    function addLiquidity(
        address tokenA,
        address tokenB,
        uint amountADesired,
        uint amountBDesired,
        uint amountAMin,
        uint amountBMin,
        address to,
        uint deadline
    ) external override returns (uint amountA, uint amountB, uint liquidity) {
        // Simplified implementation
        return (amountADesired, amountBDesired, 1000);
    }
    
    function addLiquidityETH(
        address token,
        uint amountTokenDesired,
        uint amountTokenMin,
        uint amountETHMin,
        address to,
        uint deadline
    ) external payable override returns (uint amountToken, uint amountETH, uint liquidity) {
        // Simplified implementation
        return (amountTokenDesired, msg.value, 1000);
    }
    
    function removeLiquidity(
        address tokenA,
        address tokenB,
        uint liquidity,
        uint amountAMin,
        uint amountBMin,
        address to,
        uint deadline
    ) external override returns (uint amountA, uint amountB) {
        // Simplified implementation
        return (100, 100);
    }
    
    function removeLiquidityETH(
        address token,
        uint liquidity,
        uint amountTokenMin,
        uint amountETHMin,
        address to,
        uint deadline
    ) external override returns (uint amountToken, uint amountETH) {
        // Simplified implementation
        return (100, 100);
    }
    
    function removeLiquidityWithPermit(
        address tokenA,
        address tokenB,
        uint liquidity,
        uint amountAMin,
        uint amountBMin,
        address to,
        uint deadline,
        bool approveMax, uint8 v, bytes32 r, bytes32 s
    ) external override returns (uint amountA, uint amountB) {
        // Simplified implementation
        return (100, 100);
    }
    
    function removeLiquidityETHWithPermit(
        address token,
        uint liquidity,
        uint amountTokenMin,
        uint amountETHMin,
        address to,
        uint deadline,
        bool approveMax, uint8 v, bytes32 r, bytes32 s
    ) external override returns (uint amountToken, uint amountETH) {
        // Simplified implementation
        return (100, 100);
    }
    
    function swapExactTokensForTokens(
        uint amountIn,
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external override returns (uint[] memory amounts) {
        // Simplified implementation
        amounts = new uint[](2);
        amounts[0] = amountIn;
        amounts[1] = amountIn; // 1:1 swap for simplicity
        return amounts;
    }
    
    function swapTokensForExactTokens(
        uint amountOut,
        uint amountInMax,
        address[] calldata path,
        address to,
        uint deadline
    ) external override returns (uint[] memory amounts) {
        // Simplified implementation
        amounts = new uint[](2);
        amounts[0] = amountOut; // 1:1 swap for simplicity
        amounts[1] = amountOut;
        return amounts;
    }
    
    function swapExactETHForTokens(uint amountOutMin, address[] calldata path, address to, uint deadline)
        external
        payable
        override
        returns (uint[] memory amounts)
    {
        // Simplified implementation
        amounts = new uint[](2);
        amounts[0] = msg.value;
        amounts[1] = msg.value; // 1:1 swap for simplicity
        return amounts;
    }
    
    function swapTokensForExactETH(uint amountOut, uint amountInMax, address[] calldata path, address to, uint deadline)
        external
        override
        returns (uint[] memory amounts)
    {
        // Simplified implementation
        amounts = new uint[](2);
        amounts[0] = amountOut; // 1:1 swap for simplicity
        amounts[1] = amountOut;
        return amounts;
    }
    
    function swapExactTokensForETH(uint amountIn, uint amountOutMin, address[] calldata path, address to, uint deadline)
        external
        override
        returns (uint[] memory amounts)
    {
        // Simplified implementation
        amounts = new uint[](2);
        amounts[0] = amountIn;
        amounts[1] = amountIn; // 1:1 swap for simplicity
        return amounts;
    }
    
    function swapETHForExactTokens(uint amountOut, address[] calldata path, address to, uint deadline)
        external
        payable
        override
        returns (uint[] memory amounts)
    {
        // Simplified implementation
        amounts = new uint[](2);
        amounts[0] = amountOut; // 1:1 swap for simplicity
        amounts[1] = amountOut;
        return amounts;
    }
    
    function quote(uint amountA, uint reserveA, uint reserveB) public pure override returns (uint amountB) {
        return amountA * reserveB / reserveA;
    }
    
    function getAmountOut(uint amountIn, uint reserveIn, uint reserveOut) public pure override returns (uint amountOut) {
        return amountIn * reserveOut / reserveIn;
    }
    
    function getAmountIn(uint amountOut, uint reserveIn, uint reserveOut) public pure override returns (uint amountIn) {
        return amountOut * reserveIn / reserveOut;
    }
    
    function getAmountsOut(uint amountIn, address[] calldata path) external view override returns (uint[] memory amounts) {
        // Simplified implementation
        amounts = new uint[](path.length);
        for (uint i = 0; i < path.length; i++) {
            amounts[i] = amountIn;
        }
        return amounts;
    }
    
    function getAmountsIn(uint amountOut, address[] calldata path) external view override returns (uint[] memory amounts) {
        // Simplified implementation
        amounts = new uint[](path.length);
        for (uint i = 0; i < path.length; i++) {
            amounts[i] = amountOut;
        }
        return amounts;
    }
    
    function removeLiquidityETHSupportingFeeOnTransferTokens(
        address token,
        uint liquidity,
        uint amountTokenMin,
        uint amountETHMin,
        address to,
        uint deadline
    ) external override returns (uint amountETH) {
        // Simplified implementation
        return 100;
    }
    
    function removeLiquidityETHWithPermitSupportingFeeOnTransferTokens(
        address token,
        uint liquidity,
        uint amountTokenMin,
        uint amountETHMin,
        address to,
        uint deadline,
        bool approveMax, uint8 v, bytes32 r, bytes32 s
    ) external override returns (uint amountETH) {
        // Simplified implementation
        return 100;
    }
    
    function swapExactTokensForTokensSupportingFeeOnTransferTokens(
        uint amountIn,
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external override {
        // Simplified implementation
        // In a real implementation, this would swap tokens
        // For testing purposes, we'll just transfer the tokens
        // No need to do anything here as the tokens are already transferred
    }
    
    function swapExactETHForTokensSupportingFeeOnTransferTokens(
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external payable override {
        // Simplified implementation
        // In a real implementation, this would swap ETH for tokens
        // For testing purposes, we'll just transfer the ETH
    }
    
    function swapExactTokensForETHSupportingFeeOnTransferTokens(
        uint amountIn,
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external override {
        // Simplified implementation
        // In a real implementation, this would swap tokens for ETH
        // For testing purposes, we'll just transfer the ETH
        // Check if contract has enough ETH to transfer
        if (address(this).balance >= amountIn * 997 / 1000) {
            payable(to).transfer(amountIn * 997 / 1000); // 0.3% fee
        }
    }
    
    // Add a receive function to allow the contract to receive ETH
    receive() external payable {}
}