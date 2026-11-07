import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useDeals, useDealsByCustomer } from '#/pages/deals/feature/use-deals'

vi.mock('#/hooks/use-active-organization', () => ({
	useActiveOrganization: () => ({
		activeOrganization: { id: 'org-1', slug: 'acme' },
	}),
}))

const DEALS_PATH = '/api/v1/organizations/{organization_id}/deals'
const SUMMARY_PATH = '/api/v1/organizations/{organization_id}/deals/summary'
const DEAL_PATH = '/api/v1/deals/{deal_id}'
const STAGE_PATH = '/api/v1/deals/{deal_id}/stage'
const CUSTOMERS_PATH = '/api/v1/organizations/{organization_id}/customers'
const QUOTES_PATH = '/api/v1/organizations/{organization_id}/quotes'

interface RecordedCall {
	method: string
	path: string
	params: unknown
}

const API_DEAL = {
	id: 'd1',
	organization_id: 'org-1',
	customer_id: 'c1',
	title: 'Fournil',
	amount_cents: 125000,
	stage: 'NEGOTIATION',
	expected_close_date: null,
	owner_member_id: null,
	source: null,
	lost_reason: null,
	notes: null,
	quote_count: 1,
	created_at: '2026-10-01T08:00:00Z',
	stage_changed_at: '2026-10-02T08:00:00Z',
	closed_at: null,
}

