-- 0012: ratings and reviews.

CREATE TABLE ratings (
    id               UUID PRIMARY KEY,
    job_id           UUID NOT NULL UNIQUE REFERENCES assistance_jobs (id) ON DELETE CASCADE,
    mechanic_id      UUID NOT NULL REFERENCES mechanics (id) ON DELETE CASCADE,
    customer_user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    score            SMALLINT NOT NULL CHECK (score BETWEEN 1 AND 5),
    comment          TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX ratings_mechanic_id_idx ON ratings (mechanic_id);

CREATE TABLE reviews (
    id            UUID PRIMARY KEY,
    rating_id     UUID NOT NULL UNIQUE REFERENCES ratings (id) ON DELETE CASCADE,
    visible       BOOLEAN NOT NULL DEFAULT TRUE,
    mechanic_reply TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
