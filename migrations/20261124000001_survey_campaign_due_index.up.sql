CREATE INDEX idx_campaign_due
    ON survey.campaign (status, starts_at, ends_at)
    WHERE status <> 'ENDED';
