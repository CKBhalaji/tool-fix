# ToolFix Frontend

Next.js (App Router) + TypeScript + Tailwind CSS, managed with **pnpm**.
The frontend is a fully independent application: it talks to the Rust
backend over HTTP/WebSocket and contains **no Firebase SDK, configuration,
or environment variables** — authentication is performed entirely by the
backend (see `backend/docs/AUTHENTICATION.md`).

## Commands

All frontend commands run from `frontend/`:

```bash
pnpm install
pnpm dev      # http://localhost:3000
pnpm build
pnpm lint
```

## Environment

Copy `.env.example` to `.env.local`. Only public, frontend-safe values are
allowed here:

```env
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080
```

Never add backend secrets or `NEXT_PUBLIC_FIREBASE_*` variables.

## Structure

```text
frontend/
├── app/
│   ├── page.tsx                 # landing / auth router
│   ├── login/                   # "Continue with Google" (redirect to backend)
│   ├── register/                # role onboarding (CUSTOMER or MECHANIC)
│   ├── auth/callback/           # post-login landing target
│   ├── customer/
│   │   ├── dashboard/           # vehicles + assistance history
│   │   ├── request/             # emergency flow: map, vehicle, description, photo
│   │   ├── offers/              # compare competing mechanic offers
│   │   └── tracking/            # live status, AI estimate, mechanic position
│   ├── mechanic/
│   │   ├── dashboard/           # online toggle + service profile
│   │   ├── requests/            # nearby breakdown feed + bid form
│   │   ├── jobs/                # accepted jobs: travel → arrive → repair → complete
│   │   └── earnings/            # completed-job summary
│   └── admin/                   # placeholder (admin phase)
├── components/
│   ├── map/MapView.tsx          # vendor-abstracted map (Leaflet/OSM today)
│   ├── offers/OfferCard.tsx
│   ├── layout/Header.tsx
│   └── ui/                      # buttons, cards, status badges
├── hooks/
│   ├── useAuth.tsx              # session context (reads /auth/me; no tokens in JS)
│   └── useGeolocation.ts        # one-shot + watch positioning
├── services/
│   ├── api.ts                   # fetch wrapper: credentials:"include", auto-refresh
│   ├── auth.ts                  # login redirect, onboarding, logout
│   ├── vehicles.ts, breakdowns.ts, offers.ts, jobs.ts, mechanics.ts, notifications.ts
├── types/                       # mirrors of the Rust contract DTOs
└── lib/config.ts                # API/WS URL resolution
```

## Authentication rules (mandatory)

1. The only auth surface is the ToolFix API.
2. "Continue with Google" navigates to `${API_URL}/api/v1/auth/google/login`;
   the backend runs the Google/Firebase flow server-side and redirects back
   to `/auth/callback` with **HttpOnly cookies** set.
3. Tokens are never stored in localStorage, sessionStorage, IndexedDB, URLs,
   or any JavaScript-readable state; frontend JavaScript cannot read them.
4. All API calls use `credentials: "include"`. On `401`, `services/api.ts`
   transparently calls `POST /api/v1/auth/refresh` once (single-flight) and
   retries.
5. Roles are decided by the backend. The frontend only *requests* onboarding.

## Maps

`components/map/MapView.tsx` defines the application-facing interface
(center, markers, pick handler). The current implementation uses
react-leaflet + OpenStreetMap tiles (`MapViewImpl.tsx`, client-only). Swapping
vendors means reimplementing one file — callers never import Leaflet
directly.
