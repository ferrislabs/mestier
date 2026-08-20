-- Marks a stretch whose end was declared after the fact.
--
-- An employee who forgets to clock off cannot be clocked off truthfully: the
-- moment is gone. The field app asks them the next morning, which is the best
-- source available, but it is a recollection and not a measurement. This column
-- is what lets the profitability report say so rather than presenting the two
-- as the same kind of fact.
--
-- Not derived from `updated_at`: that column records when the row changed, and
-- any later correction would make an honest entry look reconstructed.

ALTER TABLE time_entries
    ADD COLUMN closed_after_the_fact BOOLEAN NOT NULL DEFAULT false;

COMMENT ON COLUMN time_entries.closed_after_the_fact IS
    'True when ended_at was declared on a later day than the work, so the duration is a recollection rather than a measurement.';
