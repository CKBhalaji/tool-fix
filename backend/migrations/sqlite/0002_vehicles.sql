-- 0002: vehicle catalog and customer vehicles.

CREATE TABLE vehicle_types (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
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
    id                  TEXT PRIMARY KEY,
    owner_user_id       TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    vehicle_kind        TEXT NOT NULL
        CHECK (vehicle_kind IN ('motorcycle', 'scooter', 'car', 'van', 'truck', 'other')),
    make                TEXT,
    model               TEXT,
    year                INTEGER CHECK (year BETWEEN 1950 AND 2100),
    registration_number TEXT,
    created_at          TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX vehicles_owner_user_id_idx ON vehicles (owner_user_id);
