# Deployment

## Topology (development)

Root `docker-compose.yml`:

| Service | Image | Port |
|---|---|---|
| `postgres` | postgres:16-alpine | 5432 |
| `backend` | built from `backend/Dockerfile` | 8080 |
| `frontend` | built from `frontend/Dockerfile` | 3000 |

```bash
cp backend/.env.example backend/.env    # fill in secrets/paths first
docker compose up --build
```

`backend/.env` is passed to the backend container via `env_file`; compose
overrides `DATABASE_URL` to point at the `postgres` service. Uploaded media
lives in the `toolfix_storage` volume.

## Backend image

Multi-stage `rust:1.90-bookworm` build (dependency-cache friendly) →
`debian:bookworm-slim` runtime, non-root `toolfix` user, embedded migrations,
`/health/live` Docker healthcheck. The image contains no credentials —
secrets are mounted/injected at runtime via environment variables.

## Frontend image

Multi-stage `node:22-alpine` build with corepack pnpm (`--frozen-lockfile`) →
standalone Next.js server (`output: "standalone"`), non-root user, port 3000.

## Process roles

One backend process runs both API and background runtime (expiry sweeps,
notification dispatch, location retention). Background loops exit via a
cancellation token, so `SIGTERM`/`SIGINT` drains cleanly. Because all state
transitions are PostgreSQL transactions with leases-by-locking, running
multiple backend replicas is safe; WebSocket fan-out is per-process and
clients fall back to REST on reconnect.

## Production checklist

- [ ] TLS terminates at a reverse proxy (Nginx/Caddy/ALB) in front of both apps.
- [ ] `COOKIE_SECURE=true`, `COOKIE_SAME_SITE=true`; front/backend origins configured.
- [ ] `FRONTEND_ORIGIN` set to the exact production origin (CORS is an explicit allow-list; never `*` with credentials).
- [ ] Google Cloud Console: production redirect URI registered on the OAuth client.
- [ ] Secrets injected via the platform secret manager; `backend/secrets/` never baked into images.
- [ ] `JWT_ACCESS_SECRET` ≥ 32 random chars, rotated on schedule.
- [ ] `RUST_LOG` tuned; logs shipped off-box. Tokens/secret files are never logged.
- [ ] PostgreSQL backups + migration policy: migrations are immutable; deploy new ones, never edit applied ones.

## Environment variables

See `backend/.env.example` (server, database, cookies, OAuth, Firebase,
AI provider, matching, offers, storage) and `frontend/.env.example`
(`NEXT_PUBLIC_API_URL`, `NEXT_PUBLIC_WS_URL` only — no Firebase values).
