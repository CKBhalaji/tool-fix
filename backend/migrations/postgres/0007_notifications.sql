-- 0007: notification intents and delivery records.

CREATE TABLE notifications (
    id                UUID PRIMARY KEY,
    recipient_user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    kind              TEXT NOT NULL
        CHECK (kind IN ('new_breakdown_nearby', 'offer_received', 'offer_accepted',
                        'offer_expired', 'mechanic_en_route', 'mechanic_arrived',
                        'job_completed', 'job_cancelled', 'job_no_mechanic',
                        'payment_receipt')),
    channel           TEXT NOT NULL CHECK (channel IN ('push', 'sms', 'email', 'in_app')),
    job_id            UUID REFERENCES assistance_jobs (id) ON DELETE SET NULL,
    payload           JSONB NOT NULL DEFAULT '{}',
    status            TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'sent', 'failed')),
    provider          TEXT,
    delivery_detail   TEXT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_at           TIMESTAMPTZ
);

CREATE INDEX notifications_recipient_user_id_idx ON notifications (recipient_user_id, created_at DESC);
CREATE INDEX notifications_pending_idx ON notifications (created_at)
    WHERE status = 'pending';
