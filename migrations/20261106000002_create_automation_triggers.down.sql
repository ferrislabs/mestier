DROP INDEX IF EXISTS automation.idx_automation_schedule_state_org;
DROP INDEX IF EXISTS automation.idx_automation_schedule_state_due;
DROP TABLE IF EXISTS automation.schedule_state;

DROP INDEX IF EXISTS automation.idx_automation_webhook_route_org;
DROP TABLE IF EXISTS automation.webhook_route;
