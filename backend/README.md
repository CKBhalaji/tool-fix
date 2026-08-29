# ToolFix Backend

The ToolFix backend is a Rust **Cargo workspace**: one process, fifteen
crates, PostgreSQL as the durable source of truth. All Rust/Cargo files live
inside this `backend/` directory — the repository root is never a Cargo
project.

## Quick start

```bash
cd backend
cp .env.example .env          # then fill in secrets/paths
docker compose up postgres -d # from the repo root (or your own PostgreSQL)
cargo run -p toolfix-server
```

The API listens on `http://localhost:8080`:

- `GET /health/live` — process liveness
- `GET /health/ready` — checks the database connection

## Commands

All Cargo commands run from `backend/`:

```bash
cargo build --workspace --locked
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked

# Persistence round-trip tests (require a PostgreSQL database)
TOOLFIX_TEST_DATABASE_URL=postgres://toolfix:toolfix@localhost:5432/toolfix \
  cargo test -p toolfix-persistence --test postgres_roundtrip -- --ignored
```

## Workspace map

Dependency direction (no cycles; HTTP handlers hold no business logic or
SQL):

```text
toolfix-contracts
        ↓
toolfix-domain
        ↓
┌────────┼────────┬──────────────┬─────────┬──────────┐
auth   persistence  location   notifications  storage  payments
└────────┴────────┴──────────────┴─────────┴──────────┘
        ↓
matching / pricing / agent
        ↓
jobs  →  runtime
        ↓
api
        ↓
server  (composition root)
```

| Crate | Responsibility |
|---|---|
| `toolfix-contracts` | Wire DTOs, shared enums, error codes. Bottom of the graph. |
| `toolfix-domain` | Entities and business rules: the job state machine, offer validation, pricing guardrails. Knows no infrastructure. |
| `toolfix-auth` | Backend-only Firebase/Google identity, ToolFix access/refresh tokens, HttpOnly cookies, DB sessions, Axum auth extractor, role checks. |
| `toolfix-persistence` | SQLx/PostgreSQL repositories and models; embedded migrations; refresh tokens stored as hashes only. |
| `toolfix-location` | Haversine math, bounding boxes, ETA estimates, location-event types. |
| `toolfix-notifications` | `NotificationProvider` trait + logging provider (FCM later). |
| `toolfix-storage` | `StorageBackend` trait + local filesystem backend (S3-compatible later). |
| `toolfix-payments` | `PaymentProvider` trait + cash/stub provider (Razorpay later). |
| `toolfix-matching` | Deterministic mechanic matching: bounding-box search, filters, ranking, radius expansion, notification fan-out. No LLM. |
| `toolfix-pricing` | Historical price context, advisory estimate records, AI-estimate ≠ offer ≠ final-price separation. |
| `toolfix-agent` | The agentic layer: `AgentProvider` trait with Gemini-API-key and Vertex-AI implementations, structured validated outputs, diagnosis/price workflows. |
| `toolfix-jobs` | Application service orchestrating the marketplace: breakdown creation, analysis pipeline, offers, transactional acceptance, mechanic actions, payments, ratings, in-process event hub. |
| `toolfix-runtime` | Tokio background loops (offer/job expiry, notification dispatch, location retention) with graceful shutdown. |
| `toolfix-api` | Axum router, handlers, CORS, WebSocket job channels, health/metrics. |
| `toolfix-server` | Composition root: config from environment, database + migrations, service construction, graceful shutdown. |

## Database

PostgreSQL only. Migrations live in `migrations/postgres/` and are applied
automatically at startup. Migration files are **immutable once applied** —
SQLx records checksums, and editing an applied migration fails startup with
`migration N was previously applied but has been modified`. Add a new
numbered migration instead.

## Configuration

Everything comes from the environment (see `.env.example`). Secrets — the
Google OAuth client file, the Firebase service account — live in
`backend/secrets/` (gitignored) and are referenced only by path-valued
environment variables. Never commit, log, or print them.

## Documentation

| Document | Purpose |
|---|---|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Layering, seams, invariants |
| [docs/AUTHENTICATION.md](docs/AUTHENTICATION.md) | **Authoritative** Firebase/Google + cookie session spec |
| [docs/API.md](docs/API.md) | HTTP and WebSocket surface |
| [docs/DATABASE.md](docs/DATABASE.md) | Tables and relationships |
| [docs/AGENTS.md](docs/AGENTS.md) | AI boundaries and safety rules |
| [docs/MATCHING.md](docs/MATCHING.md) | Deterministic matching pipeline |
| [docs/BIDDING.md](docs/BIDDING.md) | Offer lifecycle |
| [docs/LOCATION.md](docs/LOCATION.md) | Tracking and WebSockets |
| [docs/NOTIFICATIONS.md](docs/NOTIFICATIONS.md) | Providers and dispatch |
| [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) | Docker and operations |
