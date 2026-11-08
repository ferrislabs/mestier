import type { QueryFunctionContext } from '@tanstack/react-query'

export const LIST_PAGE_SIZE = 100
export const MAX_LIST_PAGES = 50

interface PageEnvelope<T> {
	data: T[]
	pagination?: { next_page?: number | null } | null
}

export interface AllPages<T> {
	data: T[]
	pagination: null
}

export async function fetchAllPages<T>(
	readPage: (page: number) => Promise<PageEnvelope<T>>,
): Promise<AllPages<T>> {
	const rows: T[] = []
	let page = 1
	while (page <= MAX_LIST_PAGES) {
		const envelope = await readPage(page)
		rows.push(...envelope.data)
		const next = envelope.pagination?.next_page
		if (next == null || next <= page || envelope.data.length === 0) break
		page = next
	}
	return { data: rows, pagination: null }
}

interface ListOptions {
	queryKey: readonly unknown[]
	queryFn?: unknown
}

export function readListPage<T>(
	options: ListOptions,
	context: QueryFunctionContext,
): Promise<PageEnvelope<T>> {
	const read = options.queryFn
	if (typeof read !== 'function') {
		throw new Error('Lecture de liste indisponible')
	}
	return read({ ...context, queryKey: options.queryKey })
}
