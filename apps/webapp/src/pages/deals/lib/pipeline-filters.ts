import type { Customer } from '#/hooks/use-customers'
import { PEOPLE } from '#/pages/customers/feature/mocks/people'
import { LEAD_SOURCES } from '#/pages/customers/types'
import type {
	Deal,
	DealQuote,
	DealStage,
	IsoDate,
	LeadSource,
} from '#/pages/deals/lib/deal-types'
import { isOpenStage, STAGES } from '#/pages/deals/lib/pipeline'

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

export function visibleStages(closed: ClosedFilter): DealStage[] {
	return STAGES.filter((stage) => {
		if (stage === 'won') return showsWon(closed)
		if (stage === 'lost') return showsLost(closed)
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

export function filterDeals(
	deals: readonly Deal[],
	customers: readonly Pick<Customer, 'id' | 'name'>[],
	filters: DealFilters,
): Deal[] {
	const query = normalize(filters.query.trim())
	const names = new Map(
		customers.map((customer) => [customer.id, normalize(customer.name)]),
	)
	return deals.filter((deal) => {
		if (filters.owner !== ALL && deal.ownerId !== filters.owner) return false
		if (filters.source !== ALL && deal.source !== filters.source) return false
		if (query === '') return true
		return (
			normalize(deal.title).includes(query) ||
			(names.get(deal.customerId) ?? '').includes(query)
		)
	})
}

export function isOverdue(
	deal: Pick<Deal, 'stage' | 'expectedCloseDate'>,
	today: IsoDate,
): boolean {
	return (
		isOpenStage(deal.stage) &&
		deal.expectedCloseDate !== null &&
		deal.expectedCloseDate < today
	)
}

export function dealQuotes(
	quotes: readonly DealQuote[],
	dealId: string,
): DealQuote[] {
	return quotes.filter((quote) => quote.dealId === dealId)
}

export function latestQuote(
	quotes: readonly DealQuote[],
): DealQuote | undefined {
	return quotes.reduce<DealQuote | undefined>(
		(latest, quote) =>
			latest === undefined || quote.createdAt >= latest.createdAt
				? quote
				: latest,
		undefined,
	)
}

export function reopenStage(quotes: readonly DealQuote[]): DealStage {
	return quotes.some((quote) => quote.status !== 'draft') ? 'quote_sent' : 'new'
}

export function accountOptions(
	customers: readonly Customer[],
	currentId: string | null,
): Customer[] {
	return customers
		.filter(
			(customer) => customer.status !== 'ARCHIVED' || customer.id === currentId,
		)
		.sort((a, b) => a.name.localeCompare(b.name, 'fr'))
}

export function conversionNotice(
	customer: Pick<Customer, 'name' | 'status'> | undefined,
	stage: DealStage,
): string | null {
	return customer?.status === 'PROSPECT' && stage === 'won'
		? `${customer.name} est maintenant client.`
		: null
}
