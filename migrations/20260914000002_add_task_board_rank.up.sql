-- A task gets a manual position inside its board column: `board_rank`.
--
-- **TEXT, ordered lexicographically, never a number.** The rejected
-- alternative was an `INTEGER position` renumbered on every drop: moving one
-- card between two others would rewrite every row below it in the column, and
-- two people dropping at the same time would interleave their renumberings
-- into an order neither of them asked for. A lexicographic rank is generated
-- strictly between its two neighbours instead, so a move is one `UPDATE` of
-- one row and concurrent moves cannot corrupt each other's positions. The
-- cost is that ranks get longer as a column is churned — see `BoardRank` in
-- `libs/core/src/domain/task/mod.rs` for the growth bound.
--
-- `COLLATE "C"` is load-bearing, not decoration. This database is created
-- with `en_US.utf8`, whose collation is a multi-level linguistic comparison,
-- while the Rust side compares `String`s byte by byte. The domain sorts a
-- board in memory (`sort_by_board_rank`) and PostgreSQL sorts it in `ORDER
-- BY`; if those two disagree on a single pair, cards jump around on reload
-- for no visible reason. `"C"` pins the column to byte order, which is
-- exactly what `Ord for BoardRank` does. The generator's alphabet is
-- lowercase-only for the same reason, one layer up: case is where locale
-- collations differ from byte order most often, so no rank ever contains an
-- uppercase letter to begin with.
--
-- `NULL` is allowed and there is no backfill. A task has no rank until
-- somebody drags it; `NULL` sorts after every ranked task (PostgreSQL's
-- default `NULLS LAST` for `ASC`, mirrored in the domain), so an untouched
-- column reads in whatever order the board's secondary key gives it, and the
-- first drop is what starts assigning ranks.
ALTER TABLE tasks ADD COLUMN board_rank TEXT COLLATE "C" NULL;

-- Mirrors the domain's own invariants on a rank, the way
-- `chk_tasks_expenses_label_required` mirrors `normalize_expenses`:
--
--   * non-empty — the empty string sorts before every rank and is not a
--     position, it is a missing one, which is what `NULL` is for;
--   * digits `0-9` and lowercase `a-z` only — the generator's base-36
--     alphabet, whose byte order is its digit order;
--   * never ends in `0` — the lowest digit. `"1"` and `"10"` denote the same
--     fraction, so a trailing zero would make two distinct strings name one
--     position, and nothing at all can be inserted before a rank that is all
--     zeroes. The generator never emits one; this stops anything else from
--     writing one by hand.
ALTER TABLE tasks
    ADD CONSTRAINT chk_tasks_board_rank_shape
        CHECK (board_rank IS NULL OR board_rank ~ '^[0-9a-z]*[1-9a-z]$');

-- The board's read: one column of one organization, in rank order. Leading
-- with `(org_id, status)` because every query in this codebase is scoped by
-- `org_id` first, and a board column *is* a status; `board_rank` last makes
-- the index cover the ordering as well as the filter, so a column renders
-- from one index scan with no sort. `NULLS LAST` is the default for an
-- ascending b-tree column, so `ORDER BY board_rank ASC NULLS LAST` uses this
-- index as it stands.
CREATE INDEX idx_tasks_org_id_status_board_rank
    ON tasks (org_id, status, board_rank);
