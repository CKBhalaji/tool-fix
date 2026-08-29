-- 0003: mechanics, their services, availability schedule, and current position.
-- supported_vehicle_kinds / repair_categories are denormalized TEXT[] on the
-- mechanic row so matching can filter without joins; mechanic_services keeps
-- the per-category pricing baseline.

CREATE TABLE mechanics (
    id                     UUID PRIMARY KEY,
    user_id                UUID NOT NULL UNIQUE REFERENCES users (id) ON DELETE CASCADE,
    display_name           TEXT,
    phone                  TEXT,
    city                   TEXT,
    service_area_km        REAL NOT NULL DEFAULT 10.0,
    supported_vehicle_kinds TEXT[] NOT NULL DEFAULT '{}',
    repair_categories      TEXT[] NOT NULL DEFAULT '{}',
    experience_years       SMALLINT,
    availability_status    TEXT NOT NULL DEFAULT 'offline'
        CHECK (availability_status IN ('online', 'busy', 'offline')),
    rating_average         REAL,
    completed_jobs         BIGINT NOT NULL DEFAULT 0,
    is_verified            BOOLEAN NOT NULL DEFAULT FALSE,
    current_latitude       DOUBLE PRECISION,
    current_longitude      DOUBLE PRECISION,
    location_updated_at    TIMESTAMPTZ,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX mechanics_availability_status_idx ON mechanics (availability_status);
CREATE INDEX mechanics_city_idx ON mechanics (city);

CREATE TABLE mechanic_services (
    id                    UUID PRIMARY KEY,
    mechanic_id           UUID NOT NULL REFERENCES mechanics (id) ON DELETE CASCADE,
    repair_category       TEXT NOT NULL
        CHECK (repair_category IN ('battery', 'tyre', 'engine', 'electrical', 'brakes',
                                   'clutch', 'fuel', 'chain_drive', 'cooling', 'body_damage',
                                   'lockout', 'towing', 'diagnostic', 'other')),
    base_price_minor      BIGINT,
    typical_minutes       INT,
    UNIQUE (mechanic_id, repair_category)
);

CREATE TABLE mechanic_availability (
    id            UUID PRIMARY KEY,
    mechanic_id   UUID NOT NULL REFERENCES mechanics (id) ON DELETE CASCADE,
    day_of_week   SMALLINT NOT NULL CHECK (day_of_week BETWEEN 0 AND 6),
    start_minute  INT NOT NULL CHECK (start_minute BETWEEN 0 AND 1440),
    end_minute    INT NOT NULL CHECK (end_minute BETWEEN 0 AND 1440),
    UNIQUE (mechanic_id, day_of_week, start_minute)
);

-- Current position per mechanic (fast-path for matching/tracking). History
-- lives in location_events.
CREATE TABLE mechanic_locations (
    mechanic_id UUID PRIMARY KEY REFERENCES mechanics (id) ON DELETE CASCADE,
    latitude    DOUBLE PRECISION NOT NULL,
    longitude   DOUBLE PRECISION NOT NULL,
    accuracy_m  DOUBLE PRECISION,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
