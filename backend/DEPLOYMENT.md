# ToolFix Backend — Running a Release Bundle

This bundle is a standalone build of the ToolFix backend. No Rust toolchain,
database server, or other install steps are required to run it: everything is
compiled into a single `toolfix-server` binary, with database migrations
embedded inside it (applied automatically at startup).

## What's inside

| File | Purpose |
|---|---|
| `toolfix-server` / `toolfix-server.exe` | The server binary (Linux / Windows). Migrations are embedded. |
| `.env.example` | Configuration template — copy it to `.env` and edit. |
| `README.md` | This file. |

## Requirements

- **Nothing** for local development (default driver is SQLite; `dev.db` is
  created automatically at startup).
- PostgreSQL 14+ only if you switch to `DATABASE_DRIVER=postgres` (the
  database itself is created automatically when missing).

## 1. Configure

Copy `.env.example` to `.env` **in the same directory as the binary** — the
server loads `.env` from its working directory (real environment variables
win over `.env` values).

The minimum for a local run:

```env
DATABASE_URL=sqlite://dev.db?mode=rwc
JWT_ACCESS_SECRET=change-me-change-me-change-me-change-me
```

`JWT_ACCESS_SECRET` must be at least 32 characters. Generate one with
`openssl rand -hex 32` (Git Bash / WSL / Linux) or
`[guid]::NewGuid().ToString("N") * 2` (PowerShell). Everything else has a
sensible default (port `8080`, host `0.0.0.0`, storage dir `./storage`).

To use PostgreSQL instead, set:

```env
DATABASE_DRIVER=postgres
DATABASE_URL=postgres://user:password@localhost:5432/toolfix
```

## 2. Run on Windows

```powershell
# extract the zip, then from the folder containing toolfix-server.exe:
copy .env.example .env    # then edit .env (see above)
.\toolfix-server.exe
```

## 3. Run on Linux

```bash
tar -xzf toolfix-server-*-linux-x86_64.tar.gz
cd toolfix-server-*-linux-x86_64
chmod +x toolfix-server
cp .env.example .env      # then edit .env (see above)
./toolfix-server
```

If port 8080 is taken or you want it reachable from other machines, set
`TOOLFIX_HOST` / `TOOLFIX_PORT` in `.env`. On a server, open the port in the
firewall if needed (`sudo ufw allow 8080` on Ubuntu).

## 4. Verify

The server listens on `http://localhost:8080` (or what you set in `.env`):

- `GET /health/live` — process liveness
- `GET /health/ready` — checks the database connection
- `GET /swagger-ui` — interactive Swagger UI (OpenAPI 3.1)
- `GET /api-docs/openapi.json` — the raw OpenAPI document

Startup log should include `database connected and migrations applied`.

## Notes

- `./storage/` (uploaded files) is created automatically next to where you
  run the binary.
- Admin console defaults (`ADMIN_EMAIL` / `ADMIN_PASSWORD` in `.env.example`)
  must be changed for anything beyond local testing.
- CORS/login redirects assume the frontend at `FRONTEND_ORIGIN`
  (default `http://localhost:3000`).
