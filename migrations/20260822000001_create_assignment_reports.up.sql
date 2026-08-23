-- Assignment reports: the correction a field worker files when the actual
-- duration of a task differed from the plan — the record ADR 0002 expects a
-- human to produce once reality and the plan disagree, and that nothing in
-- the schema could hold until now.
--
-- A report is its own aggregate, not a column on `task_assignments`: a column
-- would allow exactly one report ever and would have nowhere to put the
-- arbitration. `task_comments` is the precedent for a small satellite
-- context around the task.
--
-- Composite FK on `(task_assignment_id, org_id)`, so a report referencing
-- another organization's assignment is structurally impossible rather than
-- checked in application code — same device as
-- `20260821000001_create_projects`'s `tasks.project_id`. It needs
-- `task_assignments` to expose a `(id, org_id)` unique target, which it does
-- not have yet.
--
-- `resolution` carries the whole lifecycle. `resolved_by`/`resolved_at` are
-- null exactly when `resolution = 'PENDING'` — an equivalence CHECK rather
-- than two independently nullable columns that could disagree with the state
-- they belong to. `resolution_note` stays a plain optional column: #288
-- makes it an optional note on both APPLIED and DISMISSED, so it is never
-- required to be set merely because the report was resolved.
--
-- One pending report per assignment, as a partial unique index: a worker who
-- changes their mind amends the existing one (`PATCH`) rather than queuing a
-- second opinion.
--
-- Physical deletion of `assignment_reports` is not exposed at this layer.
-- `DELETE /field/assignment-reports/{id}` (see #288) withdraws a pending
-- report by removing its row — there is nothing worth auditing once its
-- author retracts it before anyone acted on it.

CREATE TYPE assignment_report_resolution AS ENUM (
    'PENDING',
    'APPLIED',
    'DISMISSED'
);

ALTER TABLE task_assignments
    ADD CONSTRAINT uq_task_assignments_id_org UNIQUE (id, org_id);

CREATE TABLE assignment_reports (
    id                  UUID                          PRIMARY KEY,
    org_id              UUID                          NOT NULL REFERENCES organizations(id),
    task_assignment_id  UUID                          NOT NULL,
    reported_minutes    INTEGER                       NOT NULL,
    comment             TEXT                          NULL,
    reported_by         UUID                          NOT NULL,
    resolution          assignment_report_resolution  NOT NULL DEFAULT 'PENDING',
    resolved_by         UUID                          NULL,
    resolved_at         TIMESTAMPTZ                   NULL,
    resolution_note     TEXT                          NULL,
    created_at          TIMESTAMPTZ                   NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ                   NOT NULL DEFAULT now(),

    CONSTRAINT fk_assignment_reports_task_assignment
        FOREIGN KEY (task_assignment_id, org_id)
        REFERENCES task_assignments(id, org_id)
        ON DELETE CASCADE,
    CONSTRAINT fk_assignment_reports_reported_by
        FOREIGN KEY (reported_by, org_id)
        REFERENCES organization_members(id, organization_id),
    CONSTRAINT fk_assignment_reports_resolved_by
        FOREIGN KEY (resolved_by, org_id)
        REFERENCES organization_members(id, organization_id),
    CONSTRAINT chk_assignment_reports_minutes_not_negative
        CHECK (reported_minutes >= 0),
    CONSTRAINT chk_assignment_reports_comment_not_blank_when_present
        CHECK (comment IS NULL OR length(btrim(comment)) > 0),
    CONSTRAINT chk_assignment_reports_resolution_note_not_blank_when_present
        CHECK (resolution_note IS NULL OR length(btrim(resolution_note)) > 0),
    -- `resolved_by`/`resolved_at` are null exactly while pending: whichever
    -- side changes first (a resolve call setting them, or a hypothetical
    -- reopen clearing them) cannot land without the other, so the two can
    -- never drift apart to describe a state nothing else agrees with.
    CONSTRAINT chk_assignment_reports_resolution_fields
        CHECK (
            (resolution = 'PENDING' AND resolved_by IS NULL AND resolved_at IS NULL)
            OR
            (resolution <> 'PENDING' AND resolved_by IS NOT NULL AND resolved_at IS NOT NULL)
        )
);

CREATE UNIQUE INDEX uq_assignment_reports_pending_per_assignment
    ON assignment_reports(task_assignment_id)
    WHERE resolution = 'PENDING';

CREATE INDEX idx_assignment_reports_org_id ON assignment_reports(org_id);
CREATE INDEX idx_assignment_reports_task_assignment_id ON assignment_reports(task_assignment_id);
CREATE INDEX idx_assignment_reports_reported_by ON assignment_reports(reported_by);
CREATE INDEX idx_assignment_reports_resolution ON assignment_reports(org_id, resolution);
