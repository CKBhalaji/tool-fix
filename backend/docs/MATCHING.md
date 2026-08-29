# Matching

`toolfix-matching` finds suitable mechanics for a breakdown using **only
deterministic business logic** — no LLM is involved in proximity, availability,
or ranking.

## Pipeline

```text
breakdown location
      ↓  bounding-box candidate query (lat/lng indexes)
      ↓  exact haversine distance per candidate (SQL)
      ↓  availability filter        (online/busy, verified)
      ↓  service-area filter        (candidate distance ≤ mechanic's area)
      ↓  vehicle compatibility      (motorcycle/scooter/car/…)
      ↓  repair capability          (category from AI diagnosis, when known)
      ↓  active-job cap             (max concurrently attending jobs)
      ↓  deterministic ranking
      ↓  radius expansion (0–3 km → 3–5 km → 5–10 km) until candidates exist
      ↓  truncate to MAX_MECHANICS_NOTIFIED
      ↓  notification fan-out
```

## Ranking

`rank_score` (constant weights, unit-tested):

```text
0.55 × proximity(1 / (1 + km)) + 0.25 × rating(0..1)
+ 0.10 × experience(min(jobs,500)/500) + 0.10 × availability(1/(1+active))
```

Ties break by mechanic id — behavior is fully deterministic and repeatable.

## Configuration (environment)

| Variable | Default | Meaning |
|---|---|---|
| `MATCHING_RADIUS_STAGE1_M` | 3000 | First radius stage |
| `MATCHING_RADIUS_STAGE2_M` | 5000 | Second stage |
| `MATCHING_RADIUS_STAGE3_M` | 10000 | Third stage (also feed radius) |
| `MAX_MECHANICS_NOTIFIED` | 10 | Cap on notified mechanics per job |
| `MAX_ACTIVE_JOBS_PER_MECHANIC` | 3 | Load cap during matching |
| `MATCHING_AVG_SPEED_KMH` | 25 | ETA estimate speed |
| `FEED_LOOKBACK_SECONDS` | 21600 | Mechanic feed lookback window |

## Feed

`GET /api/v1/mechanics/requests` returns open jobs (`mechanics_notified` /
`offers_received`) within the mechanic's service area, enriched with vehicle
kind, the AI diagnosis, the advisory estimate, and distance. Notifying fewer
mechanics than configured is fine — the expanding radius stops at the first
stage that yields candidates.
