CREATE TYPE payment_status AS ENUM ('pending', 'confirmed', 'failed', 'refunded');

CREATE TABLE payments (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                     UUID NOT NULL REFERENCES users(id),
    telegram_payment_charge_id  VARCHAR(255) UNIQUE,
    provider_payment_charge_id  VARCHAR(255),
    amount                      INTEGER NOT NULL,
    currency                    VARCHAR(10) NOT NULL DEFAULT 'XTR',
    payload                     JSONB NOT NULL DEFAULT '{}',
    status                      payment_status NOT NULL DEFAULT 'pending',
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_payments_user_id ON payments(user_id);
CREATE INDEX idx_payments_status ON payments(status);
