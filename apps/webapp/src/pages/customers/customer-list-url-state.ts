import {
	type DataViewOption,
	type DataViewUrlState,
	readDataViewUrlState,
	writeDataViewUrlState,
} from '#/components/data-view'

export const CUSTOMER_FILTER_OPTIONS: DataViewOption[] = [
	{ value: 'all', label: 'Tous les clients' },
	{ value: 'prospects', label: 'Prospects' },
	{ value: 'clients', label: 'Clients' },
	{ value: 'with-email', label: 'Avec email' },
	{ value: 'with-phone', label: 'Avec téléphone' },
	{ value: 'incomplete-contact', label: 'Coordonnées incomplètes' },
]

export const DEFAULT_CUSTOMER_LIST_STATE: DataViewUrlState = {
	search: '',
	filter: 'all',
	sort: 'created-desc',
	page: 1,
	pageSize: 10,
}

export function getCustomerListUrlState(): DataViewUrlState {
	return readDataViewUrlState({
		defaults: DEFAULT_CUSTOMER_LIST_STATE,
		isValidFilter: isValidCustomerFilter,
		isValidSort: isValidCustomerSortValue,
	})
}

export function writeCustomerListUrlState(state: DataViewUrlState) {
	writeDataViewUrlState(state, DEFAULT_CUSTOMER_LIST_STATE)
}

export function isValidCustomerFilter(value: string): boolean {
	return CUSTOMER_FILTER_OPTIONS.some((option) => option.value === value)
}

export function isValidCustomerSortValue(value: string): boolean {
	return CUSTOMER_SORT_VALUES.includes(value)
}

const CUSTOMER_SORT_VALUES = [
	'created-desc',
	'created-asc',
	'name-asc',
	'name-desc',
]
