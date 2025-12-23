// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "../interfaces/IERC20.sol";
import "./PProjectToken.sol";

/**
 * @title Linear Vesting Contract
 * @dev Implements linear vesting with cliff periods for P-Project Token
 */
contract Vesting {
    PProjectToken public immutable token;
    address public owner;
    
    struct VestingSchedule {
        address beneficiary;
        uint64 start;
        uint64 cliff;
        uint64 duration;
        uint256 totalAllocation;
        uint256 released;
        bool initialized;
    }
    
    mapping(address => VestingSchedule) public vestingSchedules;
    address[] public beneficiaries;
    
    event VestingScheduleCreated(
        address indexed beneficiary,
        uint64 start,
        uint64 cliff,
        uint64 duration,
        uint256 totalAllocation
    );
    
    event TokensReleased(address indexed beneficiary, uint256 amount);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    modifier onlyBeneficiary(address beneficiary) {
        require(msg.sender == beneficiary, "Not beneficiary");
        _;
    }
    
    constructor(address _token) {
        token = PProjectToken(payable(_token));
        owner = msg.sender;
        emit OwnershipTransferred(address(0), owner);
    }
    
    /**
     * @dev Create a vesting schedule for a beneficiary
     * @param beneficiary Address of the beneficiary
     * @param start Start time of the vesting
     * @param cliffDuration Duration of the cliff period in seconds
     * @param duration Total duration of the vesting in seconds
     * @param totalAllocation Total token allocation
     */
    function createVestingSchedule(
        address beneficiary,
        uint64 start,
        uint64 cliffDuration,
        uint64 duration,
        uint256 totalAllocation
    ) external onlyOwner {
        require(beneficiary != address(0), "Beneficiary is zero address");
        require(cliffDuration <= duration, "Cliff duration > total duration");
        require(vestingSchedules[beneficiary].initialized == false, "Vesting schedule already exists");
        require(totalAllocation > 0, "Total allocation must be > 0");
        
        vestingSchedules[beneficiary] = VestingSchedule({
            beneficiary: beneficiary,
            start: start,
            cliff: start + cliffDuration,
            duration: duration,
            totalAllocation: totalAllocation,
            released: 0,
            initialized: true
        });
        
        beneficiaries.push(beneficiary);
        
        emit VestingScheduleCreated(beneficiary, start, start + cliffDuration, duration, totalAllocation);
    }
    
    /**
     * @dev Calculate the vested amount for a beneficiary at a given timestamp
     * @param beneficiary Address of the beneficiary
     * @param timestamp Timestamp to calculate vested amount for
     * @return Vested amount
     */
    function vestedAmount(address beneficiary, uint64 timestamp) public view returns (uint256) {
        VestingSchedule storage schedule = vestingSchedules[beneficiary];
        require(schedule.initialized, "No vesting schedule for beneficiary");
        
        if (timestamp < schedule.cliff) {
            return 0;
        } else if (timestamp >= schedule.start + schedule.duration) {
            return schedule.totalAllocation;
        } else {
            uint64 elapsed = timestamp - schedule.start;
            return (schedule.totalAllocation * elapsed) / schedule.duration;
        }
    }
    
    /**
     * @dev Calculate the releasable amount for a beneficiary
     * @param beneficiary Address of the beneficiary
     * @return Releasable amount
     */
    function releasable(address beneficiary) public view returns (uint256) {
        return vestedAmount(beneficiary, uint64(block.timestamp)) - vestingSchedules[beneficiary].released;
    }
    
    /**
     * @dev Release vested tokens to a beneficiary
     * @param beneficiary Address of the beneficiary
     */
    function release(address beneficiary) external onlyBeneficiary(beneficiary) {
        uint256 amount = releasable(beneficiary);
        require(amount > 0, "No tokens to release");
        
        VestingSchedule storage schedule = vestingSchedules[beneficiary];
        schedule.released += amount;
        
        require(token.transfer(beneficiary, amount), "Token transfer failed");
        
        emit TokensReleased(beneficiary, amount);
    }
    
    /**
     * @dev Get all beneficiaries
     * @return Array of beneficiary addresses
     */
    function getBeneficiaries() external view returns (address[] memory) {
        return beneficiaries;
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