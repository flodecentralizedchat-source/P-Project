# P‑Project API – Detailed Guide

This document explains what the `p-project-api` service does and how to run, test, and troubleshoot it locally or via Docker Compose.

## Overview

- Service: Rust HTTP API built with `axum` exposing endpoints for users, token transfers, staking, and airdrop claims.
- Port: `3000`
- Static files: Served from `pkg/` under `/static` (built by the web crate when using Docker).
- Database: Connects to MySQL using `DATABASE_URL` and initializes tables on startup.
- CORS: Permissive (good for dev; review before production).
- Internal crates: Depends on `p-project-core` and `p-project-contracts` for domain logic.

## Prerequisites

Choose one of the following setups.

- Docker Compose (recommended for a quick start)
  - Docker Desktop or Docker Engine + Compose plugin
- Local development (Cargo)
  - Rust toolchain (stable)
  - MySQL 8.x running locally
  - Optional: `curl` or PowerShell for API testing

---

## Option A: Run via Docker Compose

1) Prepare environment file

- Copy example env file at the repo root:
  - PowerShell: `Copy-Item .env.example .env`
  - Bash: `cp .env.example .env`
- The `.env` provides service credentials and URLs used by Compose.
  - `DATABASE_URL` is set to `mysql://pproject:pprojectpassword@mysql:3306/p_project` so the API connects to the MySQL container by hostname `mysql`.

2) Build and start the stack

- From the repo root: `docker compose up --build -d`
- Services started:
  - `mysql` (port 3306), `redis` (port 6379), `mongodb` (port 27017)
  - `api` (p-project-api on port 3000)
  - `nginx` static server (port 8080) serving the built web `pkg/`

3) Verify health and logs

- Check container health: `docker compose ps`
- Tail API logs: `docker compose logs -f api`
- Once ready, the API should print: `Server running on http://localhost:3000`

4) Test the API

- GET root
  - PowerShell: `Invoke-RestMethod http://localhost:3000/`
  - curl: `curl http://localhost:3000/`
- POST create user (persists and returns full user JSON)
  - PowerShell:
    ```powershell
    Invoke-RestMethod -Method Post -Uri http://localhost:3000/users -ContentType "application/json" -Body (@{ username="alice"; wallet_address="0xabc" } | ConvertTo-Json)
    ```
  - curl:
    ```bash
    curl -X POST http://localhost:3000/users \
      -H 'Content-Type: application/json' \
      -d '{"username":"alice","wallet_address":"0xabc"}'
    ```

- PATCH update user (requires at least one field, returns updated user JSON)
  - PowerShell:
    ```powershell
    Invoke-RestMethod -Method Patch -Uri http://localhost:3000/users/<user_id> -ContentType "application/json" -Body (@{ username="bob" } | ConvertTo-Json)
    ```
  - curl:
    ```bash
    curl -X PATCH http://localhost:3000/users/<user_id> \
      -H 'Content-Type: application/json' \
      -d '{"wallet_address":"0x1234567890abcdef1234567890abcdef12345678"}'
    ```

- POST /transfer (move tokens between wallets)
  - curl:
    ```bash
    curl -X POST http://localhost:3000/transfer \
      -H 'Content-Type: application/json' \
      -d '{"from_user_id":"<sender>","to_user_id":"<receiver>","amount":10.0}'
    ```
- POST /stake (lock tokens for a duration)
  - curl:
    ```bash
    curl -X POST http://localhost:3000/stake \
      -H 'Content-Type: application/json' \
      -d '{"user_id":"<user>","amount":50.0,"duration_days":30}'
    ```
- POST /unstake (release a stake position)
  - curl:
    ```bash
    curl -X POST http://localhost:3000/unstake \
      -H 'Content-Type: application/json' \
      -d '{"user_id":"<user>"}'
    ```
