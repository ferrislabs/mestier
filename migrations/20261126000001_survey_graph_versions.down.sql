ALTER TABLE survey.response DROP CONSTRAINT chk_response_graph_version;
ALTER TABLE survey.response DROP COLUMN graph_version;
DROP TABLE survey.survey_version;
ALTER TABLE survey.survey DROP CONSTRAINT chk_survey_graph_version;
ALTER TABLE survey.survey DROP COLUMN graph_version;
