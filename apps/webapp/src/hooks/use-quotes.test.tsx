import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it } from 'vitest'
import {
	useQuotes,
	useReviseQuote,
	useUpdateQuoteStatus,
} from '#/hooks/use-quotes'

const QUOTES_PATH = '/api/v1/organizations/{organization_id}/quotes'
const STATUS_PATH = '/api/v1/quotes/{quote_id}/status'
const REVISION_PATH = '/api/v1/quotes/{quote_id}/revision'

interface RecordedCall {
	method: string
	path: string
	params: unknown
}

const API_QUOTE = {
	id: 'q1',
	organization_id: 'org-1',
	status: 'ACCEPTED',
}

function installFakeApi() {
	const calls: RecordedCall[] = []
	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = [
				{ _id: path, path: (params as { path?: unknown } | undefined)?.path },
				(params as { query?: unknown } | undefined)?.query ?? null,
			]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params })
						return { data: [API_QUOTE] }
					},
				},
			}
		},
		mutation(method: string, path: string) {
			const mutationKey = [{ method, path }]
			return {
				mutationKey,
				mutationOptions: {
					mutationKey,
					mutationFn: async (params: unknown) => {
						calls.push({ method, path, params })
						return { data: { ...API_QUOTE, id: 'q2' } }
					},
				},
			}
		},
	}
	Object.assign(window, { tanstackApi: fakeApi })
	return calls
}

function setup() {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	function wrapper({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
		)
	}
	return { queryClient, wrapper }
}

describe('the quote hooks', () => {
	let calls: RecordedCall[]

	beforeEach(() => {
		calls = installFakeApi()
	})

	it('read the quotes from the API', async () => {
		const { wrapper } = setup()

		const { result } = renderHook(() => useQuotes('org-1'), { wrapper })

		await waitFor(() => expect(result.current.data?.data).toHaveLength(1))
		expect(calls[0]).toMatchObject({ method: 'get', path: QUOTES_PATH })
	})

	it('answer a quote through the status endpoint, signatory and reason included', async () => {
		const { wrapper } = setup()
		const { result } = renderHook(() => useUpdateQuoteStatus(), { wrapper })

		await act(async () => {
			await result.current.mutateAsync({
				path: { quote_id: 'q1' },
				body: { status: 'ACCEPTED', responded_by: 'Mme Durand' },
			})
		})

		expect(calls).toContainEqual({
			method: 'patch',
			path: STATUS_PATH,
			params: {
				path: { quote_id: 'q1' },
				body: { status: 'ACCEPTED', responded_by: 'Mme Durand' },
			},
		})
	})

	it('revise a quote through the revision endpoint', async () => {
		const { wrapper } = setup()
		const { result } = renderHook(() => useReviseQuote(), { wrapper })

		let revisionId = ''
		await act(async () => {
			const revision = await result.current.mutateAsync({
				path: { quote_id: 'q1' },
			})
			revisionId = revision.data.id
		})

		expect(revisionId).toBe('q2')
		expect(calls).toContainEqual({
			method: 'post',
			path: REVISION_PATH,
			params: { path: { quote_id: 'q1' } },
		})
	})

	it('refresh the quote list and the deals after an answer', async () => {
		const { queryClient, wrapper } = setup()
		const { result: list } = renderHook(() => useQuotes('org-1'), { wrapper })
		await waitFor(() => expect(list.current.data).toBeDefined())
		const { result } = renderHook(() => useUpdateQuoteStatus(), { wrapper })
		const before = calls.filter((call) => call.method === 'get').length

		await act(async () => {
			await result.current.mutateAsync({
				path: { quote_id: 'q1' },
				body: { status: 'DECLINED', decline_reason: 'trop cher' },
			})
		})

		await waitFor(() =>
			expect(
				calls.filter((call) => call.method === 'get').length,
			).toBeGreaterThan(before),
		)
		expect(queryClient.isFetching()).toBe(0)
	})
})