const ANSWERS: Record<string, unknown> = {
	[DEALS_PATH]: { data: [API_DEAL] },
	[SUMMARY_PATH]: {
		data: {
			open_count: 1,
			open_amount_cents: 125000,
			stages: [{ stage: 'NEGOTIATION', count: 1, amount_cents: 125000 }],
			won_count: 2,
			lost_count: 1,
			won_amount_cents: 900,
			win_rate: 67,
			conversions: 3,
		},
	},
	[CUSTOMERS_PATH]: {
		data: [{ id: 'c1', name: 'Boulangerie', status: 'PROSPECT' }],
	},
	[QUOTES_PATH]: {
		data: [
			{
				id: 'q1',
				reference: 'DEV-2026-0001',
				title: 'Devis fournil',
				deal_id: 'd1',
				status: 'SENT',
				net_cents: 100000,
				created_at: '2026-10-03T08:00:00Z',
			},
			{
				id: 'q2',
				reference: null,
				title: 'Sans opportunité',
				deal_id: null,
				status: 'DRAFT',
				net_cents: 1,
				created_at: '2026-10-04T08:00:00Z',
			},
		],
	},
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
						return ANSWERS[path] ?? { data: [] }
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
						return { data: API_DEAL }
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

describe('useDeals', () => {
	let calls: RecordedCall[]

	beforeEach(() => {
		calls = installFakeApi()
		localStorage.clear()
	})

	it('reads deals, summary and linked quotes from the API', async () => {
		const { wrapper } = setup()

		const { result } = renderHook(() => useDeals(), { wrapper })

		await waitFor(() => expect(result.current.deals).toHaveLength(1))
		await waitFor(() => expect(result.current.quotes).toHaveLength(1))
		await waitFor(() => expect(result.current.summary.winRate).toBe(67))
		expect(result.current.deals[0]).toMatchObject({
			id: 'd1',
			stage: 'negotiation',
			amount: 125000,
			quoteCount: 1,
		})
		expect(result.current.quotes[0]).toMatchObject({
			dealId: 'd1',
			number: 'DEV-2026-0001',
			status: 'sent',
			subtotal: 100000,
		})
		expect(result.current.summary.conversions).toBe(3)
		expect(calls.map((call) => call.path)).toEqual(
			expect.arrayContaining([DEALS_PATH, SUMMARY_PATH, QUOTES_PATH]),
		)
	})

	it('never reads the deals mock from localStorage', async () => {
		const getItem = vi.spyOn(Storage.prototype, 'getItem')
		const setItem = vi.spyOn(Storage.prototype, 'setItem')
		const { wrapper } = setup()

		const { result } = renderHook(() => useDeals(), { wrapper })
		await waitFor(() => expect(result.current.deals).toHaveLength(1))
		act(() => result.current.move('d1', 'visit', '', () => undefined))
		await waitFor(() =>
			expect(calls.some((call) => call.path === STAGE_PATH)).toBe(true),
		)

		const touched = [...getItem.mock.calls, ...setItem.mock.calls].map(
			([key]) => key,
		)
		expect(touched.filter((key) => key.includes('mock:deals'))).toEqual([])
		getItem.mockRestore()
		setItem.mockRestore()
	})

	it('posts a draft in cents to the organization and refreshes the list', async () => {
		const { wrapper, queryClient } = setup()
		const { result } = renderHook(() => useDeals(), { wrapper })
		await waitFor(() => expect(result.current.deals).toHaveLength(1))

		act(() =>
			result.current.create({
				customerId: 'c1',
				title: ' Vitrine ',
				amount: 4200,
				expectedCloseDate: '2026-12-01',
				ownerId: 'm1',
				source: 'salon',
				notes: '',
			}),
		)

		await waitFor(() =>
			expect(calls.find((call) => call.method === 'post')).toEqual({
				method: 'post',
				path: DEALS_PATH,
				params: {
					path: { organization_id: 'org-1' },
					body: {
						customer_id: 'c1',
						title: 'Vitrine',
						amount_cents: 4200,
						expected_close_date: '2026-12-01',
						owner_member_id: 'm1',
						source: 'salon',
						notes: null,
					},
				},
			}),
		)
		await waitFor(() => {
			const state = queryClient.getQueryCache().findAll()
			expect(state.some((query) => query.isStale())).toBe(true)
		})
	})

	it('moves a deal through the stage endpoint, with a reason only for lost', async () => {
		const { wrapper } = setup()
		const { result } = renderHook(() => useDeals(), { wrapper })
		await waitFor(() => expect(result.current.deals).toHaveLength(1))
		const notices: (string | null)[] = []

		act(() =>
			result.current.move('d1', 'lost', 'Prix trop élevé', (notice) =>
				notices.push(notice),
			),
		)
		await waitFor(() => expect(notices).toEqual([null]))
		act(() =>
			result.current.move('d1', 'visit', 'ignoré', (notice) =>
				notices.push(notice),
			),
		)
		await waitFor(() => expect(notices).toHaveLength(2))

		const moves = calls.filter((call) => call.path === STAGE_PATH)
		expect(moves.map((call) => call.params)).toEqual([
			{
				path: { deal_id: 'd1' },
				body: { stage: 'LOST', lost_reason: 'Prix trop élevé' },
			},
			{ path: { deal_id: 'd1' }, body: { stage: 'VISIT', lost_reason: null } },
		])
	})

	it('announces the conversion of a prospect won through the move', async () => {
		const { wrapper } = setup()
		const { result } = renderHook(() => useDeals(), { wrapper })
		await waitFor(() => expect(result.current.customers).toHaveLength(1))
		const notices: (string | null)[] = []

		act(() =>
			result.current.move('d1', 'won', '', (notice) => notices.push(notice)),
		)

		await waitFor(() =>
			expect(notices).toEqual(['Boulangerie est maintenant client.']),
		)
	})

	it('deletes and updates through their endpoints', async () => {
		const { wrapper } = setup()
		const { result } = renderHook(() => useDeals(), { wrapper })
		await waitFor(() => expect(result.current.deals).toHaveLength(1))

		act(() => result.current.remove('d1'))
		act(() =>
			result.current.update('d1', {
				customerId: 'c1',
				title: 'Fournil 2',
				amount: 1,
				expectedCloseDate: null,
				ownerId: null,
				source: null,
				notes: 'x',
			}),
		)

		await waitFor(() => {
			expect(
				calls.filter((call) => call.path === DEAL_PATH).map((c) => c.method),
			).toEqual(['delete', 'patch'])
		})
	})
})

describe('useDealsByCustomer', () => {
	it('asks the API for the deals of that customer only', async () => {
		const calls = installFakeApi()
		const { wrapper } = setup()

		const { result } = renderHook(() => useDealsByCustomer('c1'), { wrapper })

		await waitFor(() => expect(result.current.deals).toHaveLength(1))
		expect(calls[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			query: { page: 1, per_page: 100, customer_id: 'c1' },
		})
	})
})

describe('the pipeline sources', () => {
	const sources = import.meta.glob('/src/pages/deals/**/*.{ts,tsx}', {
		query: '?raw',
		import: 'default',
		eager: true,
	}) as Record<string, string>

	it('read no localStorage and import no deals mock', () => {
		const offenders = Object.entries(sources)
			.filter(
				([path]) => !path.endsWith('.test.tsx') && !path.endsWith('.test.ts'),
			)
			.filter(
				([, source]) =>
					source.includes('localStorage') ||
					source.includes('mocks/deals-store') ||
					source.includes('mocks/deal-quotes'),
			)
			.map(([path]) => path)

		expect(Object.keys(sources).length).toBeGreaterThan(10)
		expect(offenders).toEqual([])
	})
})
