pragma solidity ^0.8.20;

interface IERC20 {
    function decimals() external view returns (uint8);
    function balanceOf(address owner) external view returns (uint256);
    function allowance(address owner, address spender) external view returns (uint256);
    function approve(address spender, uint256 value) external returns (bool);
    function transfer(address to, uint256 value) external returns (bool);
    function transferFrom(address from, address to, uint256 value) external returns (bool);
}

contract Bridge {
    address public owner;
    uint256 public nonce;

    // Prevent replays of the same lockId on the destination chain
    mapping(bytes32 => bool) public processedLockIds;

    event Locked(bytes32 lockId, address indexed token, address indexed sender, uint256 amount, address recipient);
    event Minted(bytes32 lockId, address indexed token, address indexed recipient, uint256 amount);

    modifier onlyOwner() {
        require(msg.sender == owner, "not owner");
        _;
    }

    constructor() {
        owner = msg.sender;
    }

    function setOwner(address newOwner) external onlyOwner {
        require(newOwner != address(0), "zero");
        owner = newOwner;
    }

    // Source chain: user locks tokens into the bridge (must approve first)
    function lock(address token, uint256 amount, address recipient) external returns (bytes32 lockId) {
        require(amount > 0, "amount=0");
        require(recipient != address(0), "recipient=0");

        // Pull tokens into bridge custody
        require(IERC20(token).transferFrom(msg.sender, address(this), amount), "transferFrom failed");

        // Derive a unique lockId
        lockId = keccak256(abi.encodePacked(msg.sender, token, recipient, amount, nonce, block.chainid));
        nonce++;

        emit Locked(lockId, token, msg.sender, amount, recipient);
    }

    // Destination chain: bridge (owner/relayer) releases tokens to the recipient
    // For a true mint-on-destination design, token must be mintable and this function would call token.mint
    function mint(address token, address recipient, uint256 amount, bytes32 lockId) external onlyOwner {
        require(!processedLockIds[lockId], "lockId processed");
        processedLockIds[lockId] = true;

        require(IERC20(token).transfer(recipient, amount), "transfer failed");
        emit Minted(lockId, token, recipient, amount);
    }

    // Admin utility: withdraw tokens held by the bridge (for testing or recovery)
    function withdraw(address token, address to, uint256 amount) external onlyOwner {
        require(IERC20(token).transfer(to, amount), "withdraw failed");
    }
}

