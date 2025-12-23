// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../interfaces/IUniswapV2Pair.sol";

contract MockUniswapV2Pair is IUniswapV2Pair {
    uint private _totalSupply;
    mapping(address => uint) private _balanceOf;
    mapping(address => mapping(address => uint)) private _allowance;
    
    uint private _price0CumulativeLast = 0;
    uint private _price1CumulativeLast = 0;
    uint private _kLast = 0;
    
    address public token0;
    address public token1;
    
    uint112 private reserve0;
    uint112 private reserve1;
    uint32 private blockTimestampLast;
    
    // Remove conflicting state variables and use constants or functions instead
    string private constant _name = "Mock LP Token";
    string private constant _symbol = "MLP";
    uint8 private constant _decimals = 18;
    
    constructor() {
        _totalSupply = 1000000 * 10**18;
        _balanceOf[msg.sender] = _totalSupply;
    }
    
    function name() external pure override returns (string memory) {
        return _name;
    }
    
    function symbol() external pure override returns (string memory) {
        return _symbol;
    }
    
    function decimals() external pure override returns (uint8) {
        return _decimals;
    }
    
    function totalSupply() external view override returns (uint) {
        return _totalSupply;
    }
    
    function balanceOf(address owner) external view override returns (uint) {
        return _balanceOf[owner];
    }
    
    function allowance(address owner, address spender) external view override returns (uint) {
        return _allowance[owner][spender];
    }
    
    function approve(address spender, uint value) external override returns (bool) {
        _allowance[msg.sender][spender] = value;
        emit Approval(msg.sender, spender, value);
        return true;
    }
    
    function transfer(address to, uint value) external override returns (bool) {
        _transfer(msg.sender, to, value);
        return true;
    }
    
    function transferFrom(address from, address to, uint value) external override returns (bool) {
        if (_allowance[from][msg.sender] != type(uint).max) {
            _allowance[from][msg.sender] -= value;
        }
        _transfer(from, to, value);
        return true;
    }
    
    function _transfer(address from, address to, uint value) internal {
        _balanceOf[from] -= value;
        _balanceOf[to] += value;
        emit Transfer(from, to, value);
    }
    
    function mint(address to) external override returns (uint liquidity) {
        // Simplified mint implementation
        liquidity = 1000;
        _balanceOf[to] += liquidity;
        _totalSupply += liquidity;
        emit Transfer(address(0), to, liquidity);
        emit Mint(msg.sender, to);
        return liquidity;
    }
    
    function burn(address to) external override returns (uint amount0, uint amount1) {
        // Simplified burn implementation
        amount0 = 100;
        amount1 = 100;
        _balanceOf[to] += amount0;
        emit Transfer(address(0), to, amount0);
        emit Burn(msg.sender, amount0, amount1, to);
        return (amount0, amount1);
    }
    
    function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external override {
        // Simplified swap implementation
        emit Swap(msg.sender, amount0Out, amount1Out, 0, 0, to);
    }
    
    function skim(address to) external override {
        // Simplified skim implementation
    }
    
    function sync() external override {
        // Simplified sync implementation
    }
    
    function initialize(address _token0, address _token1) external override {
        token0 = _token0;
        token1 = _token1;
    }
    
    function getReserves() external view override returns (uint112 _reserve0, uint112 _reserve1, uint32 _blockTimestampLast) {
        return (reserve0, reserve1, blockTimestampLast);
    }
    
    function price0CumulativeLast() external view override returns (uint) {
        return _price0CumulativeLast;
    }
    
    function price1CumulativeLast() external view override returns (uint) {
        return _price1CumulativeLast;
    }
    
    function kLast() external view override returns (uint) {
        return _kLast;
    }
    
    // Add missing interface functions
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
        // Simplified implementation
    }
    
    function MINIMUM_LIQUIDITY() external pure override returns (uint) {
        return 1000;
    }
    
    function factory() external view override returns (address) {
        return address(0);
    }
}