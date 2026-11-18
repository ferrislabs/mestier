CREATE TABLE survey.survey_campaign (
    org_id      UUID        NOT NULL,
    survey_id   UUID        NOT NULL,
    campaign_id UUID        NOT NULL,
    position    INTEGER     NOT NULL DEFAULT 0,
    attached_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (survey_id, campaign_id),
    CONSTRAINT fk_survey_campaign_survey
        FOREIGN KEY (org_id, survey_id) REFERENCES survey.survey (org_id, id)
        ON DELETE CASCADE,
    CONSTRAINT fk_survey_campaign_campaign
        FOREIGN KEY (org_id, campaign_id) REFERENCES survey.campaign (org_id, id)
        ON DELETE RESTRICT,
    CONSTRAINT chk_survey_campaign_position CHECK (position >= 0)
);

CREATE INDEX idx_survey_campaign_campaign ON survey.survey_campaign (org_id, campaign_id);

INSERT INTO survey.survey_campaign (org_id, survey_id, campaign_id)
SELECT org_id, id, campaign_id
FROM survey.survey
WHERE campaign_id IS NOT NULL;

DROP INDEX survey.idx_survey_org_campaign;

ALTER TABLE survey.survey DROP CONSTRAINT fk_survey_campaign;

ALTER TABLE survey.survey DROP COLUMN campaign_id;
