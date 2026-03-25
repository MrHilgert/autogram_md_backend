CREATE TYPE notification_type AS ENUM ('price_drop', 'new_by_search', 'seller_stats', 'listing_expiring');

CREATE TABLE notifications (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type notification_type NOT NULL,
    listing_id        UUID REFERENCES listings(id) ON DELETE SET NULL,
    message_text      TEXT NOT NULL,
    sent_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    telegram_sent     BOOLEAN NOT NULL DEFAULT false
);
CREATE INDEX idx_notifications_user_date ON notifications(user_id, sent_at DESC);
CREATE INDEX idx_notifications_dedup ON notifications(user_id, notification_type, listing_id) WHERE listing_id IS NOT NULL;
