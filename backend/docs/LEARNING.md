# ToolFix Learning Handbook — Rust backend from zero

This file is a **question-and-answer handbook** for learning how the ToolFix
backend works, written for someone who knows nothing about Rust.

**How we use it:** you ask a question (in any words), it gets framed properly,
answered against the *real ToolFix code*, and appended here as a new chapter.
Over time this becomes the guide to your own backend.

- Ask anything — no question is too basic.
- Every answer points to real files you can open and read.
- New chapters are added at the bottom; the index below grows with them.

## Index

| Chapter | Question |
|---|---|
| [Q1](#q1) | What is Rust, what is a crate, and how are the ToolFix crates organised? |
| [Q2](#q2) | How does one crate connect to another crate? |
| [Q3](#a3) | How does memory management work in Rust? |
| [Q4](#q4) | What happens when the backend starts? (from `cargo run` to a listening server) |
| [Q5](#q5) | How do environment variables (.env) get connected to the code that uses them? |
| [Q6](#q6) | How does an API request actually work, end to end? |
| [Q7](#q7) | How do the frontend and backend talk to each other? |

---

## Q1 — "What is Rust, what is a crate, and how are the ToolFix crates organised?"

### What Rust is

Rust is a **compiled** language. Compare:

| | Node.js (frontend) | Python (FastAPI) | Rust (ToolFix backend) |
|---|---|---|---|
| How it runs | JavaScript engine reads code at runtime | Interpreter reads code at runtime | Compiled **ahead of time** to machine code (`target/release/toolfix-server.exe`) |
| Speed | fast-ish | slow | very fast |
| Memory | garbage collector cleans up | garbage collector cleans up | **compiler enforces rules at build time** — no collector needed |
| Errors | often appear while running | often appear while running | most are caught **before the program can even start** |

That last row is the big one: in Rust, if you forget to handle a database
error, the program refuses to compile. That is why ToolFix "just works" once
it builds.

### What a crate is

A **crate** is one compilable unit of Rust code — the Rust word for a
package/library. Two kinds exist:

- **binary crate** — has a `main()` function, becomes an executable.
- **library crate** — has no `main()`; other code imports it.

A `Cargo.toml` file describes a crate: its name, version, and
**dependencies** (other crates it uses). Cargo is the tool that reads it,
downloads dependencies, compiles everything, runs tests, etc. (Cargo ≈ npm
for Rust; `Cargo.toml` ≈ `package.json`; `Cargo.lock` ≈ `pnpm-lock.yaml`).

### How ToolFix is organised

`backend/Cargo.toml` declares a **workspace** — a folder containing many
crates that build together and share one `target/` build directory and one
`Cargo.lock`:

```toml
[workspace]
resolver = "3"
members = [
    "crates/toolfix-contracts",
    "crates/toolfix-domain",
    ... 15 crates total ...
]
```

Exactly **one** crate is a binary: `crates/toolfix-server` (its
`src/main.rs` has the `main()` function → becomes `toolfix-server.exe`).
The other 14 are libraries, each with one clear job:

| Crate | One-line job |
|---|---|
| `toolfix-contracts` | the data shapes sent over the network (DTOs) and shared enums |
| `toolfix-domain` | business rules — the job state machine, validation |
| `toolfix-auth` | Google/Firebase login, tokens, cookies, roles |
| `toolfix-persistence` | all SQL + database tables |
| `toolfix-location` | distance math, ETA calculation |
| `toolfix-notifications` | sending notifications (logging provider today, FCM later) |
| `toolfix-storage` | saving photos/videos to disk (or S3 later) |
| `toolfix-payments` | payment provider abstraction (cash stub today) |
| `toolfix-matching` | finding + ranking nearby mechanics |
| `toolfix-pricing` | price history and AI estimate records |
| `toolfix-agent` | the AI layer (Gemini / NVIDIA / Vertex) |
| `toolfix-jobs` | orchestrates the whole marketplace flow |
| `toolfix-runtime` | background loops (expiring offers, notifications) |
| `toolfix-api` | HTTP routes and handlers (Axum) |
| `toolfix-server` | the composition root — builds everything and runs it |

Why split like this? Same reason as folders in a big frontend: each piece has
one responsibility, can be understood alone, and the *compiler* enforces the
boundaries (a crate simply cannot import what it is not allowed to).

---

## Q2 — "How does one crate connect to another crate?"

A crate connects to another by **listing it as a dependency in its
`Cargo.toml`**. After that, it can use everything the other crate marks
`pub` (public). Anything not `pub` is invisible — that is how boundaries are
enforced.

Example — `crates/toolfix-jobs/Cargo.toml`:

```toml
[dependencies]
toolfix-contracts   = { workspace = true }
toolfix-domain       = { workspace = true }
toolfix-persistence  = { workspace = true }
...
```

`{ workspace = true }` means "use the version pinned once in
`backend/Cargo.toml` under `[workspace.dependencies]`", so every crate uses
the same versions.

Then in real code, `crates/toolfix-jobs/src/service.rs` can write:

```rust
use toolfix_persistence::Repositories;          // import from another crate
use toolfix_contracts::JobStatus;               // and another
```

### The dependency direction (the rule that keeps it sane)

Crates point **downward only** — no cycles allowed (Cargo would refuse to
build them anyway):

```text
toolfix-contracts          ← pure data types, depends on nothing internal
        ↓
toolfix-domain             ← business rules
        ↓
auth  persistence  location  notifications  storage  payments
        ↓
matching   pricing   agent
        ↓
toolfix-jobs               ← orchestrates the above
        ↓
toolfix-runtime            ← background loops
        ↓
toolfix-api                ← HTTP handlers
        ↓
toolfix-server             ← builds everything, runs main()
```

Concrete example of "who uses whom":

- `toolfix-api` handlers call `toolfix-jobs`' `JobService` — but a handler
  may never run SQL itself.
- `toolfix-jobs` calls `toolfix-matching` and `toolfix-persistence` — but it
  never formats an HTTP response.
- `toolfix-persistence` knows SQL — but it never decides *whether* an offer
  may be accepted (that rule lives in `toolfix-domain`).

If you ever wonder "where does X belong?", follow this ladder.

### The composition root — where everything is glued

No crate "finds" another automatically. `crates/toolfix-server/src/main.rs`
constructs every service **bottom-up** and hands each one what it needs:

```text
main.rs:
  connect database          → Db
  Repositories::new(db)     → all SQL repos in one bundle
  MatchingService::new(repos.mechanics, repos.notifications, provider, config)
  AuthService::new(config.auth, repos.users, repos.auth_sessions, ...)
  JobService::new(repos, matching, pricing, agent, storage, payments, events, ...)
  build_state(...)          → one AppState the HTTP layer shares
  axum::serve(...)          → start listening
```

This pattern is called **dependency injection done manually** — the
composition root is the *only* place that knows how the pieces connect.

---

## Q3 — "How does memory management work in Rust?"

Rust has **no garbage collector** and no manual `free()`. Instead the
**compiler** enforces three rules, collectively called *ownership*:

1. Every value has exactly **one owner** (the variable holding it).
2. When the owner goes out of scope, the memory is **freed automatically**.
3. You can either move the value to a new owner, or **borrow** it
   (`&value` = read-only loan, `&mut value` = one writable loan) — but you
   can never have two writers at once.

You never call `malloc`/`free`; the compiler inserts the frees for you, and
it proves at compile time that no use-after-free, double-free, or data race
can happen. This is why Rust programs don't have whole classes of bugs and
don't need a garbage collector.

Where you meet ownership in ToolFix (all real examples):

```rust
// Moving: `payload` is handed to the JSON macro, then cloned because the
// notification AND the provider delivery both need it.
let payload = serde_json::json!({ ... });
let notification = self.notifications.create(..., payload.clone()).await?;
self.provider.send(... payload ...)

// Borrowing: the function only READS the breakdown, it doesn't own it.
pub async fn find_mechanics(
    &self,
    breakdown: &toolfix_persistence::models::BreakdownRow,   // ← borrow
    ...
)

// Results: errors must be handled before the program compiles.
let job = self.repos.jobs.find_by_id(job_id).await?;   // `?` = if error, return it
```

The `?` operator is everywhere in the backend: it means *"if this call
failed, stop and pass the error upward; otherwise unwrap the value."*
That's how a call chain like handler → `JobService` → repository propagates
errors without a single `try/catch`.

Stack vs heap, briefly: small fixed-size values (numbers, booleans) live on
the fast stack; growable values (`String`, `Vec`, connections) live on the
heap, and the stack holds a small pointer to them. Ownership rules apply to
the heap values — that's what the compiler tracks.

Shared handles: database pools internally use `Arc` (atomic reference
counting) — many parts of the app can hold a clone of `Db`/`AppState` while
pointing at the same underlying pool. Cloning `Db`/`AppState` is cheap;
it copies a pointer, not a connection.

---

## Q4 — "What happens when the backend starts?"

Follow `crates/toolfix-server/src/main.rs` — this is the actual boot order:

```text
1. dotenvy::dotenv()                     load .env into process variables
2. tracing_subscriber::fmt()             turn on logs (RUST_LOG controls level)
3. Config::from_env()                    read + validate every variable
4. persistence::connect(url)             connect Postgres/SQLite
                                         + apply all migrations
5. Repositories::new(db)                 one bundle of every SQL repository
6. LocalFsBackend::new(storage_dir)      file storage for photos/videos
7. LoggingProvider                       notification provider (FCM later)
8. MatchingService / PricingService      deterministic business services
9. build_provider(AI_PROVIDER, ...)      Gemini or NVIDIA or Vertex (optional!)
10. AuthService::new(...)                cookies + tokens machinery
11. CashStubProvider                     payment provider (Razorpay later)
12. JobService::new(...)                 the marketplace orchestrator
13. Runtime::spawn_all(...)              background loops (expiry, dispatch, cleanup)
14. build_state(...) → AppState          one shared handle for all HTTP handlers
15. build_cors(frontend_origin)          allow-list for the frontend
16. axum::serve(listener, router)        START LISTENING on 0.0.0.0:8080
17. graceful shutdown                    Ctrl+C / SIGTERM → cancel loops → exit
```

Notice step 9 is **optional**: if no AI key is configured the server logs a
warning and runs without the agent — breakdowns still work, matching still
works, the AI is only an advisory layer.

---

## Q5 — "How do environment variables get connected to the code?"

This is a chain with five links. Follow one variable — `JWT_ACCESS_SECRET` —
from file to usage:

**Link 1 — the file.** `backend/.env` (gitignored; copied from
`.env.example`):

```env
JWT_ACCESS_SECRET=8f3b...your-random-value...
```

**Link 2 — loaded into the process.** First line of `main.rs`:

```rust
dotenvy::dotenv().ok();     // reads .env, injects into process environment
```

(Real operating-system environment variables always win over `.env` — that's
how Docker/production overrides work.)

**Link 3 — parsed into a typed struct.** `crates/toolfix-server/src/config.rs`:

```rust
let access_secret = env_required("JWT_ACCESS_SECRET")?;
if access_secret.len() < 32 { return Err(...); }        // validated!
```

`Config::from_env()` reads *every* variable and packs them into nested
structs — `Config { database_driver, database_url, auth: AuthConfig {...},
matching: MatchingConfig {...}, ... }`. One struct = typed, autocompleted,
compiler-checked configuration. A missing required variable fails startup
with a clear message; an invalid one (e.g. `DATABASE_DRIVER=oracle`) fails
immediately too.

**Link 4 — injected into the service that owns it.** `main.rs`:

```rust
let auth_service = AuthService::new(config.auth.clone(), ...);
```

`AuthService` keeps the config inside itself. Nothing else can read
`JWT_ACCESS_SECRET` — a crate that needs no secret physically cannot get it.

**Link 5 — used at runtime.** `toolfix-auth/src/claims.rs` signs every
access-token JWT with that secret; `middleware.rs` verifies every incoming
request against it.

The same chain exists for every variable:

| Variable | Parsed in | Ends up controlling |
|---|---|---|
| `DATABASE_URL`, `DATABASE_DRIVER` | config → `connect_with_driver` | SQLite `dev.db` (auto-created) or Postgres |
| `ADMIN_EMAIL` / `ADMIN_PASSWORD` | config → `AuthConfig` | the `/api/v1/admin/login` check |
| `AI_PROVIDER`, `GEMINI_API_KEY`, `NVIDIA_API_KEY` | config → `build_provider` | which AI backend the agent uses |
| `MATCHING_RADIUS_STAGE*_M`, `MAX_MECHANICS_NOTIFIED` | config → `MatchingConfig` | the matching algorithm |
| `FRONTEND_ORIGIN` | config → CORS + login redirect | which browser origin may call the API |

**To add a new env variable:** add it to `.env` + `.env.example`, read it in
`Config::from_env`, store it in the matching struct, pass it into the service
that needs it. Four small steps — the compiler tells you if you forgot one.

---

## Q6 — "How does an API request actually work, end to end?"

The HTTP layer is **Axum**. A router is literally a table: *method + path →
handler function*. Defined in `crates/toolfix-api/src/routes.rs`:

```rust
.route("/{id}/cancel", post(toolfix_api::handlers::breakdowns::cancel))
.nest("/api/v1/breakdowns", breakdown_routes)
```

When the frontend calls `POST http://localhost:8080/api/v1/breakdowns/cancel`,
the request travels through these layers:

```text
1. CORS layer            is the origin allowed? (tower-http CorsLayer)
2. Body limit            reject > 15 MB uploads
3. Router match          method + path → handler
4. Extractors            handler arguments pull data OUT of the request:
       State<AppState>   → the shared services
       AuthUser          → reads the access_token cookie, verifies JWT,
                           loads user id + role  (identity from cookie only!)
       Json<T>           → parses the body into a typed struct
       Path<Uuid>        → the {id} from the URL
5. Handler               TRANSLATION ONLY: call a service, map result to DTO
6. Service (toolfix-jobs) BUSINESS RULES: state machine, ownership checks,
                         transactions
7. Repository (persistence) SQL: UPDATE ... WHERE status = $4 (optimistic lock)
8. PostgreSQL/SQLite      durable truth
9. Response               JSON serialized back, or a typed error
```

Walk the real cancel-a-request flow:

```text
POST /api/v1/breakdowns/{id}/cancel
  → handlers/breakdowns.rs::cancel      (finds the job, checks owner)
  → jobs::JobService::cancel_job
      → domain::validate_transition(current, Cancelled)   ← state machine rule
      → repos.jobs.transition_job(...)                    ← one SQL transaction,
      → publish "job.cancelled" event                       writes history row
  → JSON JobResponse back to the browser
```

Three rules to remember:

- **Handlers never touch SQL** and never decide business rules.
- **Every external input is validated** (in the domain layer or handler).
- **Every important state change is one database transaction** — the offer
  acceptance, for example, accepts one offer, expires the competitors, and
  advances the job *atomically*.

The WebSocket channel (`/api/v1/ws/jobs/{id}`) rides beside this: services
publish events into an in-process broadcast hub, and connected browsers
receive them live — but PostgreSQL remains the source of truth; the socket
is only a notification bell.

---

## Q7 — "How do the frontend and backend talk to each other?"

- The Next.js app calls `fetch(apiUrl + path, { credentials: "include" })` —
  the browser **automatically attaches the HttpOnly cookies**. JavaScript can
  never read them (that's the whole security model: `services/api.ts`).
- CORS is an explicit allow-list: the backend only accepts browser calls from
  `FRONTEND_ORIGIN` (e.g. `http://localhost:3000`) — never `*` with cookies.
- On `401`, `services/api.ts` silently calls `POST /api/v1/auth/refresh`
  once (rotating the refresh token) and retries the original request.
- Live tracking uses one WebSocket per job (`/api/v1/ws/jobs/{id}`); the
  events it pushes mirror what is already saved in PostgreSQL.

---

## Suggested reading order (30 minutes)

1. `crates/toolfix-server/src/main.rs` — the whole app in ~120 lines
2. `crates/toolfix-domain/src/state_machine.rs` — the heart of the rules
3. `crates/toolfix-api/src/routes.rs` — the complete API surface on one page
4. `crates/toolfix-jobs/src/service.rs` — the marketplace orchestration
5. `crates/toolfix-persistence/src/repositories/offers.rs` — transactional SQL

---

*Next questions get appended here as Q8, Q9, … Ask anything: "what is
async/await?", "what is a trait?", "why are there two migration folders?",
"what happens when two mechanics bid at the same time?" — everything gets
framed, answered, and filed in this handbook.*
