import {
	type DataViewOption,
	type DataViewUrlState,
	readDataViewUrlState,
	writeDataViewUrlState,
} from '#/components/data-view'

export const QUOTE_FILTER_OPTIONS: DataViewOption[] = [
	{ value: 'all', label: 'Tous les statuts' },
	{ value: 'DRAFT', label: 'Brouillons' },
	{ value: 'SENT', label: 'Envoyés' },
	{ value: 'ACCEPTED', label: 'Acceptés' },
	{ value: 'DECLINED', label: 'Refusés' },
	{ value: 'CANCELLED', label: 'Annulés' },
]

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
