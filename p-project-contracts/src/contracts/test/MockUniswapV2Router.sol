// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../interfaces/IUniswapV2Router02.sol";

contract MockUniswapV2Router is IUniswapV2Router02 {
    address public factoryAddress;
    address public wethAddress;

    constructor() {
        factoryAddress = address(this); // Simplified for testing
        wethAddress = address(this); // Simplified for testing
    }

    function factory() external view override returns (address) {
        return factoryAddress;
    }

    function WETH() external view override returns (address) {
        return wethAddress;
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
        // Simplified implementation for testing
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
        // Simplified implementation for testing
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
        // Simplified implementation for testing
        return (1000, 1000);
    }

    function removeLiquidityETH(
        address token,
        uint liquidity,
        uint amountTokenMin,
        uint amountETHMin,
        address to,
        uint deadline
    ) external override returns (uint amountToken, uint amountETH) {
        // Simplified implementation for testing
        return (1000, 1000);
    }

    function swapExactTokensForTokens(
        uint amountIn,
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external override returns (uint[] memory amounts) {
        // Simplified implementation for testing
        amounts = new uint[](2);
        amounts[0] = amountIn;
        amounts[1] = amountIn * 997 / 1000; // 0.3% fee
        return amounts;
    }

    function swapTokensForExactTokens(
        uint amountOut,
        uint amountInMax,
        address[] calldata path,
        address to,
        uint deadline
    ) external override returns (uint[] memory amounts) {
        // Simplified implementation for testing
        amounts = new uint[](2);
        amounts[0] = amountOut * 1003 / 997; // Reverse calculation
        amounts[1] = amountOut;
        return amounts;
    }

    function swapExactETHForTokens(uint amountOutMin, address[] calldata path, address to, uint deadline)
        external
        payable
        override
        returns (uint[] memory amounts)
    {
        // Simplified implementation for testing
        amounts = new uint[](2);
        amounts[0] = msg.value;
        amounts[1] = msg.value * 997 / 1000; // 0.3% fee
        return amounts;
    }

    function swapTokensForExactETH(uint amountOut, uint amountInMax, address[] calldata path, address to, uint deadline)
        external
        override
        returns (uint[] memory amounts)
    {
        // Simplified implementation for testing
        amounts = new uint[](2);
        amounts[0] = amountOut * 1003 / 997; // Reverse calculation
        amounts[1] = amountOut;
        return amounts;
    }

    function swapExactTokensForETH(uint amountIn, uint amountOutMin, address[] calldata path, address to, uint deadline)
        external
        override
        returns (uint[] memory amounts)
    {
        // Simplified implementation for testing
        amounts = new uint[](2);
        amounts[0] = amountIn;
        amounts[1] = amountIn * 997 / 1000; // 0.3% fee
        return amounts;
    }

    function swapETHForExactTokens(uint amountOut, address[] calldata path, address to, uint deadline)
        external
        payable
        override
        returns (uint[] memory amounts)
    {
        // Simplified implementation for testing
        amounts = new uint[](2);
        amounts[0] = amountOut * 1003 / 997; // Reverse calculation
        amounts[1] = amountOut;
        return amounts;
    }

    function quote(uint amountA, uint reserveA, uint reserveB) external pure override returns (uint amountB) {
        return amountA * reserveB / reserveA;
    }

    function getAmountOut(uint amountIn, uint reserveIn, uint reserveOut) external pure override returns (uint amountOut) {
        uint amountInWithFee = amountIn * 997;
        uint numerator = amountInWithFee * reserveOut;
        uint denominator = reserveIn * 1000 + amountInWithFee;
        return numerator / denominator;
    }

    function getAmountIn(uint amountOut, uint reserveIn, uint reserveOut) external pure override returns (uint amountIn) {
        uint numerator = reserveIn * amountOut * 1000;
        uint denominator = (reserveOut - amountOut) * 997;
        return (numerator / denominator) + 1;
    }

    function getAmountsOut(uint amountIn, address[] calldata path) external view override returns (uint[] memory amounts) {
        // Simplified implementation for testing
        amounts = new uint[](2);
        amounts[0] = amountIn;
        amounts[1] = amountIn * 997 / 1000; // 0.3% fee
        return amounts;
    }

    function getAmountsIn(uint amountOut, address[] calldata path) external view override returns (uint[] memory amounts) {
        // Simplified implementation for testing
        amounts = new uint[](2);
        amounts[0] = amountOut * 1003 / 997; // Reverse calculation
        amounts[1] = amountOut;
        return amounts;
    }

    // IUniswapV2Router02 functions
    function removeLiquidityETHSupportingFeeOnTransferTokens(
        address token,
        uint liquidity,
        uint amountTokenMin,
        uint amountETHMin,
        address to,
        uint deadline
    ) external override returns (uint amountETH) {
        return 1000;
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
        return 1000;
    }

    function swapExactTokensForTokensSupportingFeeOnTransferTokens(
        uint amountIn,
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external override {
        // Simplified implementation for testing
        // Transfer ETH to the recipient
        payable(to).transfer(amountIn * 997 / 1000); // 0.3% fee
    }

    function swapExactETHForTokensSupportingFeeOnTransferTokens(
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external payable override {
        // Simplified implementation for testing
        // Transfer tokens to the recipient
    }

    function swapExactTokensForETHSupportingFeeOnTransferTokens(
        uint amountIn,
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external override {
        // Simplified implementation for testing
        // Transfer ETH to the recipient
        payable(to).transfer(amountIn * 997 / 1000); // 0.3% fee
    }
}