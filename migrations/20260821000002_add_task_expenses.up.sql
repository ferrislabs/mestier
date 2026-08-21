-- Free-form expenses on a task.
--
-- "Mission in Clermont, 9:00 to 12:00, travel included" had nowhere to put the
-- travel. Across the previous 94 migrations there is no expense table, no
-- kilometric scale, nothing. `service_rates` looks close but is a sales
-- catalogue feeding quotes: reusing it would mix a price with a cost, which is
-- the one thing the profitability model cannot afford.
--
-- One amount and one label, typed by whoever plans the task. A kilometric scale
-- is fairer and stays rejected for now, because it only works if somebody
-- actually enters distances (see docs/adr/0002-planned-cost-model.md).
--
-- The amount lives on the task, never on the project: a project's expenses are
-- the sum of its tasks', which keeps one place to enter a number and one place
-- to read it.

ALTER TABLE tasks
    ADD COLUMN expenses_cents INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN expenses_label TEXT NULL;

ALTER TABLE tasks
    ADD CONSTRAINT chk_tasks_expenses_not_negative
        CHECK (expenses_cents >= 0),
    -- An amount with no label cannot be audited three months later, so the
    -- schema refuses it rather than leaving somebody to guess what the 45 euros
    -- were. Zero keeps no label: clearing the amount clears the reason with it.
    ADD CONSTRAINT chk_tasks_expenses_label_required
        CHECK ((expenses_cents = 0) = (expenses_label IS NULL)),
    ADD CONSTRAINT chk_tasks_expenses_label_not_blank
        CHECK (expenses_label IS NULL OR btrim(expenses_label) <> '');
