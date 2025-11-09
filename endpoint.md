# P-Project API Endpoints

| Method | Path | Purpose | Status |
| --- | --- | --- | --- |
| `GET` | `/` | Health check / server info | ✅ Implemented |
| `POST` | `/users` | Create a new user (validates username/wallet, persists to MySQL) | ✅ Implemented |
| `GET` | `/users/:id` | Fetch a user by ID from MySQL | ✅ Implemented |
| `PATCH` | `/users/:id` | Update username/wallet with validation | ✅ Implemented |
| `POST` | `/transfer` | Transfer tokens between users | ⚪ Placeholder (not implemented) |
| `POST` | `/stake` | Stake tokens for a user | ⚪ Placeholder |
| `POST` | `/unstake` | Unstake previously staked tokens | ⚪ Placeholder |
| `POST` | `/airdrop/claim` | Claim an airdrop allocation | ⚪ Placeholder |
| `POST` | `/airdrop/create` | Create a new airdrop campaign | ⚪ Placeholder |
| `POST` | `/airdrop/batch-claim` | Claim airdrops for multiple users | ⚪ Placeholder |
| `GET` | `/static/*` | Serve static files (`pkg/` assets) | ✅ Implemented |

> **Next steps:** flesh out the remaining token/airdrop endpoints with database plumbing, business rules, and structured error handling before full QA.
