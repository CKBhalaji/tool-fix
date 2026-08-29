-- 0004: breakdown reports, attached media, and (advisory) AI diagnoses.

CREATE TABLE breakdowns (
    id                 UUID PRIMARY KEY,
    user_id            UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    vehicle_id         UUID NOT NULL REFERENCES vehicles (id) ON DELETE RESTRICT,
    latitude           DOUBLE PRECISION NOT NULL,
    longitude          DOUBLE PRECISION NOT NULL,
    address            TEXT,
    problem_description TEXT NOT NULL,
    vehicle_symptoms   TEXT[] NOT NULL DEFAULT '{}',
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX breakdowns_user_id_idx ON breakdowns (user_id);
CREATE INDEX breakdowns_created_at_idx ON breakdowns (created_at DESC);

CREATE TABLE breakdown_media (
    id           UUID PRIMARY KEY,
    breakdown_id UUID NOT NULL REFERENCES breakdowns (id) ON DELETE CASCADE,
    media_kind   TEXT NOT NULL CHECK (media_kind IN ('photo', 'video')),
    storage_key  TEXT NOT NULL,
    content_type TEXT,
    size_bytes   BIGINT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX breakdown_media_breakdown_id_idx ON breakdown_media (breakdown_id);

-- AI output is advisory; this row is informational and linked to the agent
-- run that produced it (FK added in 0009 once agent_runs exists).
CREATE TABLE breakdown_diagnoses (
    id                       UUID PRIMARY KEY,
    breakdown_id             UUID NOT NULL REFERENCES breakdowns (id) ON DELETE CASCADE,
    agent_run_id             UUID,
    possible_issue           TEXT NOT NULL,
    confidence               REAL NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    severity                 TEXT NOT NULL
        CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    recommended_service      TEXT NOT NULL,
    repair_category          TEXT NOT NULL
        CHECK (repair_category IN ('battery', 'tyre', 'engine', 'electrical', 'brakes',
                                   'clutch', 'fuel', 'chain_drive', 'cooling', 'body_damage',
                                   'lockout', 'towing', 'diagnostic', 'other')),
    estimated_cost_min_minor BIGINT NOT NULL CHECK (estimated_cost_min_minor > 0),
    estimated_cost_max_minor BIGINT NOT NULL CHECK (estimated_cost_max_minor >= estimated_cost_min_minor),
    requires_towing          BOOLEAN NOT NULL DEFAULT FALSE,
    reasoning_summary        TEXT NOT NULL,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX breakdown_diagnoses_breakdown_id_key ON breakdown_diagnoses (breakdown_id);
