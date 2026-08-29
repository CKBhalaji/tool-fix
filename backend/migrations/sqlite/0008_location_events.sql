-- 0008: location event stream (customer + mechanic positions over time).

CREATE TABLE location_events (
    id          TEXT PRIMARY KEY,
    subject     TEXT NOT NULL CHECK (subject IN ('customer', 'mechanic')),
    subject_id  TEXT NOT NULL,
    job_id      TEXT REFERENCES assistance_jobs (id) ON DELETE SET NULL,
    latitude    REAL NOT NULL,
    longitude   REAL NOT NULL,
    accuracy_m  REAL,
    recorded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX location_events_subject_idx ON location_events (subject_id, recorded_at DESC);
CREATE INDEX location_events_job_idx ON location_events (job_id, recorded_at DESC);
