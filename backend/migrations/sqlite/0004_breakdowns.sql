-- 0004: breakdown reports, attached media, and (advisory) AI diagnoses.

CREATE TABLE breakdowns (
    id                 TEXT PRIMARY KEY,
    user_id            TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    vehicle_id         TEXT NOT NULL REFERENCES vehicles (id) ON DELETE RESTRICT,
    latitude           REAL NOT NULL,
    longitude          REAL NOT NULL,
    address            TEXT,
    problem_description TEXT NOT NULL,
    vehicle_symptoms   TEXT NOT NULL DEFAULT '[]',
    created_at         TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at         TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX breakdowns_user_id_idx ON breakdowns (user_id);
CREATE INDEX breakdowns_created_at_idx ON breakdowns (created_at DESC);

CREATE TABLE breakdown_media (
    id           TEXT PRIMARY KEY,
    breakdown_id TEXT NOT NULL REFERENCES breakdowns (id) ON DELETE CASCADE,
    media_kind   TEXT NOT NULL CHECK (media_kind IN ('photo', 'video')),
    storage_key  TEXT NOT NULL,
    content_type TEXT,
    size_bytes   INTEGER,
    created_at   TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX breakdown_media_breakdown_id_idx ON breakdown_media (breakdown_id);

-- AI output is advisory; this row is informational and linked to the agent
-- run that produced it (FK added in 0009 once agent_runs exists).
CREATE TABLE breakdown_diagnoses (
    id                       TEXT PRIMARY KEY,
    breakdown_id             TEXT NOT NULL REFERENCES breakdowns (id) ON DELETE CASCADE,
    agent_run_id             TEXT REFERENCES agent_runs (id) ON DELETE SET NULL,
    possible_issue           TEXT NOT NULL,
    confidence               REAL NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    severity                 TEXT NOT NULL
        CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    recommended_service      TEXT NOT NULL,
    repair_category          TEXT NOT NULL
        CHECK (repair_category IN ('battery', 'tyre', 'engine', 'electrical', 'brakes',
                                   'clutch', 'fuel', 'chain_drive', 'cooling', 'body_damage',
                                   'lockout', 'towing', 'diagnostic', 'other')),
    estimated_cost_min_minor INTEGER NOT NULL CHECK (estimated_cost_min_minor > 0),
    estimated_cost_max_minor INTEGER NOT NULL CHECK (estimated_cost_max_minor >= estimated_cost_min_minor),
    requires_towing          BOOLEAN NOT NULL DEFAULT FALSE,
    reasoning_summary        TEXT NOT NULL,
    created_at               TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX breakdown_diagnoses_breakdown_id_key ON breakdown_diagnoses (breakdown_id);
