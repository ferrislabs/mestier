import type { Customer, CustomerPipelineStage } from '#/hooks/use-customers'
import type {
	AccountExtras,
	LeadSource,
} from '#/pages/customers/feature/mocks/account-extras'
import { PEOPLE } from '#/pages/customers/feature/mocks/people'
import { LEAD_SOURCES } from '#/pages/customers/types'

export const ALL = 'all'

export type ClosedFilter = 'tous' | 'gagne' | 'perdu' | 'aucun'

export type DealFilters = {
	query: string
	owner: string
	source: LeadSource | typeof ALL
	closed: ClosedFilter
}

export type PipelineSearch = {
	q?: string
	responsable?: string
	source?: LeadSource
	termine?: Exclude<ClosedFilter, 'tous'>
}

export const PIPELINE_STAGES: readonly CustomerPipelineStage[] = [
	'NEW',
	'CONTACTED',
	'QUALIFIED',
	'QUOTE_SENT',
	'WON',
	'LOST',
]

export const DEFAULT_FILTERS: DealFilters = {
	query: '',
	owner: ALL,
	source: ALL,
	closed: 'tous',
}

function findSource(value: unknown): LeadSource | undefined {
	return LEAD_SOURCES.find((source) => source === value)
}

function findClosed(value: unknown): Exclude<ClosedFilter, 'tous'> | undefined {
	return value === 'gagne' || value === 'perdu' || value === 'aucun'
		? value
		: undefined
}

function findOwner(value: unknown): string | undefined {
	return PEOPLE.find((person) => person.id === value)?.id
}

export function parseSearch(raw: Record<string, unknown>): PipelineSearch {
	const owner = findOwner(raw.responsable)
	const source = findSource(raw.source)
	const closed = findClosed(raw.termine)
	return {
		...(typeof raw.q === 'string' && raw.q !== '' ? { q: raw.q } : {}),
		...(owner ? { responsable: owner } : {}),
		...(source ? { source } : {}),
		...(closed ? { termine: closed } : {}),
	}
}

export function filtersFromSearch(search: PipelineSearch): DealFilters {
	return {
		query: search.q ?? DEFAULT_FILTERS.query,
		owner: search.responsable ?? DEFAULT_FILTERS.owner,
		source: search.source ?? DEFAULT_FILTERS.source,
		closed: search.termine ?? DEFAULT_FILTERS.closed,
	}
}

export function searchFromFilters(filters: DealFilters): PipelineSearch {
	return {
		...(filters.query !== '' ? { q: filters.query } : {}),
		...(filters.owner !== ALL ? { responsable: filters.owner } : {}),
		...(filters.source !== ALL ? { source: filters.source } : {}),
		...(filters.closed !== 'tous' ? { termine: filters.closed } : {}),
	}
}

export function hasActiveFilters(filters: DealFilters): boolean {
	return (
		filters.query.trim() !== '' ||
		filters.owner !== ALL ||
		filters.source !== ALL ||
		filters.closed !== 'tous'
	)
}

export function closedFilterFor(won: boolean, lost: boolean): ClosedFilter {
	if (won && lost) return 'tous'
	if (won) return 'gagne'
	if (lost) return 'perdu'
	return 'aucun'
}

export function showsWon(closed: ClosedFilter): boolean {
	return closed === 'tous' || closed === 'gagne'
}

export function showsLost(closed: ClosedFilter): boolean {
	return closed === 'tous' || closed === 'perdu'
}

export function visibleStages(closed: ClosedFilter): CustomerPipelineStage[] {
	return PIPELINE_STAGES.filter((stage) => {
		if (stage === 'WON') return showsWon(closed)
		if (stage === 'LOST') return showsLost(closed)
		return true
	})
}

function normalize(text: string): string {
	return text
		.normalize('NFD')
		.replace(/\p{M}/gu, '')
		.replace(/’/g, "'")
		.toLowerCase()
}

export function filterPipelineCustomers(
	customers: readonly Customer[],
	extrasOf: (customerId: string) => AccountExtras,
	filters: DealFilters,
): Customer[] {
	const query = normalize(filters.query.trim())
	return customers.filter((customer) => {
		if (customer.status === 'ARCHIVED') return false
		const extras = extrasOf(customer.id)
		if (filters.owner !== ALL && extras.ownerId !== filters.owner) return false
		if (filters.source !== ALL && extras.source !== filters.source) return false
		if (query === '') return true
		return [customer.name, customer.email ?? '', customer.phone ?? '']
			.map(normalize)
			.some((text) => text.includes(query))
	})
}

export function pipelineStageDescription(stage: CustomerPipelineStage): string {
	if (stage === 'NEW') return 'À qualifier'
	if (stage === 'CONTACTED') return 'Premier échange lancé'
	if (stage === 'QUALIFIED') return 'Besoin confirmé'
	if (stage === 'QUOTE_SENT') return 'Offre transmise'
	if (stage === 'WON') return 'Converti en client'
	return 'Opportunité perdue'
}

export function conversionNotice(
	customer: Pick<Customer, 'name' | 'status'>,
	stage: CustomerPipelineStage,
): string | null {
	return customer.status === 'PROSPECT' && stage === 'WON'
		? `${customer.name} est maintenant client.`
		: null
}
