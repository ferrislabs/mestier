import {
	type DataViewOption,
	type DataViewUrlState,
	readDataViewUrlState,
	writeDataViewUrlState,
} from '#/components/data-view'

export const INVOICE_FILTER_OPTIONS: DataViewOption[] = [
	{ value: 'all', label: 'Toutes' },
	{ value: 'unpaid', label: 'À payer' },
	{ value: 'overdue', label: 'En retard' },
	{ value: 'paid', label: 'Payées' },
	{ value: 'draft', label: 'Brouillons' },
	{ value: 'cancelled', label: 'Annulées' },
	{ value: 'refund_due', label: 'À rembourser' },
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

export function readInvoiceClientParam(): string | null {
	if (typeof window === 'undefined') return null
	return new URLSearchParams(window.location.search).get('client') || null
}

export function clearInvoiceClientParam() {
	if (typeof window === 'undefined') return
	const params = new URLSearchParams(window.location.search)
	params.delete('client')
	const query = params.toString()
	window.history.replaceState(
		null,
		'',
		query ? `${window.location.pathname}?${query}` : window.location.pathname,
	)
}