- POST /airdrop/claim
  - curl:
    ```bash
    curl -X POST http://localhost:3000/airdrop/claim \
      -H 'Content-Type: application/json' \
      -d '{"airdrop_id":"airdrop1","user_id":"<user>"}'
    ```

- POST /transfer (move tokens)
  - curl:
    ```bash
    curl -X POST http://localhost:3000/transfer \
      -H 'Content-Type: application/json' \
      -d '{"from_user_id":"<sender>","to_user_id":"<receiver>","amount":10.0}'
    ```
- POST /stake (lock tokens for a duration)
  - curl:
    ```bash
    curl -X POST http://localhost:3000/stake \
      -H 'Content-Type: application/json' \
      -d '{"user_id":"<user>","amount":50.0,"duration_days":30}'
    ```
- POST /unstake (release a stake by ID or use first active)
  - curl:
    ```bash
    curl -X POST http://localhost:3000/unstake \
      -H 'Content-Type: application/json' \
      -d '{"user_id":"<user>"}'
    ```
- POST /airdrop/claim (claim allocation)
  - curl:
    ```bash
    curl -X POST http://localhost:3000/airdrop/claim \
      -H 'Content-Type: application/json' \
      -d '{"airdrop_id":"airdrop1","user_id":"<user>"}'
    ```

Notes
- Most endpoints currently return `501 Not Implemented` until business logic is wired in (see Endpoints section). `POST /users` is implemented and persists to MySQL.
- Input validation: 400 if `username` is not 3-32 chars of `[A-Za-z0-9_-]`, or if `wallet_address` is not an Ethereum-style `0x`-prefixed, 42-char hex string.
- `/static` will be available because Docker builds the web crate and copies `pkg/` into the runtime image.

5) Stop and clean up

- Stop: `docker compose down`
- Stop and remove named volumes: `docker compose down -v` (removes MySQL/Mongo data)

---

## Option B: Run locally with Cargo

1) Start MySQL locally

- Ensure a MySQL server (8.x) is running on `127.0.0.1:3306`.
- Create database and user (example):
  ```sql
  CREATE DATABASE p_project CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
  CREATE USER 'pproject'@'%' IDENTIFIED BY 'pprojectpassword';
  GRANT ALL PRIVILEGES ON p_project.* TO 'pproject'@'%';
  FLUSH PRIVILEGES;
  ```
  Adjust host and password policy to match your environment.

2) Set environment variables for the API

- PowerShell:
  ```powershell
  $env:DATABASE_URL = "mysql://pproject:pprojectpassword@127.0.0.1:3306/p_project"
  ```
- Bash:
  ```bash
  export DATABASE_URL="mysql://pproject:pprojectpassword@127.0.0.1:3306/p_project"
  ```

3) Run the API

- From repo root (or the `p-project-api` directory):
  ```bash
  cargo run -p p-project-api
  ```
- On first run, the service initializes required tables.
- You should see: `Server running on http://localhost:3000`.

4) Test the API locally

- GET root:
  - PowerShell: `Invoke-RestMethod http://localhost:3000/`
  - curl: `curl http://localhost:3000/`
- POST create user (persists and returns full user JSON):
  - PowerShell:
    ```powershell
    Invoke-RestMethod -Method Post -Uri http://localhost:3000/users -ContentType "application/json" -Body (@{ username="alice"; wallet_address="0xabc" } | ConvertTo-Json)
    ```
  - curl:
    ```bash
    curl -X POST http://localhost:3000/users \
      -H 'Content-Type: application/json' \
      -d '{"username":"alice","wallet_address":"0xabc"}'
    ```

- PATCH update user (requires at least one field, returns updated user JSON)
  - PowerShell:
    ```powershell
    Invoke-RestMethod -Method Patch -Uri http://localhost:3000/users/<user_id> -ContentType "application/json" -Body (@{ username="bob" } | ConvertTo-Json)
    ```
  - curl:
    ```bash
    curl -X PATCH http://localhost:3000/users/<user_id> \
      -H 'Content-Type: application/json' \
      -d '{"wallet_address":"0x1234567890abcdef1234567890abcdef12345678"}'
    ```

