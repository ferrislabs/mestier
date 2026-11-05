import { describe, expect, it } from 'vitest'
import {
	activeFilterCount,
	countByAssignee,
	countOpen,
	DEFAULT_FILTER_VALUES,
	filterPatchOf,
	filterTicketViews,
	filterValuesOf,
	type TicketViewFilters,
} from '#/pages/support/lib/ticket-filters'
import type { SupportTicketVM } from '#/pages/support/types'

function ticket(overrides: Partial<SupportTicketVM> = {}): SupportTicketVM {
	return {
		id: 't-1',
		number: 1,
		numberLabel: '#1',
		subject: 'Fuite sous évier',
		status: 'new',
		priority: 'normal',
		assignee: null,
		requester: { name: 'Claire Fontaine', email: 'claire@fontaine.fr' },
		badges: [],
		displayFields: [],
		customFields: [],
		preview: '',
		externalId: null,
		sourceLabel: 'Saisie manuelle',
		createdAt: '2026-10-01T07:00:00Z',
		updatedAt: '2026-10-01T07:00:00Z',
		...overrides,
	}
}

const BASE: TicketViewFilters = {
	assignee: 'all',
	status: 'all',
	priority: 'all',
	query: '',
	sort: 'recent',
}

const alix = { id: 'member-1', name: 'Alix Martin', hue: 10 }

describe('filterTicketViews', () => {
	const tickets = [
		ticket({ id: 'a', number: 1, status: 'new', assignee: alix }),
		ticket({ id: 'b', number: 2, status: 'resolved', priority: 'urgent' }),
		ticket({
			id: 'c',
			number: 3,
			status: 'in_progress',
			subject: 'Devis pergola',
		}),
	]

	it('keeps everything with the "all" filters', () => {
		expect(filterTicketViews(tickets, BASE)).toHaveLength(3)
	})

	it('hides closed tickets with the active status filter', () => {
		expect(
			filterTicketViews(tickets, { ...BASE, status: 'active' }).map(
				(t) => t.id,
			),
		).toEqual(expect.not.arrayContaining(['b']))
	})

	it('keeps one status', () => {
		expect(
			filterTicketViews(tickets, { ...BASE, status: 'resolved' }).map(
				(t) => t.id,
			),
		).toEqual(['b'])
	})

	it('keeps unassigned tickets only for the "none" assignee', () => {
		expect(
			filterTicketViews(tickets, { ...BASE, assignee: 'none' }).map(
				(t) => t.id,
			),
		).toEqual(expect.not.arrayContaining(['a']))
		expect(
			filterTicketViews(tickets, { ...BASE, assignee: 'member-1' }),
		).toHaveLength(3)
	})

	it('filters on priority', () => {
		expect(
			filterTicketViews(tickets, { ...BASE, priority: 'urgent' }).map(
				(t) => t.id,
			),
		).toEqual(['b'])
	})

	it('searches the number, the subject and the requester without regard to accents', () => {
		expect(
			filterTicketViews(tickets, { ...BASE, query: '#3' }).map((t) => t.id),
		).toEqual(['c'])
		expect(
			filterTicketViews(tickets, { ...BASE, query: 'PERGOLA' }).map(
				(t) => t.id,
			),
		).toEqual(['c'])
		expect(
			filterTicketViews(tickets, { ...BASE, query: 'evier' }),
		).toHaveLength(2)
		expect(
			filterTicketViews(tickets, { ...BASE, query: 'fontaine' }),
		).toHaveLength(3)
	})

	it('sorts by recent activity, then by number', () => {
		const sorted = filterTicketViews(
			[
				ticket({ id: 'old', number: 1, updatedAt: '2026-10-01T00:00:00Z' }),
				ticket({ id: 'new', number: 2, updatedAt: '2026-10-03T00:00:00Z' }),
				ticket({ id: 'tie', number: 3, updatedAt: '2026-10-01T00:00:00Z' }),
			],
			BASE,
		)

		expect(sorted.map((t) => t.id)).toEqual(['new', 'tie', 'old'])
	})

	it('sorts by priority first when asked', () => {
		const sorted = filterTicketViews(
			[
				ticket({
					id: 'low',
					priority: 'low',
					updatedAt: '2026-10-05T00:00:00Z',
				}),
				ticket({
					id: 'urgent',
					priority: 'urgent',
					updatedAt: '2026-10-01T00:00:00Z',
				}),
			],
			{ ...BASE, sort: 'priority' },
		)

		expect(sorted.map((t) => t.id)).toEqual(['urgent', 'low'])
	})
})

describe('counts', () => {
	const tickets = [
		ticket({ id: 'a', assignee: alix }),
		ticket({ id: 'b', assignee: alix }),
		ticket({ id: 'c' }),
		ticket({ id: 'd', status: 'resolved' }),
	]

	it('counts open tickets by assignee, unassigned under null', () => {
		const counts = countByAssignee(tickets)

		expect(counts.get('member-1')).toBe(2)
		expect(counts.get(null)).toBe(1)
	})

	it('counts the tickets that are not closed', () => {
		expect(countOpen(tickets)).toBe(3)
	})
})

describe('filter values', () => {
	it('reads the defaults from an empty filter set', () => {
		expect(filterValuesOf({})).toEqual(DEFAULT_FILTER_VALUES)
	})

	it('turns defaults back into undefined search params', () => {
		expect(
			filterPatchOf({
				assignee: 'all',
				status: 'active',
				priority: 'all',
				badge: 'all',
			}),
		).toEqual({
			assignee: undefined,
			status: undefined,
			priority: undefined,
			badge: undefined,
		})
	})

	it('keeps real choices as search params', () => {
		expect(filterPatchOf({ status: 'resolved', priority: 'high' })).toEqual({
			status: 'resolved',
			priority: 'high',
		})
	})

	it('counts the filters that differ from their default', () => {
		const values = {
			...DEFAULT_FILTER_VALUES,
			assignee: 'none',
			status: 'all' as const,
		}

		expect(activeFilterCount(values, true)).toBe(2)
		expect(activeFilterCount(values, false)).toBe(1)
	})
})
