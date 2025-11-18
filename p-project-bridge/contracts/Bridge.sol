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
    // Ownership
    address public owner;
    uint256 public nonce;

    // Pausing and compliance
    bool public paused;
    bool public kycRequired;
    bool public enforceTokenAllowlist;
    mapping(address => bool) public kycApproved;     // address -> approved
    mapping(address => bool) public blocked;         // address -> blocked
    mapping(address => bool) public tokenAllowed;    // token -> allowed

    // Prevent replays of the same lockId on the destination chain
    mapping(bytes32 => bool) public processedLockIds;

    // Audit metadata
    struct AuditInfo {
        string firm;
        string reportURI;     // e.g., IPFS or HTTPS URL
        bytes32 reportHash;   // hash of the report/artifact
        uint256 timestamp;    // unix timestamp of audit
        bool finalized;       // once finalized, cannot be changed
    }
    AuditInfo public audit;

    // Events
    event Locked(bytes32 lockId, address indexed token, address indexed sender, uint256 amount, address recipient);
    event Minted(bytes32 lockId, address indexed token, address indexed recipient, uint256 amount);

    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
    event OwnershipRenounced(address indexed previousOwner);
    event Paused(address indexed account);
    event Unpaused(address indexed account);
    event TokenAllowlistUpdated(address indexed token, bool allowed);
    event AddressBlocklistUpdated(address indexed account, bool blocked);
    event KycStatusUpdated(address indexed account, bool approved);
    event KycRequiredUpdated(bool required);
    event EnforceTokenAllowlistUpdated(bool enforce);
    event AuditInfoUpdated(string firm, string reportURI, bytes32 reportHash, uint256 timestamp);
    event AuditFinalized();

    modifier onlyOwner() {
        require(msg.sender == owner, "not owner");
        _;
    }

    modifier whenNotPaused() {
        require(!paused, "paused");
        _;
    }

    constructor() {
        owner = msg.sender;
        emit OwnershipTransferred(address(0), owner);
    }

    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "zero");
        address prev = owner;
        owner = newOwner;
        emit OwnershipTransferred(prev, newOwner);
    }

    function renounceOwnership() external onlyOwner {
        address prev = owner;
        owner = address(0);
        emit OwnershipRenounced(prev);
    }

    // Source chain: user locks tokens into the bridge (must approve first)
    function lock(address token, uint256 amount, address recipient) external whenNotPaused returns (bytes32 lockId) {
        require(amount > 0, "amount=0");
        require(recipient != address(0), "recipient=0");
        require(!blocked[msg.sender] && !blocked[recipient], "blocked");
        if (kycRequired) {
            require(kycApproved[msg.sender] && kycApproved[recipient], "kyc");
        }
        if (enforceTokenAllowlist) {
            require(tokenAllowed[token], "token !allowed");
        }

        // Pull tokens into bridge custody
        require(IERC20(token).transferFrom(msg.sender, address(this), amount), "transferFrom failed");

        // Derive a unique lockId
        lockId = keccak256(abi.encodePacked(msg.sender, token, recipient, amount, nonce, block.chainid));
        nonce++;

        emit Locked(lockId, token, msg.sender, amount, recipient);
    }

    // Destination chain: bridge (owner/relayer) releases tokens to the recipient
    // For a true mint-on-destination design, token must be mintable and this function would call token.mint
    function mint(address token, address recipient, uint256 amount, bytes32 lockId) external onlyOwner whenNotPaused {
        require(!processedLockIds[lockId], "lockId processed");
        processedLockIds[lockId] = true;

        require(IERC20(token).transfer(recipient, amount), "transfer failed");
        emit Minted(lockId, token, recipient, amount);
    }

    // Admin utility: withdraw tokens held by the bridge (for testing or recovery)
    function withdraw(address token, address to, uint256 amount) external onlyOwner whenNotPaused {
        require(IERC20(token).transfer(to, amount), "withdraw failed");
    }

    // Pause controls
    function pause() external onlyOwner {
        require(!paused, "paused");
        paused = true;
        emit Paused(msg.sender);
    }

    function unpause() external onlyOwner {
        require(paused, "!paused");
        paused = false;
        emit Unpaused(msg.sender);
    }

    // Compliance management
    function setKycRequired(bool required) external onlyOwner {
        kycRequired = required;
        emit KycRequiredUpdated(required);
    }

    function setKyc(address account, bool approved) external onlyOwner {
        kycApproved[account] = approved;
        emit KycStatusUpdated(account, approved);
    }

    function setBlocked(address account, bool isBlocked) external onlyOwner {
        blocked[account] = isBlocked;
        emit AddressBlocklistUpdated(account, isBlocked);
    }

    function setEnforceTokenAllowlist(bool enforce) external onlyOwner {
        enforceTokenAllowlist = enforce;
        emit EnforceTokenAllowlistUpdated(enforce);
    }

    function setTokenAllowed(address token, bool allowed) external onlyOwner {
        tokenAllowed[token] = allowed;
        emit TokenAllowlistUpdated(token, allowed);
    }

    // Audit metadata management
    function setAuditInfo(
        string calldata firm,
        string calldata reportURI,
        bytes32 reportHash,
        uint256 timestamp
    ) external onlyOwner {
        require(!audit.finalized, "audit finalized");
        audit.firm = firm;
        audit.reportURI = reportURI;
        audit.reportHash = reportHash;
        audit.timestamp = timestamp;
        emit AuditInfoUpdated(firm, reportURI, reportHash, timestamp);
    }

    function finalizeAudit() external onlyOwner {
        require(!audit.finalized, "audit finalized");
        audit.finalized = true;
        emit AuditFinalized();
    }
}
