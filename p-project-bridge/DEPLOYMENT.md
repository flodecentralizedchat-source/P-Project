# Bridge Contract Deployment (Ethereum)

This guide walks you through deploying the minimal Bridge Solidity contract and configuring the app to use it.

## What You’ll Deploy
- `Bridge.sol` at `p-project-bridge/contracts/Bridge.sol`
  - `lock(token, amount, recipient) -> lockId`: pulls approved tokens from user into the bridge and emits `Locked`.
  - `mint(token, recipient, amount, lockId)`: owner releases tokens to recipient on destination chain and emits `Minted`.

Note: This simple Bridge holds tokens on the chain it’s deployed to and “releases” them. For a wrapped-token/mint model, point `token` to a mintable token and change `mint` to call `token.mint(...)`.

## Quick Deploy via Remix (Sepolia recommended)
1. Open Remix (https://remix.ethereum.org)
2. Create file `Bridge.sol` and paste contents from `p-project-bridge/contracts/Bridge.sol`.
3. Compile with Solidity 0.8.20+ (or compatible).
4. In Deploy & Run:
   - Environment: Injected Provider — MetaMask (select Sepolia or your target network)
   - Deploy `Bridge` (no constructor args)
   - Confirm the transaction
5. Copy the deployed contract address → set `ETH_BRIDGE_ADDRESS`.

## Token Address
You need an ERC‑20 token address (`ETH_TOKEN_ADDRESS`) on the same network:
- Option A: Use an existing ERC‑20 test token you control.
- Option B: Deploy an OpenZeppelin ERC20 (e.g., via Remix Wizard or a simple ERC20 contract), then mint some tokens to your wallet.

## Environment Variables
Set the following in your app environment:

- `ETH_RPC_URL` — your RPC endpoint (e.g., Sepolia)
- `ETH_BRIDGE_ADDRESS` — deployed Bridge address
- `ETH_TOKEN_ADDRESS` — ERC‑20 token used for locking/releasing
- `ETH_PRIVATE_KEY` — private key of the relayer/operator account (DON’T commit this)
- Optional: `ETH_PRIVATE_KEY_ENV` — name of the private key env var (defaults to `ETH_PRIVATE_KEY`)
- Optional: `ETH_CONFIRMATIONS` — confirmations threshold (default 3)

Example (.env):
```
ETH_RPC_URL=https://sepolia.infura.io/v3/<project-id>
ETH_BRIDGE_ADDRESS=0xYourBridgeAddress
ETH_TOKEN_ADDRESS=0xYourTokenAddress
ETH_PRIVATE_KEY=0xabc123...
ETH_CONFIRMATIONS=3
```

## Testing the Flow
1. Fund your account with test ETH and test tokens.
2. App path (from code):
   - `BridgeService.bridge_tokens(user_id, from="Ethereum", to=...)` calls Ethereum adapter `lock`:
     - Ensures allowance, calls `lock(token, amount, recipient)` on Bridge
     - Stores `src_tx_hash` and marks DB status `Locked`
   - Relayer checks confirmations, then calls destination adapter `mint_or_release(...)`

Manual test via Remix:
- Approve: On the token contract, call `approve(bridge, amount)` from your wallet.
- Lock: On Bridge, call `lock(token, amount, recipient)`; confirm `Locked` event.
- Mint/Release: From the Bridge owner account, call `mint(token, recipient, amount, lockId)` using the `lockId` from the event.

## Security & Production Notes
- Owner/relayer: Only the owner can call `mint`/`withdraw` in this minimal contract. In production, use a multisig or a permissioned relayer with signatures.
- Proofs: A robust bridge requires cross-chain proofs or trust-minimized messages; this minimal example uses an owner gate.
- Replay protection: `processedLockIds` prevents re-using a `lockId` on mint.
- Accounting: If using "release" semantics (this contract holds tokens), ensure liquidity is pre-loaded on the destination chain as needed.

