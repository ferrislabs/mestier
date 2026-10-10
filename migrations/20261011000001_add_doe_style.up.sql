-- A DOE's look moves from two columns to one style document, which a model
-- may replace. Fields left out of a stored style read as their defaults.
ALTER TABLE doe_settings ADD COLUMN style JSONB;
UPDATE doe_settings
SET style = jsonb_strip_nulls(jsonb_build_object('primary', accent, 'secondary', accent, 'logo_key', logo_key));
ALTER TABLE doe_settings
    ALTER COLUMN style SET NOT NULL,
    ADD CONSTRAINT chk_doe_settings_style_object CHECK (jsonb_typeof(style) = 'object'),
    DROP CONSTRAINT chk_doe_settings_accent_hex,
    DROP CONSTRAINT chk_doe_settings_logo_key_not_blank,
    DROP COLUMN accent,
    DROP COLUMN logo_key;

ALTER TABLE doe_models
    ADD COLUMN style JSONB NULL,
    ADD CONSTRAINT chk_doe_models_style_object
        CHECK (style IS NULL OR jsonb_typeof(style) = 'object');

-- The model a DOE was made from decides its style at export. Existing DOEs
-- did not keep it and use the organization's.
ALTER TABLE does
    ADD COLUMN model_id UUID NULL REFERENCES doe_models(id) ON DELETE SET NULL,
    ADD COLUMN cover_photo_key TEXT NULL,
    ADD CONSTRAINT chk_does_cover_photo_key_not_blank
        CHECK (cover_photo_key IS NULL OR btrim(cover_photo_key) <> '');
