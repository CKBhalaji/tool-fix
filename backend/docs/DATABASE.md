# Database

Two backends are supported and selected by configuration:

- **SQLite** — local development. `DATABASE_DRIVER=sqlite` with
  `DATABASE_URL=sqlite://dev.db?mode=rwc`; the database file and its schema
  are created automatically at startup (no services required).
- **PostgreSQL** — production. `DATABASE_DRIVER=postgres` with a
  `postgres://…` URL; when the target database does not exist it is created
  automatically before migrations are applied.

Migrations live in `backend/migrations/{postgres,sqlite}/` (matching
schemas, dialect-adjusted) and are applied automatically at startup by
`toolfix-persistence`.

**Migration rule:** migration files are immutable once applied — SQLx stores
checksums in `_sqlx_migrations`. Editing an applied migration fails startup
with `migration N was previously applied but has been modified`; add a new
numbered migration instead.

Statuses are stored as `TEXT` with `CHECK` constraints (snake_case contract
enum names). Money is `BIGINT`/`INTEGER` **paise** (INR minor units) with a
currency column. Timestamps are `TIMESTAMPTZ` (Postgres) / ISO-8601 TEXT
(SQLite) and are bound from Rust. List columns (vehicle kinds, repair
categories, symptoms) and JSON payloads are stored as JSON **text** on both
dialects. Geographic columns are lat/lng — **no PostGIS**: bounding-box
pre-filters run in SQL and exact haversine distance is computed in Rust
(`toolfix-location`), keeping all repository SQL dialect-neutral (no
`FOR UPDATE`, `now()`, `::casts`, or server-side arrays).

## Migrations

| File | Contents |
|---|---|
| `0001_users_auth_sessions.sql` | `users`, `auth_sessions` (hashed refresh tokens) |
| `0002_vehicles.sql` | `vehicle_types`, `vehicles` |
| `0003_mechanics.sql` | `mechanics` (+ denormalized capability arrays), `mechanic_services`, `mechanic_availability`, `mechanic_locations` (current position) |
| `0004_breakdowns.sql` | `breakdowns`, `breakdown_media`, `breakdown_diagnoses` (advisory) |
| `0005_assistance_jobs.sql` | `assistance_jobs` (16-state CHECK), `job_status_history` |
| `0006_mechanic_offers.sql` | `mechanic_offers` (UNIQUE(job, mechanic)), `offer_events` |
| `0007_notifications.sql` | `notifications` (intent + delivery record) |
| `0008_location_events.sql` | `location_events` (position history) |
| `0009_agent_runs.sql` | `agent_runs`, `agent_outputs` (AI audit trail), FK from diagnoses |
| `0010_pricing.sql` | `price_estimates` (advisory), `price_history` (realized) |
| `0011_payments.sql` | `payments`, `payment_transactions` |
| `0012_ratings.sql` | `ratings` (UNIQUE per job), `reviews` |
| `0013_audit_logs_indexes.sql` | `audit_logs` + hot-path indexes |

## Key relationships

```text
users 1─* vehicles 1─* breakdowns 1─1 assistance_jobs 1─* job_status_history
users 1─* auth_sessions
assistance_jobs 1─* mechanic_offers ─1 mechanics
assistance_jobs 1─1 payments, ratings
breakdowns 1─1 breakdown_diagnoses, * breakdown_media, agent_runs
price_history ← one row per completed job (learning data)
```

## Notable constraints

- `mechanic_offers`: `UNIQUE (job_id, mechanic_id)` — one bid per mechanic
  per job; partial index on `expires_at WHERE status = 'pending'` drives the
  expiry sweep.
- `assistance_jobs`: partial index over in-flight statuses for scheduler/
  feed queries; `breakdown_id` is UNIQUE (one job per report).
- `ratings`: `UNIQUE (job_id)` — one rating per completed job; the cached
  `mechanics.rating_average` is recomputed on insert.
- `auth_sessions`: `refresh_token_hash UNIQUE`; raw refresh tokens are never
  stored.
- Foreign keys use CASCADE for owned children (media, history, events) and
  RESTRICT/SET NULL where history must survive (jobs → users/vehicles,
  selected_mechanic).
