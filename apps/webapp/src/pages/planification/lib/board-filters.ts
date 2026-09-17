import { z } from 'zod'

/**
 * A text filter as the URL carries it: absent, or a non-empty trimmed
 * string. `''` is folded to `undefined` rather than kept, and that is a
 * server-side rule as much as a cosmetic one — `GET /tasks` treats a present
 * `q=` as a filter that matches every title *and* widens the listing from
 * roots to every depth (see `ListTasksQuery::narrows_beyond_the_hierarchy`).
 * An emptied search box must mean "no filter", not "a filter that changes
 * the shape of the board without narrowing it".
 */
const optionalText = z.string().trim().min(1).optional().catch(undefined)

/**
 * Validates `/planification/board?project_id=&assignee_id=&label_id=&q=&unscheduled=`
 * — the board's whole filter state, which lives in the URL and nowhere else
 * so that a narrowed board is a link (#467).
 *
 * Every field carries `.catch(...)`, the same treatment `planningSearchSchema`
 * gives the calendar and for the same reason: a hand-edited or stale share
 * link should land on the unfiltered board rather than blank-page the user
 * on a router error. Nothing here throws.
 *
 * Every field is also `undefined` when absent rather than defaulted to `''`
 * or `false`: TanStack drops an `undefined` search value from the address,
 * so "no filter" is spelled by the key simply not being there. That is what
 * keeps a cleared board's URL clean — see {@link EMPTY_BOARD_FILTERS}.
 *
 * `unscheduled` is a `literal(true)` and not a `boolean` on purpose. `false`
 * is the default state, so it never needs to appear in the address, and
 * `?unscheduled=false` sent to the API would be a *present* filter — which
 * widens the listing to subtasks for no gain. Folding it to `undefined` here
 * means the toggle is either on and in the URL, or off and absent.
 */
const NO_FILTERS = {
	project_id: undefined,
	assignee_id: undefined,
	label_id: undefined,
	q: undefined,
	unscheduled: undefined,
}

export const boardSearchSchema = z
	.object({
		project_id: optionalText,
		assignee_id: optionalText,
		label_id: optionalText,
		q: optionalText,
		unscheduled: z.literal(true).optional().catch(undefined),
	})
	// The outer `.catch` only ever fires for a search that is not an object
	// at all — the router always hands one over, so this is belt and braces
	// rather than a live path. It is here so that "this schema never throws"
	// holds as written, without a reader having to reason about the caller.
	.catch(() => NO_FILTERS)

export type BoardFilters = z.infer<typeof boardSearchSchema>

/**
 * Every filter key, explicitly undefined.
 *
 * Navigating with this — rather than with `{}` — is what clears the address:
 * a key missing from the object handed to `navigate` keeps whatever the
 * previous search had, while a key present and `undefined` removes it.
 */
export const EMPTY_BOARD_FILTERS: BoardFilters = NO_FILTERS

/** True when at least one filter narrows the board. */
export function hasActiveBoardFilters(filters: BoardFilters): boolean {
	return Object.values(filters).some((value) => value !== undefined)
}

/**
 * Whether the listing the current filters produce reaches past the roots.
 *
 * `GET /tasks` with no `parent_task_id` returns root tasks — unless any
 * other filter is present, in which case it returns matching tasks at any
 * depth. That is deliberate backend behaviour (a project's board that hid
 * its subtasks would be quietly missing rows), but it means the board's
 * contents change kind the moment a filter is applied, and the screen says
 * so rather than letting the counts move unexplained.
 */
export function boardFiltersIncludeSubtasks(filters: BoardFilters): boolean {
	return hasActiveBoardFilters(filters)
}

export interface BoardTasksQuery {
	page: number
	per_page: number
	project_id?: string
	assignee_id?: string
	label_id?: string
	q?: string
	unscheduled?: true
}

/**
 * The filters as `GET /tasks` query parameters.
 *
 * Only keys that are actually set are emitted. Two reasons, both load-bearing:
 * an unknown or unparsable parameter is a `400` from that endpoint rather
 * than an ignored filter, and this object is part of the board query's cache
 * key — the one the optimistic move in `useMoveBoardTask` writes into — so
 * it has to be exactly the request that was sent, no more.
 *
 * `parent_task_id` is never sent: the board shows what the filters select,
 * and the roots-only default is what the absence of that key means.
 */
export function boardFiltersToQuery(
	filters: BoardFilters,
	perPage: number,
): BoardTasksQuery {
	const query: BoardTasksQuery = { page: 1, per_page: perPage }

	if (filters.project_id) query.project_id = filters.project_id
	if (filters.assignee_id) query.assignee_id = filters.assignee_id
	if (filters.label_id) query.label_id = filters.label_id
	if (filters.q) query.q = filters.q
	if (filters.unscheduled) query.unscheduled = true

	return query
}
