UPDATE automation.workflow AS workflow
SET enabled = false,
    updated_at = now()
FROM automation.workflow_version AS version
WHERE version.id = workflow.current_version_id
  AND workflow.enabled
  AND version.graph::text ~ '\{\{[^}]*\yconnectors\.[A-Za-z0-9_]';
