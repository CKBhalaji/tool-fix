-- 0012: ratings and reviews.

CREATE TABLE ratings (
    id               TEXT PRIMARY KEY,
    job_id           TEXT NOT NULL UNIQUE REFERENCES assistance_jobs (id) ON DELETE CASCADE,
    mechanic_id      TEXT NOT NULL REFERENCES mechanics (id) ON DELETE CASCADE,
    customer_user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    score            INTEGER NOT NULL CHECK (score BETWEEN 1 AND 5),
    comment          TEXT,
    created_at       TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX ratings_mechanic_id_idx ON ratings (mechanic_id);

CREATE TABLE reviews (
    id            TEXT PRIMARY KEY,
    rating_id     TEXT NOT NULL UNIQUE REFERENCES ratings (id) ON DELETE CASCADE,
    visible       BOOLEAN NOT NULL DEFAULT TRUE,
    mechanic_reply TEXT,
    created_at    TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
