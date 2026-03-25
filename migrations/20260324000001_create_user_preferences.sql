CREATE TABLE user_preferences (
    user_id             UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    make_weights        JSONB NOT NULL DEFAULT '{}'::jsonb,
    model_weights       JSONB NOT NULL DEFAULT '{}'::jsonb,
    body_weights        JSONB NOT NULL DEFAULT '{}'::jsonb,
    fuel_weights        JSONB NOT NULL DEFAULT '{}'::jsonb,
    trans_weights       JSONB NOT NULL DEFAULT '{}'::jsonb,
    drive_weights       JSONB NOT NULL DEFAULT '{}'::jsonb,
    price_center        DOUBLE PRECISION,
    year_center         DOUBLE PRECISION,
    total_interactions  INTEGER NOT NULL DEFAULT 0,
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
