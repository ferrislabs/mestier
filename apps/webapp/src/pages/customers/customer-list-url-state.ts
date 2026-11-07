import type { Customer } from '#/hooks/use-customers'
import { NO_TERMS, type SearchTerms } from '#/pages/customers/lib/accounts'
import { accountStatusOf } from '#/pages/customers/types'

export type StatusFilter = 'all' | 'client' | 'prospect' | 'archived'

export const OWNER_ALL = 'all'

export const OWNER_NONE = 'none'

export type ClientFilters = {
	query: string
	status: StatusFilter
	owner: string
}

export const DEFAULT_FILTERS: ClientFilters = {
	query: '',
	status: 'all',
	owner: OWNER_ALL,
}

export type ClientsSearch = {
	statut?: Exclude<StatusFilter, 'all'>
	q?: string
	responsable?: string
}

const STATUS_FILTERS: readonly string[] = [
	'all',
	'client',
	'prospect',
	'archived',
]

export function isStatusFilter(value: unknown): value is StatusFilter {
	return typeof value === 'string' && STATUS_FILTERS.includes(value)
}

const MEMBER_ID_PATTERN =
	/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i

export function isOwnerFilter(value: unknown): value is string {
	return (
		typeof value === 'string' &&
		(value === OWNER_NONE || MEMBER_ID_PATTERN.test(value))
	)
}

function queryOf(value: unknown): string {
	if (typeof value === 'string') return value
	return typeof value === 'number' ? String(value) : ''
}

export function validateClientsSearch(
	raw: Record<string, unknown>,
): ClientsSearch {
	const q = queryOf(raw.q)
	const statut = raw.statut
	return {
		...(isStatusFilter(statut) && statut !== 'all' ? { statut } : {}),
		...(q.trim() !== '' ? { q } : {}),
		...(isOwnerFilter(raw.responsable) ? { responsable: raw.responsable } : {}),
	}
}

export function filtersOfSearch(search: ClientsSearch): ClientFilters {
	return {
		query: search.q ?? DEFAULT_FILTERS.query,
		status: search.statut ?? DEFAULT_FILTERS.status,
		owner: search.responsable ?? DEFAULT_FILTERS.owner,
	}
}

export function searchOfFilters(filters: ClientFilters): ClientsSearch {
	return validateClientsSearch({
		statut: filters.status,
		q: filters.query,
		responsable: filters.owner,
	})
}

export function normalizeText(text: string): string {
	return text.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase().trim()
}

function compactDigits(text: string): string {
	return text.replace(/[\s.-]/g, '')
}

function matchesQuery(
	customer: Customer,
	query: string,
	terms: SearchTerms,
): boolean {
	if (query === '') return true
	const haystack = [customer.name, customer.email ?? '', ...terms.text]
		.map(normalizeText)
		.join('\n')
	if (haystack.includes(query)) return true
	const digits = compactDigits(query)
	if (!/^\+?\d{3,}$/.test(digits)) return false
	return [customer.phone ?? '', ...terms.phones].some((phone) =>
		compactDigits(phone).includes(digits),
	)
}

function matchesStatus(customer: Customer, status: StatusFilter): boolean {
	const own = accountStatusOf(customer.status)
	return status === 'all' ? own !== 'archived' : own === status
}

function matchesOwner(customer: Customer, owner: string): boolean {
	if (owner === OWNER_ALL) return true
	if (owner === OWNER_NONE) return !customer.owner_member_id
	return customer.owner_member_id === owner
}

export function filterCustomers(
	customers: readonly Customer[],
	filters: ClientFilters,
	termsOf: (customerId: string) => SearchTerms = () => NO_TERMS,
): Customer[] {
	const query = normalizeText(filters.query)
	return customers
		.filter(
			(customer) =>
				matchesStatus(customer, filters.status) &&
				matchesOwner(customer, filters.owner) &&
				matchesQuery(customer, query, termsOf(customer.id)),
		)
		.sort((a, b) => a.name.localeCompare(b.name, 'fr'))
}

export function hasActiveFilters(filters: ClientFilters): boolean {
	return (
		filters.query.trim() !== '' ||
		filters.status !== DEFAULT_FILTERS.status ||
		filters.owner !== DEFAULT_FILTERS.owner
	)
}

function countLabel(count: number, singular: string): string {
	return `${count} ${singular}${count > 1 ? 's' : ''}`
}

export function customersSummary(customers: readonly Customer[]): string {
	const clients = customers.filter((customer) => customer.status === 'CLIENT')
	const prospects = customers.filter(
		(customer) => customer.status === 'PROSPECT',
	)
	return `${countLabel(clients.length, 'client')}, ${countLabel(prospects.length, 'prospect')}`
}
