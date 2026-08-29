-- 0002: vehicle catalog and customer vehicles.

CREATE TABLE vehicle_types (
    id    SMALLSERIAL PRIMARY KEY,
    kind  TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL
);

INSERT INTO vehicle_types (kind, label) VALUES
    ('motorcycle', 'Motorcycle'),
    ('scooter',    'Scooter'),
    ('car',        'Car'),
    ('van',        'Van'),
    ('truck',      'Truck'),
    ('other',      'Other');

CREATE TABLE vehicles (
    id                  UUID PRIMARY KEY,
    owner_user_id       UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    vehicle_kind        TEXT NOT NULL
        CHECK (vehicle_kind IN ('motorcycle', 'scooter', 'car', 'van', 'truck', 'other')),
    make                TEXT,
    model               TEXT,
    year                SMALLINT CHECK (year BETWEEN 1950 AND 2100),
    registration_number TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX vehicles_owner_user_id_idx ON vehicles (owner_user_id);
