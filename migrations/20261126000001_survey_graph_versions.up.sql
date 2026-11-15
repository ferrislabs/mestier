ALTER TABLE survey.survey ADD COLUMN graph_version INTEGER NULL;

UPDATE survey.survey SET graph_version = 1 WHERE graph IS NOT NULL;

ALTER TABLE survey.survey
    ADD CONSTRAINT chk_survey_graph_version CHECK (
        (graph IS NULL) = (graph_version IS NULL)
        AND (graph_version IS NULL OR graph_version > 0)
    );

CREATE TABLE survey.survey_version (
    org_id     UUID        NOT NULL,
    survey_id  UUID        NOT NULL,
    version    INTEGER     NOT NULL,
    graph      JSONB       NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (survey_id, version),
    CONSTRAINT fk_survey_version_survey
        FOREIGN KEY (org_id, survey_id) REFERENCES survey.survey (org_id, id)
        ON DELETE CASCADE,
    CONSTRAINT chk_survey_version_positive CHECK (version > 0)
);

ALTER TABLE survey.response ADD COLUMN graph_version INTEGER NULL;

UPDATE survey.response SET graph_version = 1 WHERE status = 'COMPLETED';

INSERT INTO survey.survey_version (org_id, survey_id, version, graph)
SELECT s.org_id, s.id, 1, s.graph
FROM survey.survey s
WHERE s.graph IS NOT NULL
  AND EXISTS (
      SELECT 1 FROM survey.response r
      WHERE r.org_id = s.org_id AND r.survey_id = s.id AND r.status = 'COMPLETED'
  );

ALTER TABLE survey.response
    ADD CONSTRAINT chk_response_graph_version CHECK (
        (status = 'COMPLETED') = (graph_version IS NOT NULL)
        AND (graph_version IS NULL OR graph_version > 0)
    );
