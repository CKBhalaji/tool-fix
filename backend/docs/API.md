# API Reference

Base URL (dev): `http://localhost:8080`. All authenticated routes require the
`access_token` HttpOnly cookie (set by the auth flow). Errors return a JSON
body `{ "code": "...", "message": "..." }` with an appropriate status.

## Health & metrics

```http
GET /health/live    → 200 (no auth)
GET /health/ready   → 200 when the database answers, 503 otherwise
GET /metrics        → JSON counters (extensible)
```

## Authentication

```http
POST /api/v1/auth/google            → { "login_url": "…" }
GET  /api/v1/auth/google/login      → 302 Google consent (+ handshake cookies)
GET  /api/v1/auth/google/callback   → 302 ${FRONTEND_ORIGIN}/auth/callback (+ session cookies)
POST /api/v1/auth/refresh           → 204, rotates cookies
POST /api/v1/auth/logout            → 204, revokes session, clears cookies
GET  /api/v1/auth/me                → { user, mechanic_id }
```

See `AUTHENTICATION.md` for the full contract.

## Users & onboarding

```http
POST /api/v1/users/onboarding   { requested_role, phone?, display_name? } → UserResponse
GET  /api/v1/users/me                                                     → UserResponse
PATCH /api/v1/users/me         { display_name?, phone? }                 → UserResponse
```

Roles are decided server-side; `admin` requests are rejected.

## Vehicles

```http
POST   /api/v1/vehicles         { vehicle_kind, make?, model?, year?, registration_number? }
GET    /api/v1/vehicles
GET    /api/v1/vehicles/{vehicle_id}
PATCH  /api/v1/vehicles/{vehicle_id}
DELETE /api/v1/vehicles/{vehicle_id}
```

## Breakdowns

```http
POST /api/v1/breakdowns
     { vehicle_id, latitude, longitude, address?, problem_description, vehicle_symptoms[] }
     → 202 { job }  (AI analysis + matching run in the background)

GET  /api/v1/breakdowns/{id}                  → BreakdownDetail (job + diagnosis + estimate + media)
POST /api/v1/breakdowns/{id}/media            → multipart upload (fields: photo | video)
GET  /api/v1/breakdowns/{id}/media/{media_id}/content
GET  /api/v1/breakdowns/{id}/diagnosis        → advisory AI output + estimate
POST /api/v1/breakdowns/{id}/cancel
```

## Mechanics

```http
GET  /api/v1/mechanics/me
PATCH /api/v1/mechanics/me                    { service_area_km?, supported_vehicle_kinds?, repair_categories?, ... }
POST /api/v1/mechanics/onboarding             { supported_vehicle_kinds[], repair_categories[], service_area_km?, ... }
POST /api/v1/mechanics/availability           { status: online | busy | offline }
POST /api/v1/mechanics/location               { latitude, longitude, accuracy_m?, job_id? }
GET  /api/v1/mechanics/requests?latitude=..&longitude=..   → nearby open jobs feed
```

## Jobs & lifecycle

```http
GET  /api/v1/jobs                              → caller's jobs (customer or mechanic view)
GET  /api/v1/jobs/{id}
GET  /api/v1/jobs/{id}/status-history
POST /api/v1/jobs/{id}/start-travel            (selected mechanic)
POST /api/v1/jobs/{id}/arrived                 (selected mechanic)
POST /api/v1/jobs/{id}/start-repair            (selected mechanic)
POST /api/v1/jobs/{id}/complete                (selected mechanic)
POST /api/v1/jobs/{id}/pay                     { method: cash }  (customer; stub provider)
POST /api/v1/jobs/{id}/location                (mechanic live ping during a job)
```

State transitions are validated by the domain state machine — arbitrary
status updates are impossible from the client.

## Offers (bidding)

```http
POST /api/v1/jobs/{job_id}/offers    { quoted_price_minor, estimated_arrival_minutes, message? }
GET  /api/v1/jobs/{job_id}/offers    (owner; joined with mechanic rating/completed jobs)
POST /api/v1/offers/{offer_id}/select    (customer; transactional acceptance)
POST /api/v1/offers/{offer_id}/withdraw  (mechanic)
```

Prices are integer **paise** (INR minor units). Acceptance atomically accepts
one offer, expires competitors, and advances the job state.

## Ratings & reviews

```http
POST /api/v1/ratings                             { job_id, score 1..5, comment? }
GET  /api/v1/mechanics/{mechanic_id}/reviews
```

## Notifications

```http
GET /api/v1/notifications   → caller's in-app notification inbox
```

## Agent (advisory)

```http
POST /api/v1/agent/diagnose
     { description, vehicle_kind?, vehicle_display?, symptoms[], image_base64?, image_mime? }
     → AgentDiagnosisDto      (nothing is persisted by this endpoint)
```

## Admin console

```http
POST /api/v1/admin/login                        { email, password }  (ADMIN_EMAIL/ADMIN_PASSWORD)
GET  /api/v1/admin/overview                     -> platform counters
GET  /api/v1/admin/users?role=&limit=           -> all users
POST /api/v1/admin/users/{user_id}/status       { status: active|suspended|deleted }
GET  /api/v1/admin/mechanics                    -> all mechanic profiles
POST /api/v1/admin/mechanics/{id}/verify        { verified: bool }
GET  /api/v1/admin/jobs?limit=                  -> all jobs (joined customer/mechanic)
```

Login issues the standard HttpOnly session cookies with the ADMIN role;
every other admin route requires that role. Rotate `ADMIN_PASSWORD` in
production.

## WebSocket

```http
GET /api/v1/ws/jobs/{job_id}   (cookie-authenticated upgrade)
```

Frames are JSON `JobEvent`s: `{ job_id, kind, payload, at }` with kinds such
as `job.created`, `job.diagnosis_ready`, `job.mechanics_notified`,
`offer.created`, `offer.accepted`, `mechanic.location`, `job.en_route`,
`job.arrived`, `job.repair_started`, `job.repair_completed`,
`job.completed`, `job.cancelled`, `job.no_mechanic`. The socket is transport
only — reconnecting clients re-read state via REST.
