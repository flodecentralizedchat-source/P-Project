// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../interfaces/IUniswapV2Pair.sol";

contract MockUniswapV2Pair is IUniswapV2Pair {
    string public name = "Mock LP Token";
    string public symbol = "MLP";
    uint8 public decimals = 18;
    uint public totalSupply = 1000000 * 10**18;
    mapping(address => uint) public balanceOf;
    mapping(address => mapping(address => uint)) public allowance;
    
    address public token0Address;
    address public token1Address;
    uint112 public reserve0 = 1000000 * 10**18;
    uint112 public reserve1 = 1000000 * 10**18;
    uint32 public blockTimestampLast = 0;
    uint public price0CumulativeLast = 0;
    uint public price1CumulativeLast = 0;
    uint public kLast = 0;

    constructor() {
        balanceOf[address(this)] = totalSupply;
        token0Address = address(this);
        token1Address = address(this);
    }

    function name() external pure override returns (string memory) {
        return "Mock LP Token";
    }

    function symbol() external pure override returns (string memory) {
        return "MLP";
    }

    function decimals() external pure override returns (uint8) {
        return 18;
    }

    function totalSupply() external view override returns (uint) {
        return totalSupply;
    }

    function balanceOf(address owner) external view override returns (uint) {
        return balanceOf[owner];
    }

    function allowance(address owner, address spender) external view override returns (uint) {
        return allowance[owner][spender];
    }

    function approve(address spender, uint value) external override returns (bool) {
        allowance[msg.sender][spender] = value;
        emit Approval(msg.sender, spender, value);
        return true;
    }

    function transfer(address to, uint value) external override returns (bool) {
        _transfer(msg.sender, to, value);
        return true;
    }

    function transferFrom(address from, address to, uint value) external override returns (bool) {
        if (allowance[from][msg.sender] != type(uint).max) {
            allowance[from][msg.sender] -= value;
        }
        _transfer(from, to, value);
        return true;
    }

    function _transfer(address from, address to, uint value) internal {
        balanceOf[from] -= value;
        balanceOf[to] += value;
        emit Transfer(from, to, value);
    }

    function DOMAIN_SEPARATOR() external view override returns (bytes32) {
        return bytes32(0);
    }

    function PERMIT_TYPEHASH() external pure override returns (bytes32) {
        return bytes32(0);
    }

    function nonces(address owner) external view override returns (uint) {
        return 0;
    }

    function permit(address owner, address spender, uint value, uint deadline, uint8 v, bytes32 r, bytes32 s) external override {
        // Simplified for testing
    }

    function MINIMUM_LIQUIDITY() external pure override returns (uint) {
        return 1000;
    }

    function factory() external view override returns (address) {
        return address(this);
    }

    function token0() external view override returns (address) {
        return token0Address;
    }

    function token1() external view override returns (address) {
        return token1Address;
    }

    function getReserves() external view override returns (uint112 _reserve0, uint112 _reserve1, uint32 _blockTimestampLast) {
        return (reserve0, reserve1, blockTimestampLast);
    }

    function price0CumulativeLast() external view override returns (uint) {
        return price0CumulativeLast;
    }

    function price1CumulativeLast() external view override returns (uint) {
        return price1CumulativeLast;
    }

    function kLast() external view override returns (uint) {
        return kLast;
    }

    function mint(address to) external override returns (uint liquidity) {
        // Simplified for testing
        return 1000;
    }

    function burn(address to) external override returns (uint amount0, uint amount1) {
        // Simplified for testing
        return (1000, 1000);
    }

    function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external override {
        // Simplified for testing
    }

    function skim(address to) external override {
        // Simplified for testing
    }

    function sync() external override {
        // Simplified for testing
    }

    function initialize(address _token0, address _token1) external override {
        token0Address = _token0;
        token1Address = _token1;
    }
}