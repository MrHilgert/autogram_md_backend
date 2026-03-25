CREATE TABLE listings (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                 UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title                   VARCHAR(200) NOT NULL,
    description             TEXT NOT NULL DEFAULT '',
    price                   INTEGER NOT NULL CHECK (price >= 0),
    currency                currency_code NOT NULL DEFAULT 'USD',
    status                  listing_status NOT NULL DEFAULT 'active',
    make_id                 INTEGER NOT NULL REFERENCES car_makes(id),
    model_id                INTEGER NOT NULL REFERENCES car_models(id),
    year                    SMALLINT NOT NULL CHECK (year BETWEEN 1900 AND 2100),
    fuel                    fuel_type NOT NULL,
    body                    body_type NOT NULL,
    transmission            transmission_type NOT NULL,
    drive                   drive_type,
    engine_displacement_cc  INTEGER CHECK (engine_displacement_cc > 0),
    horsepower              SMALLINT CHECK (horsepower > 0),
    mileage_km              INTEGER NOT NULL CHECK (mileage_km >= 0),
    color                   VARCHAR(50),
    doors_count             SMALLINT CHECK (doors_count BETWEEN 2 AND 6),
    steering                steering_side NOT NULL DEFAULT 'left',
    condition               car_condition NOT NULL DEFAULT 'used',
    features                JSONB NOT NULL DEFAULT '[]'::jsonb,
    location                VARCHAR(200),
    views_count             INTEGER NOT NULL DEFAULT 0,
    likes_count             INTEGER NOT NULL DEFAULT 0,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Feed pagination index
CREATE INDEX idx_listings_feed ON listings (created_at DESC, id DESC) WHERE status = 'active';
-- Filter indexes
CREATE INDEX idx_listings_make_model ON listings (make_id, model_id, year) WHERE status = 'active';
CREATE INDEX idx_listings_price ON listings (price) WHERE status = 'active';
CREATE INDEX idx_listings_year ON listings (year) WHERE status = 'active';
CREATE INDEX idx_listings_fuel ON listings (fuel) WHERE status = 'active';
CREATE INDEX idx_listings_body ON listings (body) WHERE status = 'active';
CREATE INDEX idx_listings_user_id ON listings (user_id, created_at DESC);
-- Features GIN index
CREATE INDEX idx_listings_features ON listings USING GIN (features jsonb_path_ops);

-- Auto-update updated_at trigger
CREATE OR REPLACE FUNCTION trigger_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_listings_updated_at
    BEFORE UPDATE ON listings
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_updated_at();
