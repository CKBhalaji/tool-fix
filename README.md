# ToolFix — On-Demand Roadside Assistance Platform

ToolFix connects stranded drivers with nearby mechanics. A customer reports a
breakdown (location, vehicle, description, photo/video), an agentic AI layer
analyzes the problem and estimates a fair cost range, nearby available
mechanics are notified, and competing mechanics submit offers the customer
compares and chooses from — followed by live tracking, repair, payment, and
rating.

## Repository layout

```text
toolfix/
├── backend/     Rust / Cargo workspace (Axum API, PostgreSQL, agentic layer)
├── frontend/    Next.js + TypeScript + Tailwind CSS app (pnpm)
├── docs/        Project-level documentation index
└── docker-compose.yml
```

The two applications are fully separate:

- **Everything Rust/Cargo lives in `backend/`** — the root is not a Cargo project.
- **Everything Next.js lives in `frontend/`** — managed with **pnpm**, never npm.
- **Firebase is a backend-only dependency.** The frontend has no Firebase SDK,
  configuration, or environment variables; it only talks to the ToolFix API.

## Quick start

### Backend

All Cargo commands run from `backend/`. See `backend/README.md`.

```bash
cd backend
cp .env.example .env            # then fill in secrets/paths
docker compose up postgres -d   # from repo root, or your own PostgreSQL
cargo run -p toolfix-server
```

The API listens on `http://localhost:8080` (`/health/live`, `/health/ready`).

### Frontend

All frontend commands run from `frontend/`. See `frontend/README.md`.

```bash
cd frontend
pnpm install
pnpm dev
```

The app listens on `http://localhost:3000`.

## Documentation

| Document | Purpose |
|---|---|
| [backend/README.md](backend/README.md) | Workspace, crates, commands |
| [frontend/README.md](frontend/README.md) | Frontend setup and structure |
| [backend/docs/ARCHITECTURE.md](backend/docs/ARCHITECTURE.md) | Layering and dependency direction |
| [backend/docs/AUTHENTICATION.md](backend/docs/AUTHENTICATION.md) | Authoritative Firebase/Google + cookie session spec |
| [backend/docs/API.md](backend/docs/API.md) | HTTP/WS surface |
| [backend/docs/DATABASE.md](backend/docs/DATABASE.md) | Schema and migrations |
| [backend/docs/AGENTS.md](backend/docs/AGENTS.md) | Agentic layer boundaries and safety rules |
| [backend/docs/MATCHING.md](backend/docs/MATCHING.md) | Deterministic mechanic matching |
| [backend/docs/BIDDING.md](backend/docs/BIDDING.md) | Offer lifecycle |
| [backend/docs/LOCATION.md](backend/docs/LOCATION.md) | Location tracking and WebSockets |
| [backend/docs/NOTIFICATIONS.md](backend/docs/NOTIFICATIONS.md) | Notification providers |
| [backend/docs/DEPLOYMENT.md](backend/docs/DEPLOYMENT.md) | Docker and deployment |

## Core principles

1. **Deterministic core** — authentication, authorization, job state, matching,
   offers, and payments are normal, testable business logic.
2. **Advisory AI** — the agentic layer only classifies/estimates; its output is
   schema-validated and applied through business rules. It never charges,
   approves, or mutates state directly.
3. **PostgreSQL is the source of truth** — WebSocket connections are transport,
   never state.
4. **Tokens live only in HttpOnly cookies** — never in localStorage, URLs, or
   logs; refresh tokens are stored server-side as hashes only.
