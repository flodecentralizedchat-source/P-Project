Auth and Security Guide

Overview

- JWT (HS256) protects all mutating and sensitive API endpoints.
- CORS is enforced; configure allowed origins via env.
- Request guardrails: per‑IP rate limit, concurrency cap, timeouts, and body size limits.

Roles

- user: default role for regular users
- governance: elevated role for DAO operations (tally/execute)
- admin: full admin for management actions (airdrop/web2/bot)

Environment Variables

- JWT_SECRET: secret for HS256 signing/verification.
- CORS_ALLOWED_ORIGINS: comma‑separated list or "*".
- CORS_ALLOW_CREDENTIALS: true/false.
- MAX_REQUEST_BYTES: request body limit in bytes (default 1 MB).
- RATE_LIMIT_WINDOW_SECS: per‑IP window (default 60).
- RATE_LIMIT_MAX: max requests per IP per window (default 120).

Generate a Dev Token

- Command:
  - cargo run -p p-project-api --bin dev_token -- sub=user123 role=admin hours=24
  - Expects JWT_SECRET to be set.

Test with curl

- Set a TOKEN env var in your shell from the generator output.

- Public endpoints (no auth):
  - curl http://localhost:3000/
  - curl http://localhost:3000/metrics

- Authenticated endpoint:
  - curl -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"username":"alice","wallet_address":"0x..."}' \
    http://localhost:3000/users

- Admin-only endpoint:
  - curl -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"total_amount":1000.0,"recipients":[{"user_id":"u1","amount":100.0}]}' \
    http://localhost:3000/airdrop/create

WhoAmI endpoint

- Inspect JWT claims (requires auth):
  - curl -H "Authorization: Bearer $TOKEN" http://localhost:3000/auth/whoami

Notes

- In production, place Nginx or a trusted proxy in front of the API and ensure it sets X-Forwarded-For so per‑IP limiting is accurate.
- Rotate JWT_SECRET periodically; tokens issued with the old secret become invalid.
 - Admin routes use stricter rate-limits; tune STRICT_RATE_LIMIT_WINDOW_SECS and STRICT_RATE_LIMIT_MAX.
