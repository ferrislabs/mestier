import {
	type DataViewOption,
	type DataViewUrlState,
	readDataViewUrlState,
	writeDataViewUrlState,
} from '#/components/data-view'

export const INVOICE_FILTER_OPTIONS: DataViewOption[] = [
	{ value: 'all', label: 'Tous les statuts' },
	{ value: 'DRAFT', label: 'Brouillons' },
	{ value: 'ISSUED', label: 'Émises' },
	{ value: 'PARTIALLY_PAID', label: 'Partiellement payées' },
	{ value: 'PAID', label: 'Payées' },
	{ value: 'CANCELLED', label: 'Annulées' },
	{ value: 'OVERDUE', label: 'En retard' },
]

export const DEFAULT_INVOICE_LIST_STATE: DataViewUrlState = {
	search: '',
	filter: 'all',
	sort: 'created-desc',
	page: 1,
	pageSize: 10,
}

export function getInvoiceListUrlState(): DataViewUrlState {
	return readDataViewUrlState({
		defaults: DEFAULT_INVOICE_LIST_STATE,
		isValidFilter: isValidInvoiceFilter,
		isValidSort: isValidInvoiceSortValue,
	})
}

export function writeInvoiceListUrlState(state: DataViewUrlState) {
	writeDataViewUrlState(state, DEFAULT_INVOICE_LIST_STATE)
}

export function isValidInvoiceFilter(value: string): boolean {
	return INVOICE_FILTER_OPTIONS.some((option) => option.value === value)
}

export function isValidInvoiceSortValue(value: string): boolean {
	return INVOICE_SORT_VALUES.includes(value)
}

const INVOICE_SORT_VALUES = [
	'created-desc',
	'created-asc',
	'total-desc',
	'total-asc',
	'due-asc',
]
