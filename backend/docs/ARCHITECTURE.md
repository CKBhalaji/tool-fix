# Architecture

## Process model

One Rust process (`toolfix-server`) runs the Axum HTTP/WebSocket API, the
application services, and the background runtime loops. PostgreSQL is the
durable queue and source of truth; process memory (broadcast channels,
caches) is never authoritative. There are no external brokers.

```text
                ┌──────────────────────┐
                │     Next.js App      │
                │  Tailwind UI (pnpm)  │
                └──────────┬───────────┘
                 HTTP + WebSocket (cookies)
                           ▼
                ┌──────────────────────┐
                │      Axum API        │
                │   toolfix-api        │
                └──────────┬───────────┘
          ┌────────────────┼─────────────────┐
          ▼                ▼                 ▼
    ┌───────────┐   ┌─────────────┐   ┌────────────┐
    │ Job layer │   │ Matching    │   │ Agent layer│
    │ toolfix-  │   │ toolfix-    │   │ toolfix-   │
    │ jobs      │   │ matching    │   │ agent      │
    └─────┬─────┘   └──────┬──────┘   └─────┬──────┘
          └────────────────┼────────────────┘
                           ▼
                ┌──────────────────────┐
                │      PostgreSQL      │
                │  durable truth       │
                └──────────┬───────────┘
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                  ▼
  Notifications        Payments          Object storage
  (provider trait)    (provider trait)   (storage trait)
```

## Crate layering

```text
toolfix-contracts          wire DTOs, shared enums (serde only)
        ↓
toolfix-domain             entities, state machine, business validation
        ↓
toolfix-auth      toolfix-persistence     toolfix-location     toolfix-notifications
     (sessions)        (SQLx/Postgres)        (geo math)           (provider trait)
toolfix-storage   toolfix-payments
        ↓
toolfix-matching   toolfix-pricing   toolfix-agent
        ↓
toolfix-jobs                 application orchestration + event hub
        ↓
toolfix-runtime              background loops (expiry, dispatch, cleanup)
        ↓
toolfix-api                  Axum router, handlers, WebSocket, CORS
        ↓
toolfix-server               composition root (config, DI, graceful shutdown)
```

Invariants:

- No circular dependencies; the layering above is enforced by `Cargo.toml`.
- `toolfix-domain` knows nothing about Axum, cookies, Firebase, or SQL.
- Handlers translate HTTP ↔ DTOs only; no SQL and no business rules in
  `toolfix-api`.
- All SQL lives in `toolfix-persistence`; status strings are converted to
  contract enums at that boundary.

## Request flows

**Emergency request:** `POST /api/v1/breakdowns` → breakdown + job rows
(state `created`) → background pipeline in `toolfix-jobs`:
`analyzing` (agent diagnosis, validated + audited) → `mechanics_searching`
(deterministic matching) → `mechanics_notified` (notification fan-out, offer
window set) or `no_mechanic_available`.

**Bidding:** mechanics `POST /api/v1/jobs/{id}/offers` (window + state
checked) → first offer moves the job to `offers_received`. Customer
`POST /api/v1/offers/{id}/select` runs **one transaction**: accept the offer,
expire competitors, advance the state machine, write history + events.

**Execution:** mechanic transitions `mechanic_en_route → mechanic_arrived →
repair_in_progress → repair_completed` (state-machine guarded) → customer
pays (`payment_pending → completed`, final price recorded to `price_history`,
mechanic stats bumped).

## Deterministic vs agentic

| Deterministic (plain code) | Agentic (`toolfix-agent`, advisory) |
|---|---|
| Authentication/authorization | Problem understanding |
| Distance, availability, matching | Image interpretation |
| Offers, state machine, payments | Fault/repair classification |
| Database transactions | Price *recommendation* |

The agent never mutates marketplace state directly: its raw output must pass
schema validation and business rules, and is recorded in
`agent_runs`/`agent_outputs` for auditability.

## Background runtime

`toolfix-runtime` owns Tokio loops — offer/job expiry, notification
dispatch, location retention — all driven by a `CancellationToken` so the
server drains cleanly on SIGINT/SIGTERM. Recurring work never lives in HTTP
handlers.

## Observability

`tracing` spans/logs throughout (`RUST_LOG=info`, JSON-capable),
`/health/live`, `/health/ready` (DB probe), `/metrics` (extensible).
