-- Add 'expired' variant to removal_reason enum
ALTER TYPE removal_reason ADD VALUE IF NOT EXISTS 'expired';
