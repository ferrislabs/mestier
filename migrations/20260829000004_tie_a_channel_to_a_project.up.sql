-- Ties a chat channel to the project it belongs to (#345, first half of
-- #286: project channels — permission handling is the existing
-- EVERYONE/ROLE/MEMBER overwrite machinery on `chat.channel_permission_overwrite`,
-- nothing new needed there; automatically granting access to whoever is
-- assigned to the project is #346, not this one).
--
-- Composite FK against `(project_id, org_id)`, not a bare FK against
-- `projects(id)`: mirrors `fk_tasks_project` from
-- `20260821000001_create_projects` exactly, and for the same reason — a bare
-- FK on the id alone would let a channel point at another organization's
-- project, representable rather than merely forbidden by application code.
--
-- No `ON DELETE` clause, also like `fk_tasks_project`: nothing in this
-- codebase hard-deletes a project (`DELETE /projects/{id}` archives — see
-- `handlers-planning`'s `archive.rs`), so a hard delete of a `projects` row
-- that still has a channel is refused by the database rather than silently
-- orphaning the channel or cascading into a conversation nobody asked to
-- destroy.
ALTER TABLE chat.channels ADD COLUMN project_id UUID NULL;

ALTER TABLE chat.channels
    ADD CONSTRAINT fk_channels_project
        FOREIGN KEY (project_id, org_id) REFERENCES projects(id, org_id);

-- One channel per project, enforced here rather than by a service-layer
-- check a second write path could bypass. Partial: most projects (internal
-- admin, a quick job) never grow a channel at all, and a bare unique index
-- would refuse every one of them a second `NULL`.
CREATE UNIQUE INDEX uq_channels_project_id ON chat.channels(project_id) WHERE project_id IS NOT NULL;

-- A THREAD already hangs off a parent channel
-- (`chk_channels_thread_requires_parent`). Letting it independently carry a
-- `project_id` would be a second, contradictory hierarchy for the same row —
-- only a TEXT channel may be a project's channel.
ALTER TABLE chat.channels
    ADD CONSTRAINT chk_channels_thread_no_project
        CHECK (channel_type <> 'THREAD' OR project_id IS NULL);
