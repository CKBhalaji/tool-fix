# Location & Live Tracking

## Data model

- `location_events` — append-only position history for customers and
  mechanics (`subject`, `subject_id`, `job_id`, lat/lng, accuracy, time).
  A retention loop prunes samples older than the configured window
  (default 24 h).
- `mechanic_locations` — current mechanic position (fast path for matching).
- `mechanics.current_latitude/longitude` — denormalized mirror maintained in
  the same transaction for cheap bounding-box candidate queries.

## Reporting

- Customers: position is captured at breakdown creation (browser geolocation
  + map picker on the frontend).
- Mechanics: `POST /api/v1/mechanics/location` while online; the frontend
  watches the Geolocation API and pings periodically. Pings update the
  current-position row, append a `location_events` sample, and — when a job
  is attached — broadcast over the job WebSocket channel.

## Live tracking

```text
Customer browser  ↕  WebSocket /api/v1/ws/jobs/{job_id}  ↕  Rust backend  ↕  Mechanic
```

- The upgrade is cookie-authenticated and authorized (customer, selected
  mechanic, or admin).
- `toolfix-jobs` publishes in-process events (`tokio::sync::broadcast`) for
  job state changes, offers, and mechanic location pings; the API forwards
  frames filtered by job id. The frontend updates the map marker without a
  page reload.
- **PostgreSQL remains the source of truth.** A reconnecting client re-reads
  the job and status history via REST; the socket is transport only and never
  authoritative. Multiple replicas are supported because matching, offer
  acceptance, and state transitions are database transactions, not in-memory
  state.

## ETA

`toolfix_location::eta_minutes(distance, MATCHING_AVG_SPEED_KMH)` provides
rough ETAs for matching and the feed. Route polyline/routing APIs are a
future enhancement; the `MapView` abstraction already accepts route
representation.
