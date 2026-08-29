-- 0010: price estimates (advisory) and realized price history (learning data).

CREATE TABLE price_estimates (
    id                       TEXT PRIMARY KEY,
    breakdown_id             TEXT NOT NULL REFERENCES breakdowns (id) ON DELETE CASCADE,
    job_id                   TEXT REFERENCES assistance_jobs (id) ON DELETE CASCADE,
    repair_category          TEXT NOT NULL,
    estimated_cost_min_minor INTEGER NOT NULL CHECK (estimated_cost_min_minor > 0),
    estimated_cost_max_minor INTEGER NOT NULL CHECK (estimated_cost_max_minor >= estimated_cost_min_minor),
    currency                 TEXT NOT NULL DEFAULT 'INR',
    source                   TEXT NOT NULL CHECK (source IN ('agent', 'historical')),
    notes                    TEXT,
    created_at               TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX price_estimates_breakdown_id_idx ON price_estimates (breakdown_id);

-- One row per completed job; feeds the historical pricing analysis.
CREATE TABLE price_history (
    id                 TEXT PRIMARY KEY,
    job_id             TEXT REFERENCES assistance_jobs (id) ON DELETE SET NULL,
    repair_category    TEXT NOT NULL,
    vehicle_kind       TEXT,
    city               TEXT,
    final_amount_minor INTEGER NOT NULL CHECK (final_amount_minor > 0),
    currency           TEXT NOT NULL DEFAULT 'INR',
    recorded_at        TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX price_history_lookup_idx ON price_history (repair_category, vehicle_kind, recorded_at DESC);
