// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
}

contract LinearVesting {
    IERC20 public immutable token;
    address public immutable beneficiary;
    uint64 public immutable start;
    uint64 public immutable cliff;
    uint64 public immutable duration;
    uint256 public immutable totalAllocation;
    uint256 public released;

    constructor(
        IERC20 _token,
        address _beneficiary,
        uint64 _start,
        uint64 _cliffDuration,
        uint64 _duration,
        uint256 _totalAllocation
    ) {
        require(_cliffDuration <= _duration, "cliff > duration");
        token = _token;
        beneficiary = _beneficiary;
        start = _start;
        cliff = _start + _cliffDuration;
        duration = _duration;
        totalAllocation = _totalAllocation;
    }

    function vestedAmount(uint64 timestamp) public view returns (uint256) {
        if (timestamp < cliff) {
            return 0;
        } else if (timestamp >= start + duration) {
            return totalAllocation;
        } else {
            uint64 elapsed = timestamp - start;
            return (totalAllocation * elapsed) / duration;
        }
    }

    function releasable() public view returns (uint256) {
        return vestedAmount(uint64(block.timestamp)) - released;
    }

    function release() external {
        require(msg.sender == beneficiary, "not beneficiary");
        uint256 amount = releasable();
        require(amount > 0, "nothing to release");
        released += amount;
        require(token.transfer(beneficiary, amount), "transfer failed");
    }
}