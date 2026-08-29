# Bidding (Offers)

Mechanics compete for a breakdown by submitting offers; the customer compares
and chooses. **The system never auto-selects the cheapest (or any) mechanic.**

## Offer model

```json
{
  "mechanic_id": "…",
  "job_id": "…",
  "quoted_price_minor": 65000,
  "estimated_arrival_minutes": 15,
  "message": "Can reach in 15 minutes",
  "created_at": "…",
  "expires_at": "…"
}
```

Prices are integer paise. Validation bounds: ₹50 … ₹5,00,000, ETA 1–600
minutes, message ≤ 1000 chars.

## Lifecycle

```text
mechanic opens request (feed: location, vehicle, issue, photos, AI estimate)
        ↓
POST /api/v1/jobs/{job_id}/offers        (offer window open + state checked)
        ↓   status: pending               — first offer moves job → offers_received
customer compares offers                  — price, rating, ETA, distance, experience
        ↓
POST /api/v1/offers/{offer_id}/select     (customer)
        ↓  TRANSACTION:
        ├── lock job row (FOR UPDATE), verify owner + state
        ├── mark chosen offer accepted
        ├── expire all competing pending offers
        ├── advance job → mechanic_selected (+ job_status_history)
        ├── offer_events audit row
        └── commit
        ↓
POST /api/v1/offers/{offer_id}/withdraw   (mechanic, while pending)
   or expiry sweep (runtime) → status: expired
```

## Rules

- One offer per mechanic per job (`UNIQUE(job_id, mechanic_id)`).
- Offers are only accepted while the job is `mechanics_notified`/
  `offers_received` **and** the offer window (`OFFER_EXPIRY_SECONDS`,
  default 15 min) is open.
- When all offers expire and the window closes, the job transitions to
  `no_mechanic_available` (handled by `toolfix-runtime` + `toolfix-jobs`).
- Acceptance is atomic — partially accepted offers are impossible.
- Selection is always the customer's decision; the AI estimate is display-only.
