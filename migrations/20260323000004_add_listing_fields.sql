ALTER TABLE listings ADD COLUMN previous_price INTEGER;
ALTER TABLE listings ADD COLUMN expires_at TIMESTAMPTZ;

UPDATE listings SET expires_at = created_at + INTERVAL '30 days' WHERE status = 'active' AND expires_at IS NULL;

CREATE OR REPLACE FUNCTION trigger_capture_previous_price()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.price IS DISTINCT FROM NEW.price THEN
        NEW.previous_price = OLD.price;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_listings_previous_price
    BEFORE UPDATE ON listings FOR EACH ROW
    EXECUTE FUNCTION trigger_capture_previous_price();
