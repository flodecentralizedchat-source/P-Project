// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../interfaces/IUniswapV2Factory.sol";

contract MockUniswapV2Factory is IUniswapV2Factory {
    address public feeToAddress;
    address public feeToSetterAddress;
    mapping(address => mapping(address => address)) public getPair;
    address[] public allPairsList;

    constructor() {
        feeToSetterAddress = msg.sender;
    }

    function feeTo() external view override returns (address) {
        return feeToAddress;
    }

    function feeToSetter() external view override returns (address) {
        return feeToSetterAddress;
    }

    function allPairs(uint) external view override returns (address pair) {
        return address(this); // Simplified for testing
    }

    function allPairsLength() external view override returns (uint) {
        return allPairsList.length;
    }

    function createPair(address tokenA, address tokenB) external override returns (address pair) {
        // Simplified implementation for testing
        return address(this);
    }

    function setFeeTo(address _feeTo) external override {
        require(msg.sender == feeToSetterAddress, "Not feeToSetter");
        feeToAddress = _feeTo;
    }

    function setFeeToSetter(address _feeToSetter) external override {
        require(msg.sender == feeToSetterAddress, "Not feeToSetter");
        feeToSetterAddress = _feeToSetter;
    }
}