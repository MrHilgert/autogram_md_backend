DO $$ BEGIN
  CREATE TYPE removal_reason AS ENUM ('sold', 'other');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

ALTER TABLE listings
ADD COLUMN IF NOT EXISTS removal_reason removal_reason,
ADD COLUMN IF NOT EXISTS removed_at TIMESTAMPTZ;
