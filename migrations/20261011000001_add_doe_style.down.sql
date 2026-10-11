ALTER TABLE does
    DROP CONSTRAINT chk_does_cover_photo_key_not_blank,
    DROP COLUMN cover_photo_key,
    DROP COLUMN model_id;

ALTER TABLE doe_models
    DROP CONSTRAINT chk_doe_models_style_object,
    DROP COLUMN style;

ALTER TABLE doe_settings
    ADD COLUMN accent TEXT,
    ADD COLUMN logo_key TEXT NULL;
UPDATE doe_settings
SET accent = coalesce(style->>'primary', '#0f3d36'),
    logo_key = nullif(btrim(style->>'logo_key'), '');
ALTER TABLE doe_settings
    ALTER COLUMN accent SET NOT NULL,
    DROP CONSTRAINT chk_doe_settings_style_object,
    DROP COLUMN style,
    ADD CONSTRAINT chk_doe_settings_accent_hex CHECK (accent ~ '^#[0-9a-f]{6}$'),
    ADD CONSTRAINT chk_doe_settings_logo_key_not_blank
        CHECK (logo_key IS NULL OR btrim(logo_key) <> '');
