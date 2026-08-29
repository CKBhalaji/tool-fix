-- 0013: audit log and remaining hot indexes.

CREATE TABLE audit_logs (
    id            UUID PRIMARY KEY,
    actor_user_id UUID REFERENCES users (id) ON DELETE SET NULL,
    action        TEXT NOT NULL,
    entity_type   TEXT,
    entity_id     UUID,
    detail        JSONB,
    ip_address    TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX audit_logs_created_at_idx ON audit_logs (created_at DESC);
CREATE INDEX audit_logs_actor_idx ON audit_logs (actor_user_id, created_at DESC);

-- Additional hot-path indexes.
CREATE INDEX breakdown_media_lookup_idx ON breakdown_media (breakdown_id, created_at);
CREATE INDEX mechanics_services_area_idx ON mechanics (service_area_km)
    WHERE availability_status = 'online';
