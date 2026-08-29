-- 0008: location event stream (customer + mechanic positions over time).

CREATE TABLE location_events (
    id          UUID PRIMARY KEY,
    subject     TEXT NOT NULL CHECK (subject IN ('customer', 'mechanic')),
    subject_id  UUID NOT NULL,
    job_id      UUID REFERENCES assistance_jobs (id) ON DELETE SET NULL,
    latitude    DOUBLE PRECISION NOT NULL,
    longitude   DOUBLE PRECISION NOT NULL,
    accuracy_m  DOUBLE PRECISION,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX location_events_subject_idx ON location_events (subject_id, recorded_at DESC);
CREATE INDEX location_events_job_idx ON location_events (job_id, recorded_at DESC);
