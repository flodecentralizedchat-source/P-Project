# Credit & Micro-Finance

This guide explains how to use the Credit & Micro-Loans features:

- **P-Coin-collateral micro-loans** backed by NGO partners
- **Credit scoring** that rewards verified social impact
- **NGO-backed microfinance** for upliftment projects

## Environment Variables

Configure the service via environment variables in `.env` or your deployment manifest:

- `CREDIT_MIN_SCORE` – Minimum credit score required to request a loan (default `60`)
- `CREDIT_MAX_LOAN_AMOUNT` – Global cap on individual loan amounts (default `1000`)
- `CREDIT_COLLATERAL_RATIO` – Minimum collateral / principal ratio (default `0.5`)
- `CREDIT_INTEREST_RATE` – Fixed interest rate applied to all micro-loans (default `0.08`)
- `CREDIT_BASE_SCORE` – Starting score for every user (default `50`)
- `CREDIT_MAX_DURATION_DAYS` – Loan duration in days (default `30`)

## 1. Register an NGO partner (admin only)

```
POST /credit/register-ngo
Authorization: Bearer <admin_jwt>
{
  "config": {
    "name": "Hope microfinance",
    "region": "East Africa",
    "max_loan_amount": 750.0,
    "approved": true
  }
}
```

The response contains the `ngo.id` used to underwrite future loans.

## 2. Boost credit score with social impact

```
POST /credit/add-impact-event
Authorization: Bearer <jwt>
{
  "user_id": "user123",
  "event": {
    "id": "event_abc",
    "description": "Volunteered at clean water drive",
    "impact_score": 12.5,
    "verified_by": "Hope microfinance",
    "timestamp": "2025-04-01T15:23:00"
  }
}
```

Each verified event increases the user's credit score. The API returns the updated score.

## 3. Request a micro-loan

```
POST /credit/request-loan
Authorization: Bearer <jwt>
{
  "borrower_id": "user123",
  "amount": 250.0,
  "collateral_amount": 200.0,
  "ngo_id": "<ngo_id>"
}
```

Loan requests verify:

- Credit score >= `CREDIT_MIN_SCORE`
- Requested amount <= NGO and service limits
- Collateral >= `collateral_ratio * amount`

Successful responses include the `microloan` payload and current score.

## 4. Repay a micro-loan

```
POST /credit/repay-loan
Authorization: Bearer <jwt>
{
  "loan_id": "loan_xyz",
  "amount": 275.0
}
```

Payments exceeding the total due mark the loan as `Repaid` and return the adjusted loan status.

## 5. Inspect loans & scores

- `GET /credit/loan/:loan_id` – fetch loan metadata (status, due date, collateral).
- `GET /credit/score/:user_id` – retrieve the latest credit score for a borrower.

## Notes

- The current implementation stores data in-memory for illustration; swap in persistent storage for production.
- Credit scores are kept between 0–100 and derived from the configured `base_score` plus all impact events.
