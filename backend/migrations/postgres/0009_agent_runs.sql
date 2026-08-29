-- 0009: agent runs and outputs (the AI audit trail).

CREATE TABLE agent_runs (
    id            UUID PRIMARY KEY,
    breakdown_id  UUID NOT NULL REFERENCES breakdowns (id) ON DELETE CASCADE,
    job_id        UUID REFERENCES assistance_jobs (id) ON DELETE SET NULL,
    kind          TEXT NOT NULL CHECK (kind IN ('diagnosis', 'price_estimate')),
    provider      TEXT NOT NULL,
    model         TEXT,
    status        TEXT NOT NULL CHECK (status IN ('succeeded', 'failed')),
    error_message TEXT,
    latency_ms    INT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX agent_runs_breakdown_id_idx ON agent_runs (breakdown_id, created_at DESC);

CREATE TABLE agent_outputs (
    id           UUID PRIMARY KEY,
    agent_run_id UUID NOT NULL REFERENCES agent_runs (id) ON DELETE CASCADE,
    output       TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Link diagnoses to the run that produced them (table existed since 0004).
ALTER TABLE breakdown_diagnoses
    ADD CONSTRAINT breakdown_diagnoses_agent_run_id_fkey
    FOREIGN KEY (agent_run_id) REFERENCES agent_runs (id) ON DELETE SET NULL;
