ALTER TABLE survey.survey ADD COLUMN campaign_id UUID NULL;

UPDATE survey.survey s
SET campaign_id = earliest.campaign_id
FROM (
    SELECT DISTINCT ON (survey_id) survey_id, campaign_id
    FROM survey.survey_campaign
    ORDER BY survey_id, position, attached_at
) earliest
WHERE earliest.survey_id = s.id;

ALTER TABLE survey.survey
    ADD CONSTRAINT fk_survey_campaign
        FOREIGN KEY (org_id, campaign_id) REFERENCES survey.campaign (org_id, id)
        ON DELETE RESTRICT;

CREATE INDEX idx_survey_org_campaign ON survey.survey (org_id, campaign_id);

DROP TABLE survey.survey_campaign;
