import { UNASSIGNED_FILTER } from '#/pages/support/lib/filters'
import { priorityRank } from '#/pages/support/lib/priorities'
import type {
	SupportFilters,
	SupportPriority,
	SupportStatus,
	SupportTicketVM,
} from '#/pages/support/types'

export const ALL_FILTER = 'all'
export const ACTIVE_STATUS_FILTER = 'active'

export type StatusFilterValue =
	| typeof ACTIVE_STATUS_FILTER
	| 'all'
	| SupportStatus

export interface TicketFilterValues {
	assignee: string
	status: StatusFilterValue
	priority: SupportPriority | typeof ALL_FILTER
	badge: string
}

export const DEFAULT_FILTER_VALUES: TicketFilterValues = {
	assignee: ALL_FILTER,
	status: ACTIVE_STATUS_FILTER,
	priority: ALL_FILTER,
	badge: ALL_FILTER,
}

export function filterValuesOf(filters: SupportFilters): TicketFilterValues {
	return {
		assignee: filters.assignee ?? ALL_FILTER,
		status: filters.status ?? ACTIVE_STATUS_FILTER,
		priority: filters.priority ?? ALL_FILTER,
		badge: filters.badge ?? ALL_FILTER,
	}
}

export function filterPatchOf(
	patch: Partial<TicketFilterValues>,
): Partial<SupportFilters> {
	const result: Partial<SupportFilters> = {}

	if ('assignee' in patch) {
		result.assignee = patch.assignee === ALL_FILTER ? undefined : patch.assignee
	}
	if ('status' in patch) {
		result.status =
			patch.status === ACTIVE_STATUS_FILTER
				? undefined
				: (patch.status as SupportFilters['status'])
	}
	if ('priority' in patch) {
		result.priority =
			patch.priority === ALL_FILTER
				? undefined
				: (patch.priority as SupportPriority)
	}
	if ('badge' in patch) {
		result.badge = patch.badge === ALL_FILTER ? undefined : patch.badge
	}

	return result
}

export function activeFilterCount(
	values: TicketFilterValues,
	showStatus: boolean,
): number {
	return (
		Number(values.assignee !== DEFAULT_FILTER_VALUES.assignee) +
		Number(showStatus && values.status !== DEFAULT_FILTER_VALUES.status) +
		Number(values.priority !== DEFAULT_FILTER_VALUES.priority) +
		Number(values.badge !== DEFAULT_FILTER_VALUES.badge)
	)
}

export type TicketSort = 'recent' | 'priority'

function normalize(value: string): string {
	return value
		.normalize('NFD')
		.replace(/\p{Diacritic}/gu, '')
		.toLowerCase()
		.trim()
}

function byRecentActivity(a: SupportTicketVM, b: SupportTicketVM): number {
	return (
		Date.parse(b.updatedAt) - Date.parse(a.updatedAt) || b.number - a.number
	)
}

function byPriorityThenRecent(a: SupportTicketVM, b: SupportTicketVM): number {
	return (
		priorityRank(a.priority) - priorityRank(b.priority) ||
		byRecentActivity(a, b)
	)
}

export interface TicketViewFilters {
	assignee: string
	status: StatusFilterValue
	priority: SupportPriority | typeof ALL_FILTER
	query: string
	sort: TicketSort
}

export function filterTicketViews(
	tickets: readonly SupportTicketVM[],
	filters: TicketViewFilters,
): SupportTicketVM[] {
	const needle = normalize(filters.query.replace(/^#/, ''))

	return tickets
		.filter(
			(ticket) =>
				filters.assignee !== UNASSIGNED_FILTER || ticket.assignee === null,
		)
		.filter((ticket) => {
			if (filters.status === 'all') return true
			if (filters.status === ACTIVE_STATUS_FILTER)
				return ticket.status !== 'resolved'
			return ticket.status === filters.status
		})
		.filter(
			(ticket) =>
				filters.priority === ALL_FILTER || ticket.priority === filters.priority,
		)
		.filter(
			(ticket) =>
				needle === '' ||
				String(ticket.number) === needle ||
				normalize(
					`${ticket.subject} ${ticket.requester.name} ${ticket.requester.email ?? ''} ${ticket.externalId ?? ''}`,
				).includes(needle),
		)
		.sort(filters.sort === 'priority' ? byPriorityThenRecent : byRecentActivity)
}

export function countByAssignee(
	tickets: readonly SupportTicketVM[],
): ReadonlyMap<string | null, number> {
	const counts = new Map<string | null, number>()

	for (const ticket of tickets) {
		if (ticket.status === 'resolved') continue
		const key = ticket.assignee?.id ?? null
		counts.set(key, (counts.get(key) ?? 0) + 1)
	}

	return counts
}

export function countOpen(tickets: readonly SupportTicketVM[]): number {
	return tickets.filter((ticket) => ticket.status !== 'resolved').length
}
