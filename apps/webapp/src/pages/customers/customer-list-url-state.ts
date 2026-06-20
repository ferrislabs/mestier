export interface CustomerListUrlState {
	search: string
	filter: string
	sort: string
	page: number
	pageSize: number
}

export const DEFAULT_CUSTOMER_LIST_STATE: CustomerListUrlState = {
	search: '',
	filter: 'all',
	sort: 'created-desc',
	page: 1,
	pageSize: 10,
}

export function getCustomerListUrlState(): CustomerListUrlState {
	if (typeof window === 'undefined') return DEFAULT_CUSTOMER_LIST_STATE

	const params = new URLSearchParams(window.location.search)

	return {
		search: params.get('q') ?? DEFAULT_CUSTOMER_LIST_STATE.search,
		filter: params.get('filter') ?? DEFAULT_CUSTOMER_LIST_STATE.filter,
		sort: params.get('sort') ?? DEFAULT_CUSTOMER_LIST_STATE.sort,
		page: parsePositiveInteger(
			params.get('page'),
			DEFAULT_CUSTOMER_LIST_STATE.page,
		),
		pageSize: parsePageSize(
			params.get('perPage'),
			DEFAULT_CUSTOMER_LIST_STATE.pageSize,
		),
	}
}

export function writeCustomerListUrlState(state: CustomerListUrlState) {
	if (typeof window === 'undefined') return

	const params = new URLSearchParams(window.location.search)
	setOptionalParam(params, 'q', state.search.trim())
	setOptionalParam(
		params,
		'filter',
		state.filter === DEFAULT_CUSTOMER_LIST_STATE.filter ? '' : state.filter,
	)
	setOptionalParam(
		params,
		'sort',
		state.sort === DEFAULT_CUSTOMER_LIST_STATE.sort ? '' : state.sort,
	)
	setOptionalParam(
		params,
		'page',
		state.page === DEFAULT_CUSTOMER_LIST_STATE.page ? '' : String(state.page),
	)
	setOptionalParam(
		params,
		'perPage',
		state.pageSize === DEFAULT_CUSTOMER_LIST_STATE.pageSize
			? ''
			: String(state.pageSize),
	)

	const query = params.toString()
	const nextUrl = query
		? `${window.location.pathname}?${query}`
		: window.location.pathname
	const currentUrl = `${window.location.pathname}${window.location.search}`
	if (nextUrl !== currentUrl) window.history.replaceState(null, '', nextUrl)
}

function setOptionalParam(params: URLSearchParams, key: string, value: string) {
	if (value) {
		params.set(key, value)
	} else {
		params.delete(key)
	}
}

function parsePositiveInteger(value: string | null, fallback: number): number {
	const parsed = Number(value)
	if (!Number.isInteger(parsed) || parsed < 1) return fallback
	return parsed
}

function parsePageSize(value: string | null, fallback: number): number {
	const parsed = parsePositiveInteger(value, fallback)
	return [10, 25, 50].includes(parsed) ? parsed : fallback
}
