ALTER TABLE products      ADD COLUMN custom_fields JSONB NOT NULL DEFAULT '{}';
ALTER TABLE service_rates ADD COLUMN custom_fields JSONB NOT NULL DEFAULT '{}';
