import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { TanstackQueryApiClient } from '#/api/api.tanstack'
import { fetchAllPages, MAX_LIST_PAGES } from '#/hooks/fetch-all-pages'
import { useAbsences } from '#/hooks/use-absences'
import { useExtraPeriods } from '#/hooks/use-extra-periods'

const TOTAL = 250
const PER_PAGE = 100

function pageOf(rows: readonly { id: string }[], page: number) {
	const from = (page - 1) * PER_PAGE
	const last = Math.ceil(rows.length / PER_PAGE)
	return {
		data: rows.slice(from, from + PER_PAGE),
		pagination: {
			current_page: page,
			next_page: page < last ? page + 1 : null,
		},
	}
}

function rowsOf(prefix: string, count: number) {
	return Array.from({ length: count }, (_, index) => ({
		id: `${prefix}-${index}`,
	}))
}

function wrapper({ children }: { children: ReactNode }) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	return (
		<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
	)
}

let get: ReturnType<typeof vi.fn>

beforeEach(() => {
	get = vi.fn()
	window.tanstackApi = new TanstackQueryApiClient({
		get,
	} as never)
})

describe('fetchAllPages', () => {
	it('spec_rh_7_6 follows next_page until the last page', async () => {
		const rows = rowsOf('row', TOTAL)
		const read = vi.fn(async (page: number) => pageOf(rows, page))
		const all = await fetchAllPages(read)
		expect(all.data).toHaveLength(TOTAL)
		expect(read).toHaveBeenCalledTimes(3)
	})

	it('spec_rh_7_6 stops at the safety cap on a list that never ends', async () => {
		const read = vi.fn(async (page: number) => ({
			data: [{ id: `p-${page}` }],
			pagination: { next_page: page + 1 },
		}))
		const all = await fetchAllPages(read)
		expect(read).toHaveBeenCalledTimes(MAX_LIST_PAGES)
		expect(all.data).toHaveLength(MAX_LIST_PAGES)
	})

	it('spec_rh_7_6 reads a single page when the API gives no pagination', async () => {
		const read = vi.fn(async () => ({ data: [{ id: 'a' }], pagination: null }))
		const all = await fetchAllPages(read)
		expect(all.data).toEqual([{ id: 'a' }])
		expect(read).toHaveBeenCalledTimes(1)
	})
})

describe('the absence and extra lists', () => {
	it('spec_rh_7_6 useAbsences brings back all 250 rows', async () => {
		const rows = rowsOf('absence', TOTAL)
		get.mockImplementation(
			async (_path: string, params: { query: { page: number } }) =>
				pageOf(rows, params.query.page),
		)
		const { result } = renderHook(() => useAbsences('org-1'), { wrapper })
		await waitFor(() => expect(result.current.isSuccess).toBe(true))
		expect(result.current.data?.data).toHaveLength(TOTAL)
		expect(get).toHaveBeenCalledTimes(3)
	})

	it('spec_rh_7_6 useExtraPeriods brings back all 250 rows', async () => {
		const rows = rowsOf('extra', TOTAL)
		get.mockImplementation(
			async (_path: string, params: { query: { page: number } }) =>
				pageOf(rows, params.query.page),
		)
		const { result } = renderHook(() => useExtraPeriods('org-1'), { wrapper })
		await waitFor(() => expect(result.current.isSuccess).toBe(true))
		expect(result.current.data?.data).toHaveLength(TOTAL)
	})
})
