-- 0006: mechanic offers (the bidding layer) and offer events.

CREATE TABLE mechanic_offers (
    id                        UUID PRIMARY KEY,
    job_id                    UUID NOT NULL REFERENCES assistance_jobs (id) ON DELETE CASCADE,
    mechanic_id               UUID NOT NULL REFERENCES mechanics (id) ON DELETE CASCADE,
    quoted_price_minor        BIGINT NOT NULL CHECK (quoted_price_minor > 0),
    currency                  TEXT NOT NULL DEFAULT 'INR',
    estimated_arrival_minutes INT NOT NULL CHECK (estimated_arrival_minutes > 0),
    message                   TEXT,
    status                    TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'withdrawn', 'expired', 'rejected')),
    expires_at                TIMESTAMPTZ NOT NULL,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (job_id, mechanic_id)
);

CREATE INDEX mechanic_offers_job_id_idx ON mechanic_offers (job_id);
CREATE INDEX mechanic_offers_mechanic_id_idx ON mechanic_offers (mechanic_id);
CREATE INDEX mechanic_offers_expiry_idx ON mechanic_offers (expires_at)
    WHERE status = 'pending';

CREATE TABLE offer_events (
    id            UUID PRIMARY KEY,
    offer_id      UUID NOT NULL REFERENCES mechanic_offers (id) ON DELETE CASCADE,
    event_kind    TEXT NOT NULL,
    actor_user_id UUID REFERENCES users (id) ON DELETE SET NULL,
    detail        TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX offer_events_offer_id_idx ON offer_events (offer_id, created_at);
