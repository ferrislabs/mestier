-- No-op: the up migration only sets `enabled = false` on workflows whose current
-- version holds a `connectors.<id>` expression, and does not record which ones it
-- touched. Re-enabling "the same" workflows is impossible without that record, and
-- re-enabling a workflow whose expressions the engine now refuses would only make
-- its runs fail. A user re-enables a workflow once they rewrote its expressions.
SELECT 1;
