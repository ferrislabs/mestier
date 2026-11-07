import type { QuoteDisplayStatus } from '#/pages/quotes/lib/quote-status'

export const OWNER_ALL = 'all'
export const OWNER_NONE = 'none'

export type QuoteStatusFilter = 'all' | QuoteDisplayStatus

export interface QuoteFilters {
	query: string
	status: QuoteStatusFilter
	owner: string
}

export const STATUS_FILTERS: readonly QuoteStatusFilter[] = [
	'all',
	'draft',
	'sent',
	'accepted',
	'declined',
	'expired',
	'superseded',
	'cancelled',
]

export function isStatusFilter(value: unknown): value is QuoteStatusFilter {
	return STATUS_FILTERS.some((status) => status === value)
}

export function matchesOwner(ownerId: string | null, owner: string): boolean {
	if (owner === OWNER_ALL) return true
	if (owner === OWNER_NONE) return ownerId === null
	return ownerId === owner
}

export function matchesStatus(
	display: QuoteDisplayStatus,
	filter: QuoteStatusFilter,
): boolean {
	return filter === 'all' || display === filter
}

export function matchesClient(
	customerId: string,
	client: string | null,
): boolean {
	return client === null || customerId === client
}

export function hasActiveFilters(filters: QuoteFilters): boolean {
	return (
		filters.query.trim() !== '' ||
		filters.status !== 'all' ||
		filters.owner !== OWNER_ALL
	)
}
