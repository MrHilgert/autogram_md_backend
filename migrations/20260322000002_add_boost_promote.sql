ALTER TABLE listings ADD COLUMN boosted_at TIMESTAMPTZ;
ALTER TABLE listings ADD COLUMN promoted_stars INTEGER NOT NULL DEFAULT 0;
CREATE INDEX idx_listings_promoted ON listings(promoted_stars DESC) WHERE promoted_stars > 0 AND status = 'active';
