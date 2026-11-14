CREATE SCHEMA survey;

CREATE TABLE survey.campaign (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id     UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name       TEXT        NOT NULL,
    starts_at  TIMESTAMPTZ NULL,
    ends_at    TIMESTAMPTZ NULL,
    status     TEXT        NOT NULL,
    metadata   JSONB       NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_campaign_dates CHECK (starts_at IS NULL OR ends_at IS NULL OR starts_at < ends_at),
    CONSTRAINT chk_campaign_status CHECK (status IN ('SCHEDULED', 'RUNNING', 'ENDED'))
);

CREATE TABLE survey.survey (
    id                       UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id                   UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    campaign_id              UUID        NULL REFERENCES survey.campaign(id),
    title                    TEXT        NOT NULL,
    description              TEXT        NULL,
    visibility               TEXT        NOT NULL,
    graph                    JSONB       NOT NULL,
    closing_message          TEXT        NULL,
    allow_multiple_responses BOOLEAN     NOT NULL DEFAULT false,
    response_cap             INTEGER     NULL,
    metadata                 JSONB       NOT NULL DEFAULT '{}',
    status                   TEXT        NOT NULL,
    public_slug              TEXT        NULL UNIQUE,
    opened_at                TIMESTAMPTZ NULL,
    closed_at                TIMESTAMPTZ NULL,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_survey_visibility CHECK (visibility IN ('PUBLIC', 'MEMBERS')),
    CONSTRAINT chk_survey_status CHECK (status IN ('DRAFT', 'OPENED', 'CLOSED')),
    CONSTRAINT chk_survey_response_cap CHECK (response_cap IS NULL OR response_cap > 0)
);

CREATE INDEX idx_survey_org_campaign ON survey.survey (org_id, campaign_id);

CREATE TABLE survey.response (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id           UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    survey_id        UUID        NOT NULL REFERENCES survey.survey(id),
    member_id        UUID        NULL REFERENCES organization_members(id),
    answers          JSONB       NOT NULL DEFAULT '{}',
    status           TEXT        NOT NULL,
    resume_token     TEXT        NULL UNIQUE,
    identifier_value TEXT        NULL,
    completed_at     TIMESTAMPTZ NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_response_status CHECK (status IN ('IN_PROGRESS', 'COMPLETED'))
);

CREATE INDEX idx_response_org_survey ON survey.response (org_id, survey_id);
