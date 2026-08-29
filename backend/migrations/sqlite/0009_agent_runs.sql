-- 0009: agent runs and outputs (the AI audit trail).

CREATE TABLE agent_runs (
    id            TEXT PRIMARY KEY,
    breakdown_id  TEXT NOT NULL REFERENCES breakdowns (id) ON DELETE CASCADE,
    job_id        TEXT REFERENCES assistance_jobs (id) ON DELETE SET NULL,
    kind          TEXT NOT NULL CHECK (kind IN ('diagnosis', 'price_estimate')),
    provider      TEXT NOT NULL,
    model         TEXT,
    status        TEXT NOT NULL CHECK (status IN ('succeeded', 'failed')),
    error_message TEXT,
    latency_ms    INT,
    created_at    TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX agent_runs_breakdown_id_idx ON agent_runs (breakdown_id, created_at DESC);

CREATE TABLE agent_outputs (
    id           TEXT PRIMARY KEY,
    agent_run_id TEXT NOT NULL REFERENCES agent_runs (id) ON DELETE CASCADE,
    output       TEXT NOT NULL,
    created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- SQLite cannot ADD CONSTRAINT post-hoc; the FK is declared inline in the
-- SQLite version of 0004 (SQLite resolves forward table references lazily).
