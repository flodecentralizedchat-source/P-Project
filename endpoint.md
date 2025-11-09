# P-Project API Endpoints

| Method | Path | Purpose | Status |
| --- | --- | --- | --- |
| `GET` | `/` | Health check / server info | ✅ Implemented |
| `POST` | `/users` | Create a new user (validates username/wallet, persists to MySQL) | ✅ Implemented |
| `GET` | `/users/:id` | Fetch a user by ID from MySQL | ✅ Implemented |
| `PATCH` | `/users/:id` | Update username/wallet with validation | ✅ Implemented |
| `POST` | `/transfer` | Transfer tokens between users | ✅ Implemented |
| `POST` | `/stake` | Stake tokens for a user | ✅ Implemented |
| `POST` | `/unstake` | Unstake previously staked tokens | ✅ Implemented |
| `POST` | `/airdrop/claim` | Claim an airdrop allocation | ✅ Implemented |
| `POST` | `/airdrop/create` | Create a new airdrop campaign | ✅ Implemented |
| `POST` | `/airdrop/batch-claim` | Claim airdrops for multiple users | ✅ Implemented |
| `GET` | `/static/*` | Serve static files (`pkg/` assets) | ✅ Implemented |

> All endpoints now include validation, persistence, and structured error responses; see `api.md` for usage notes and examples.
