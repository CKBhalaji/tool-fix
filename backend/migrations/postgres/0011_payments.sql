-- 0011: payments and their transaction trail.

CREATE TABLE payments (
    id                  UUID PRIMARY KEY,
    job_id              UUID NOT NULL REFERENCES assistance_jobs (id) ON DELETE CASCADE,
    amount_minor        BIGINT NOT NULL CHECK (amount_minor > 0),
    currency            TEXT NOT NULL DEFAULT 'INR',
    method              TEXT NOT NULL CHECK (method IN ('cash', 'upi', 'card')),
    status              TEXT NOT NULL CHECK (status IN ('initiated', 'confirmed', 'failed', 'refunded')),
    provider            TEXT NOT NULL,
    provider_payment_id TEXT,
    receipt_number      TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    confirmed_at        TIMESTAMPTZ
);

CREATE INDEX payments_job_id_idx ON payments (job_id);

CREATE TABLE payment_transactions (
    id         UUID PRIMARY KEY,
    payment_id UUID NOT NULL REFERENCES payments (id) ON DELETE CASCADE,
    event_kind TEXT NOT NULL,
    detail     TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX payment_transactions_payment_id_idx ON payment_transactions (payment_id, created_at);
