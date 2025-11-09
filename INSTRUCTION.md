# P-Project Workspace Boot Instructions

These steps cover the minimum environment, sequence, and commands to start the core bridge/API workflow.

## 1. Prepare shared services
- **MySQL**: start a MySQL instance and create (or use) a schema, e.g. `p_project`. Make sure your user has rights to create tables.
- **Environment variables** (PowerShell example):
  ```powershell
  $env:DATABASE_URL="mysql://user:password@localhost/p_project"
  $env:ETH_RPC_URL="https://sepolia.infura.io/v3/<project-id>"
  $env:ETH_BRIDGE_ADDRESS="0xYourBridgeAddress"
  $env:ETH_TOKEN_ADDRESS="0xYourTokenAddress"
  $env:ETH_PRIVATE_KEY="0xYourRelayerPrivateKey"
  $env:ETH_CONFIRMATIONS="3"
  ```
  Adjust values per your deployment/chain. `DATABASE_URL` is mandatory for both the API and bridge.

## 2. Run the API server (optional but usually next)
1. From the workspace root run:
   ```bash
   cargo run -p p-project-api
   ```
2. This boots the Axum-based API on `http://localhost:3001`. It automatically calls `MySqlDatabase::init_tables()` so the schema (users, airdrops, `bridge_txs`, etc.) exists.

## 3. Start the bridge relayer
1. In another shell, run:
   ```bash
   cargo run -p p-project-bridge --bin relayer_demo
   ```
2. This process polls `bridge_txs`, waits for Ethereum confirmations (via `ETH_RPC_URL`), and calls destination adapters to mint tokens. Logs show when locks are read and when a mint is performed (`[Relayer] ...`).

## 4. Simulate without real chains (optional)
Use this when you just want to confirm the logic:
```
cargo run -p p-project-bridge --bin simulate_bridge
```
It runs the `BridgeService` with `MockStore` and `MockAdapter`s, triggers a `bridge_tokens` request, and prints the state after the relayer runs once.

## 5. Triggering bridge flows
- Hit the API (`/bridge` endpoint) or call `BridgeService::bridge_tokens` from integration scripts to create entries in `bridge_txs`. The relayer then sees them in order: `Pending` → `Locked` (after `lock`) → `Minted` (after relayer run).
- You can view rows via MySQL to confirm `lock_id`, `src_tx_hash`, `dst_tx_hash`, and status.

## Summary
| Step | Command | Purpose |
|------|---------|---------|
| 1 | Set env (see above) | Connect to MySQL + Ethereum RPC/bridge/token |
| 2 | `cargo run -p p-project-api` | Run HTTP API + init tables |
| 3 | `cargo run -p p-project-bridge --bin relayer_demo` | Turn on relayer that finalizes `Locked` rows |
| 4 | `cargo run -p p-project-bridge --bin simulate_bridge` | Quick in-memory sanity check |

Keep the database and bridge relayer shells running as you iterate. Restart either process if you change env vars or redeploy contracts.
