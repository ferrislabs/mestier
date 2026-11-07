import {
	type DataViewOption,
	type DataViewUrlState,
	readDataViewUrlState,
	writeDataViewUrlState,
} from '#/components/data-view'
import { STATUS_FILTERS } from '#/pages/quotes/lib/quote-filter'
import { QUOTE_DISPLAY_LABELS } from '#/pages/quotes/lib/quote-status'

export const QUOTE_FILTER_OPTIONS: DataViewOption[] = STATUS_FILTERS.map(
	(status) => ({
		value: status,
		label: status === 'all' ? 'Tous les statuts' : QUOTE_DISPLAY_LABELS[status],
	}),
)

export const DEFAULT_QUOTE_LIST_STATE: DataViewUrlState = {
	search: '',
	filter: 'all',
	sort: 'created-desc',
	page: 1,
	pageSize: 10,
}

export function getQuoteListUrlState(): DataViewUrlState {
	return readDataViewUrlState({
		defaults: DEFAULT_QUOTE_LIST_STATE,
		isValidFilter: isValidQuoteFilter,
		isValidSort: isValidQuoteSortValue,
	})
}

export function writeQuoteListUrlState(state: DataViewUrlState) {
	writeDataViewUrlState(state, DEFAULT_QUOTE_LIST_STATE)
}

export function isValidQuoteFilter(value: string): boolean {
	return QUOTE_FILTER_OPTIONS.some((option) => option.value === value)
}

export function isValidQuoteSortValue(value: string): boolean {
	return QUOTE_SORT_VALUES.includes(value)
}

const QUOTE_SORT_VALUES = [
	'created-desc',
	'created-asc',
	'total-desc',
	'total-asc',
]

export function readQuoteClientParam(): string | null {
	if (typeof window === 'undefined') return null
	const client = new URLSearchParams(window.location.search).get('client')
	return client && client.trim() !== '' ? client : null
}

export function readQuoteDealParam(): string | null {
	if (typeof window === 'undefined') return null
	const deal = new URLSearchParams(window.location.search).get('opportunite')
	return deal && deal.trim() !== '' ? deal : null
}
