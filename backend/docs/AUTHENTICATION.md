# ToolFix Authentication Architecture

**This document is the authoritative specification** for authentication and
session management. The guiding rule:

> **Firebase is a backend-only dependency. The frontend must have no
> knowledge of Firebase.**

The frontend contains no Firebase SDK, no Firebase configuration, no
`NEXT_PUBLIC_FIREBASE_*` variables, and never touches identity tokens. It
knows only: *ToolFix API, ToolFix session, ToolFix user, ToolFix roles.*

---

## 1. High-level architecture

```text
┌──────────────────────────┐
│      Next.js Frontend    │
│   "Continue with Google" │
└────────────┬─────────────┘
             │ navigation (no SDK, no tokens in JS)
             ▼
┌──────────────────────────┐
│       Rust / Axum API    │
│  ToolFix authentication  │
│  (toolfix-auth crate)    │
└────────────┬─────────────┘
             │ server-to-server
             ▼
┌──────────────────────────┐
│  Google OAuth 2.0        │
│  + Firebase Identity     │
│    Toolkit (provisioning)│
└──────────────────────────┘
```

After verification, the backend issues **its own** tokens:

```text
Google identity → ToolFix user → ToolFix session
                                     ├─ access_token  (HttpOnly cookie)
                                     └─ refresh_token (HttpOnly cookie)
```

## 2. Flow — "Continue with Google"

Because the browser must never handle identity tokens, the Google flow is a
**server-side OAuth 2.0 authorization-code exchange** with PKCE:

1. `GET /api/v1/auth/google/login` — the backend builds Google's consent URL
   (`state` + `code_challenge`), stores `state` and the PKCE `verifier` in
   short-lived HttpOnly cookies, and 302s the browser to Google.
2. Google redirects to `GET /api/v1/auth/google/callback?code=…&state=…`.
3. The backend verifies `state` against the cookie, exchanges the code
   **server-to-server**, and verifies the returned ID token against
   Google's JWKS (issuer + audience).
4. The backend provisions/links the Firebase user via the Identity Toolkit
   REST call `accounts:signInWithIdp` (backend-only, using the Firebase Web
   API key). In development, if `FIREBASE_WEB_API_KEY` is not configured,
   provisioning is skipped with a warning and the verified Google identity
   is used directly (uid `google:<sub>`).
5. The ToolFix user is found or created (`users` table, role defaults to
   `customer`), an `auth_sessions` row is created, and the access/refresh
   cookies are set.
6. The browser is redirected to `${FRONTEND_ORIGIN}/auth/callback`, which
   calls `GET /api/v1/auth/me` and routes by role/onboarding state.

`POST /api/v1/auth/google` additionally exists and returns the consent URL
as JSON for programmatic initiation.

### One-time operator setup

1. In Google Cloud Console → the OAuth client → *Authorized redirect URIs*,
   add `http://localhost:8080/api/v1/auth/google/callback` (and the
   production equivalent).
2. Put the Firebase **Web API key** in `backend/.env` as
   `FIREBASE_WEB_API_KEY` (backend-only value).

## 3. Tokens

| Token | Lifetime | Purpose | Storage |
|---|---|---|---|
| ToolFix access token | 15 min (configurable) | Authenticates API requests | `access_token` HttpOnly cookie, `Path=/` |
| ToolFix refresh token | 30 days (configurable) | Obtains new access tokens | `refresh_token` HttpOnly cookie, `Path=/api/v1/auth` |

- Access tokens are HS256 JWTs with claims `sub` (ToolFix user id), `role`,
  `iat`, `exp`, `iss`, `aud`. Minimal personal data.
- Refresh tokens are random 256-bit opaque values. **The database stores only
  their SHA-256 hash** (`auth_sessions.refresh_token_hash`); the raw value is
  never persisted, logged, or returned in JSON bodies.

### Cookie rules

- `HttpOnly` — JavaScript cannot read the cookies.
- `Secure` — required in production (`COOKIE_SECURE=true`); disabled for
  local development only.
- `SameSite=Lax` (`COOKIE_SAME_SITE=true`). SameSite plus explicit-origin
  CORS is the CSRF baseline for state-changing requests; add Origin
  validation or tokens if the deployment topology demands more.
- Logout clears cookies with the same attributes used to set them.

## 4. Endpoints

```http
POST /api/v1/auth/google          → { "login_url": "…" }
GET  /api/v1/auth/google/login    → 302 to Google consent
GET  /api/v1/auth/google/callback → 302 to frontend, sets cookies
POST /api/v1/auth/refresh         → rotates tokens, sets new cookies
POST /api/v1/auth/logout          → revokes session, clears cookies
GET  /api/v1/auth/me              → authenticated user profile
```

## 5. Rotation and theft detection

`POST /api/v1/auth/refresh` reads the refresh cookie, looks up the session
by hash, then:

- If the session is already **revoked** → the token is being replayed.
  Every session of that user is revoked (reuse detection) and the request
  is rejected.
- If the session is **expired** → revoked, rejected.
- Otherwise → the session is rotated to a new hash, a new refresh token is
  set as a cookie, and a fresh access token is issued.

The frontend's `services/api.ts` performs a single-flight transparent
refresh on any `401` and retries the original request once.

## 6. Roles and authorization

- Roles: `CUSTOMER`, `MECHANIC`, `ADMIN` (stored server-side in `users.role`).
- The role in the access token is minted by the backend from database
  state — **a role sent from the browser is never trusted**. Onboarding
  (`POST /api/v1/users/onboarding`) may request CUSTOMER or MECHANIC;
  ADMIN is never granted through the API.
- Protected handlers take the `AuthUser` extractor, which resolves identity
  exclusively from the access-token cookie. Object ownership is enforced
  per-request (customers see only their jobs; mechanics act only on jobs
  they were selected for).

## 7. Session database model

```text
auth_sessions
──────────────
id                  UUID PK
user_id             UUID → users(id)
refresh_token_hash  TEXT UNIQUE   -- hash only, never the raw token
created_at          TIMESTAMPTZ
expires_at          TIMESTAMPTZ
last_used_at        TIMESTAMPTZ
revoked_at          TIMESTAMPTZ   -- rotation/revocation marker
user_agent          TEXT
ip_address          TEXT
```

## 8. Mandatory security rules

```text
Firebase SDK / config / env vars  → backend only
Google client secret              → backend only (backend/secrets/, gitignored)
Firebase service account          → backend only (backend/secrets/, gitignored)
ToolFix access token              → HttpOnly cookie only
ToolFix refresh token             → HttpOnly cookie only
Refresh token at rest             → SHA-256 hash only
Tokens in localStorage/sessionStorage/URLs/logs  → NEVER
Raw refresh token in JSON responses → NEVER
CORS                              → explicit origins, never `*` with credentials
Role from the browser             → NEVER trusted
```

## 9. What the two systems are responsible for

| Google / Firebase (backend-only) | ToolFix Rust backend |
|---|---|
| Google identity of the human | Application user (`users` table) |
| Federated credential verification | Application roles + authorization |
| Firebase user registry (`signInWithIdp`) | Application session, token mint/rotation, protected APIs |

Authentication and onboarding are separate concerns: signing in proves
identity; choosing CUSTOMER vs MECHANIC (and the mechanic's service profile)
is application onboarding.
