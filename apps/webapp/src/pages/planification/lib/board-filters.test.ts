import { describe, expect, it } from 'vitest'
import {
	type BoardFilters,
	boardFiltersIncludeSubtasks,
	boardFiltersToQuery,
	boardSearchSchema,
	EMPTY_BOARD_FILTERS,
	hasActiveBoardFilters,
} from '#/pages/planification/lib/board-filters'

function parse(search: unknown): BoardFilters {
	return boardSearchSchema.parse(search)
}

describe('boardSearchSchema', () => {
	it('reads a full filter set out of the search params', () => {
		expect(
			parse({
				project_id: 'project-1',
				assignee_id: 'member-1',
				label_id: 'label-1',
				q: 'toiture',
				unscheduled: true,
			}),
		).toEqual({
			project_id: 'project-1',
			assignee_id: 'member-1',
			label_id: 'label-1',
			q: 'toiture',
			unscheduled: true,
		})
	})

	it('reads an empty search as no filter at all', () => {
		expect(parse({})).toEqual(EMPTY_BOARD_FILTERS)
	})

	/**
	 * The whole point of the `.catch` on every field: a share link that has
	 * rotted still opens a board. Nothing below throws, and nothing below
	 * reaches the API either.
	 */
	it('falls back rather than throwing on a malformed value', () => {
		expect(() =>
			parse({ project_id: 42, unscheduled: 'oui', q: { nope: true } }),
		).not.toThrow()
		expect(
			parse({ project_id: 42, unscheduled: 'oui', q: { nope: true } }),
		).toEqual(EMPTY_BOARD_FILTERS)
	})

	it('keeps the filters it can read when a neighbour is malformed', () => {
		expect(parse({ project_id: 'project-1', unscheduled: 'maybe' })).toEqual({
			...EMPTY_BOARD_FILTERS,
			project_id: 'project-1',
		})
	})

	/**
	 * `?q=` is a *present* filter server-side: it matches every title and it
	 * widens the listing from the roots to every depth. An emptied search box
	 * must not mean that.
	 */
	it('folds an empty or blank search term to no filter', () => {
		expect(parse({ q: '' }).q).toBeUndefined()
		expect(parse({ q: '   ' }).q).toBeUndefined()
		expect(parse({ q: '  toiture  ' }).q).toBe('toiture')
	})

	/** `unscheduled=false` is the default state, so it never belongs in the URL. */
	it('keeps only the on state of the unscheduled toggle', () => {
		expect(parse({ unscheduled: true }).unscheduled).toBe(true)
		expect(parse({ unscheduled: false }).unscheduled).toBeUndefined()
	})

	it('never throws on a search that is not an object at all', () => {
		expect(() => parse(undefined)).not.toThrow()
	})
})

describe('hasActiveBoardFilters', () => {
	it('is false for the empty set', () => {
		expect(hasActiveBoardFilters(EMPTY_BOARD_FILTERS)).toBe(false)
	})

	it('is true as soon as one filter is set', () => {
		expect(
			hasActiveBoardFilters({ ...EMPTY_BOARD_FILTERS, label_id: 'label-1' }),
		).toBe(true)
		expect(
			hasActiveBoardFilters({ ...EMPTY_BOARD_FILTERS, unscheduled: true }),
		).toBe(true)
	})
})

describe('boardFiltersIncludeSubtasks', () => {
	/**
	 * `GET /tasks` answers with the roots when nothing narrows it, and with
	 * every matching task at any depth as soon as something does. The board
	 * mirrors that rule here so it can say so on screen instead of letting
	 * the column counts move unexplained.
	 */
	it('tracks whether the listing reaches past the roots', () => {
		expect(boardFiltersIncludeSubtasks(EMPTY_BOARD_FILTERS)).toBe(false)
		expect(
			boardFiltersIncludeSubtasks({
				...EMPTY_BOARD_FILTERS,
				project_id: 'project-1',
			}),
		).toBe(true)
	})
})

describe('boardFiltersToQuery', () => {
	it('sends nothing but the page when no filter is set', () => {
		expect(boardFiltersToQuery(EMPTY_BOARD_FILTERS, 100)).toEqual({
			page: 1,
			per_page: 100,
		})
	})

	/**
	 * An unknown *or empty* key is a `400` from `GET /tasks`, not an ignored
	 * filter, and this object is also the board query's cache key — so it
	 * carries the filters that are set and strictly nothing else.
	 */
	it('emits only the filters that are set', () => {
		expect(
			boardFiltersToQuery(
				{
					project_id: 'project-1',
					assignee_id: undefined,
					label_id: undefined,
					q: 'toiture',
					unscheduled: true,
				},
				100,
			),
		).toEqual({
			page: 1,
			per_page: 100,
			project_id: 'project-1',
			q: 'toiture',
			unscheduled: true,
		})
	})

	it('never sends parent_task_id', () => {
		expect(
			boardFiltersToQuery({ ...EMPTY_BOARD_FILTERS, project_id: 'p' }, 200),
		).not.toHaveProperty('parent_task_id')
	})

	/** Two different filter sets must never produce the same request object. */
	it('distinguishes one project from another', () => {
		expect(
			boardFiltersToQuery({ ...EMPTY_BOARD_FILTERS, project_id: 'a' }, 200),
		).not.toEqual(
			boardFiltersToQuery({ ...EMPTY_BOARD_FILTERS, project_id: 'b' }, 200),
		)
	})
})