- Notes
- `/static` serves from a local `pkg/` directory. Unless you build the web crate to `pkg/` locally, requesting `/static/...` will return a 500. For API development only, you can ignore `/static`.
- Transfer, staking, unstaking, and airdrop endpoints now persist through MySQL (see Error Responses for validation/duplicate handling).
- Input validation: 400 if `username` is not 3-32 chars of `[A-Za-z0-9_-]`, or if `wallet_address` is not an Ethereum-style `0x`-prefixed, 42-char hex string.

---

## Endpoints (current state)

- `GET /` – Health/info. Returns `"P-Project API Server"`.
- `POST /users` – Create a user (implemented). Persists to MySQL and returns full user JSON with `created_at`.
- `GET /users/:id` – Fetch a user. Returns 200 with user JSON or 404/`{ "error": "not_found" }`.
- `PATCH /users/:id` – Update username and/or wallet. Validates input, persists changes, and returns the updated user JSON or a structured error.
- `POST /transfer` – Token transfer between users (validates amount, updates balances, records the transaction).
- `POST /stake` – Stake tokens for a duration (moves funds from available to staked balance and records the staking entry).
- `POST /unstake` – Release a staking position (reverses balances, marks the stake completed, logs the transaction).
- `POST /airdrop/claim` – Claim an airdrop allocation (marks the recipient as claimed and returns the amount).
- `POST /airdrop/create` – Register a new airdrop and its recipients, returns the new `airdrop_id`.
- `POST /airdrop/batch-claim` – Claim airdrops for multiple users in one request, returns claimed amounts.
- `GET /static/*` – Serves files from `pkg/` (works in Docker build; local requires prebuilt `pkg/`).

Request examples
- Create user
  ```json
  {
    "username": "alice",
    "wallet_address": "0xabc"
  }
  ```

Responses
- Create user (example):
  ```json
  {
    "id": "<generated>",
    "username": "alice",
    "wallet_address": "0xabc",
    "created_at": "2024-01-01T12:34:56"
  }
  ```
- Transfer response (example):
  ```json
  {
    "transaction_id": "<uuid>",
    "from_user_id": "<sender>",
    "to_user_id": "<receiver>",
    "amount": 10.0
  }
  ```
- Stake response uses the `StakingInfo` shape (`start_time`, optional `end_time`, `rewards_earned`).
- Airdrop claim:
  ```json
  {
    "airdrop_id": "airdrop1",
    "user_id": "<user>",
    "amount": 100.0,
    "message": "Airdrop claimed successfully"
  }
  ```
- Batch claim:
  ```json
  {
    "claimed": [
      { "user_id": "alice", "amount": 50.0 },
      { "user_id": "bob", "amount": 75.0 }
    ]
  }
  ```
---
## Configuration Reference

- Environment
  - `DATABASE_URL` - MySQL connection string the API uses.
  - In Docker Compose, `.env` feeds `mysql`, `mongodb`, `redis`, and `api` containers.
- Networking
  - API listens on `0.0.0.0:3000`.
  - CORS is permissive for development.
- Static assets
  - `/static` -> `pkg/`. In Docker builds, `wasm-pack` builds `p-project-web` and copies `pkg/` into the image.

---

## Troubleshooting

- Port conflicts
  - API: `3000`, MySQL: `3306`, Redis: `6379`, MongoDB: `27017`, Nginx: `8080`.
  - Stop conflicting services or change host port mappings in `docker-compose.yml`.
- Database connection errors
  - Verify `DATABASE_URL`. For Docker it should use hostname `mysql`; locally use `127.0.0.1`.
  - Ensure the database `p_project` exists and credentials match.
- Static file 500 errors
  - Ensure `pkg/` exists. In Docker, it's built automatically. Locally, either build `p-project-web` to `pkg/` or ignore `/static` routes during API-only development.
