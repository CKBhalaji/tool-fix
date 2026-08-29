-- 0005: assistance jobs and their immutable status history.
-- Every status change inserts a job_status_history row in the same
-- transaction as the assistance_jobs update.

CREATE TABLE assistance_jobs (
    id                      TEXT PRIMARY KEY,
    breakdown_id            TEXT NOT NULL UNIQUE REFERENCES breakdowns (id) ON DELETE CASCADE,
    customer_user_id        TEXT NOT NULL REFERENCES users (id) ON DELETE RESTRICT,
    vehicle_id              TEXT NOT NULL REFERENCES vehicles (id) ON DELETE RESTRICT,
    status                  TEXT NOT NULL DEFAULT 'created'
        CHECK (status IN ('created', 'analyzing', 'mechanics_searching', 'mechanics_notified',
                          'offers_received', 'mechanic_selected', 'mechanic_en_route',
                          'mechanic_arrived', 'repair_in_progress', 'repair_completed',
                          'payment_pending', 'completed', 'cancelled', 'expired', 'failed',
                          'no_mechanic_available')),
    selected_mechanic_id    TEXT REFERENCES mechanics (id) ON DELETE SET NULL,
    final_amount_minor      INTEGER,
    currency                TEXT NOT NULL DEFAULT 'INR',
    offer_window_expires_at TEXT,
    created_at              TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at              TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX assistance_jobs_status_idx ON assistance_jobs (status)
    WHERE status IN ('mechanics_notified', 'offers_received', 'mechanic_selected',
                     'mechanic_en_route', 'mechanic_arrived', 'repair_in_progress',
                     'payment_pending');
CREATE INDEX assistance_jobs_customer_user_id_idx ON assistance_jobs (customer_user_id);
CREATE INDEX assistance_jobs_selected_mechanic_id_idx ON assistance_jobs (selected_mechanic_id);

CREATE TABLE job_status_history (
    id                 TEXT PRIMARY KEY,
    job_id             TEXT NOT NULL REFERENCES assistance_jobs (id) ON DELETE CASCADE,
    from_status        TEXT,
    to_status          TEXT NOT NULL,
    changed_by_user_id TEXT REFERENCES users (id) ON DELETE SET NULL,
    reason             TEXT,
    created_at         TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX job_status_history_job_id_idx ON job_status_history (job_id, created_at);