- Containers not healthy
  - `docker compose ps` and `docker compose logs -f <service>` for details.

---

## Useful Commands

- Docker
  - Start: `docker compose up --build -d`
  - Stop: `docker compose down`
  - Full reset: `docker compose down -v`
  - Logs: `docker compose logs -f api`
- Local
  - Run API: `cargo run -p p-project-api`
  - Test (curl): `curl http://localhost:3000/`
  - Test (PowerShell): `Invoke-RestMethod http://localhost:3000/`

## Regression Testing

- SQLx integration tests (requires a `TEST_DATABASE_URL`, default `mysql://pproject:pprojectpassword@localhost/p_project_test`):
  ```bash
  TEST_DATABASE_URL="mysql://pproject:pprojectpassword@localhost/p_project_test" cargo test -p p-project-api tests::transfer_updates_balances
  ```
  These cover the transfer, staking, and airdrop workflows by exercising the new database helpers.
- HTTP harness (requires the API running on `API_BASE_URL`, and the same MySQL from `API_DB_URL`):
  ```bash
  API_BASE_URL=http://localhost:3000 API_DB_URL="mysql://pproject:pprojectpassword@localhost/p_project" cargo run -p p-project-api --bin harness
  ```
  This script drives `/transfer`, `/stake`, `/unstake`, and airdrop endpoints, then prints the responses for manual verification.

---

## Where Things Live

- API source: `p-project-api/src/main.rs`, `p-project-api/src/handlers.rs`, `p-project-api/src/middleware.rs`
- Dockerfile (multi-stage): `Dockerfile`
- Compose stack: `docker-compose.yml`
- Environment examples: `.env.example`

This guide targets development usage. For production hardening, you'll want to lock CORS, configure secrets securely, add authentication in middleware, and implement the placeholder handlers.

---

## Error Responses and Endpoint Notes

- POST `/users`
  - 400 `{ "error": "invalid_username" }` when username fails validation (3-32 chars, `[A-Za-z0-9_-]`).
  - 400 `{ "error": "invalid_wallet_address" }` when wallet fails basic `0x` + 40-hex check.
  - 409 `{ "error": "username_taken" }` when username is already used.
  - 500 `{ "error": "internal_error" }` for other DB failures.

- GET `/users/:id`
  - Implemented. Returns 200 with user JSON if found, or 404 with `{ "error": "not_found" }` if not.

- POST `/transfer`
  - 400 `{ "error": "invalid_amount" }` when the transfer amount isn't positive.
  - 400 `{ "error": "insufficient_balance" }` when the sender lacks funds.
  - 404 `{ "error": "user_not_found" }` when either wallet ID is missing.
  - 500 `{ "error": "internal_error" }` for unexpected SQL failures.

- POST `/stake`
  - 400 `{ "error": "invalid_amount" }` when the amount or duration is invalid.
  - 400 `{ "error": "insufficient_balance" }` when available tokens are insufficient.
  - 500 `{ "error": "internal_error" }` otherwise.

- POST `/unstake`
  - 404 `{ "error": "stake_not_found" }` if no active stake matches the request.
  - 400 `{ "error": "insufficient_balance" }` when the staked balance cannot be released.
  - 500 `{ "error": "internal_error" }` for other DB issues.

- POST `/airdrop/claim`
  - 404 `{ "error": "claim_not_found" }` if the user has no outstanding unclaimed allocation.
  - 500 `{ "error": "internal_error" }` for SQL failures.

- POST `/airdrop/create`
  - 400 `{ "error": "invalid_airdrop" }` when the payload has no recipients or a non-positive total.
  - 400 `{ "error": "amount_mismatch" }` when recipient totals exceed `total_amount`.
  - 500 `{ "error": "internal_error" }` on insert failures.

- POST `/airdrop/batch-claim`
  - 400 `{ "error": "no_user_ids" }` when the request omits recipients.
  - 500 `{ "error": "internal_error" }` for database errors.
