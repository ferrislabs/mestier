import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import {
	useBillingSettings,
	useNumbering,
	useReplaceBillingSettings,
	useSetNextNumber,
} from '#/hooks/use-billing-settings'

const BILLING_PATH = '/api/v1/organizations/{organization_id}/billing-settings'
const NUMBERING_PATH = '/api/v1/organizations/{organization_id}/numbering'
const SERIES_PATH = '/api/v1/organizations/{organization_id}/numbering/{kind}'

interface RecordedCall {
	method: string
	path: string
	params: unknown
}

function installFakeApi(respond: (params: unknown) => unknown) {
	const calls: RecordedCall[] = []

	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = [
				{ _id: path, path: (params as { path?: unknown } | undefined)?.path },
			]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params })
						return respond(params)
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
						return respond(params)
					},
				},
			}
		},
	}

	Object.assign(window, { tanstackApi: fakeApi })

	return calls
}

function key(path: string, organizationId: string) {
	return [{ _id: path, path: { organization_id: organizationId } }]
}

function setup() {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	queryClient.setQueryData(key(BILLING_PATH, 'org-1'), { data: {} })
	queryClient.setQueryData(key(NUMBERING_PATH, 'org-1'), { data: [] })
	queryClient.setQueryData(key(NUMBERING_PATH, 'org-2'), { data: [] })

	function wrapper({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
		)
	}

	return { queryClient, wrapper }
}

const BODY = {
	payment_terms_days: 30,
	quote_validity_days: 30,
	default_vat_rate_bp: 2000,
	late_penalty: 'trois fois le taux légal',
	recovery_fee_cents: 4000,
	invoice_footer: '',
	number_padding: 4,
	iban: null,
	bic: null,
}

describe('useBillingSettings', () => {
	it('reads the billing settings of the organization from the API', async () => {
		const calls = installFakeApi(() => ({ data: BODY }))
		const { wrapper } = setup()

		const { result } = renderHook(() => useBillingSettings('org-9'), {
			wrapper,
		})

		await waitFor(() => expect(result.current.data).toEqual({ data: BODY }))
		expect(calls).toEqual([
			{
				method: 'get',
				path: BILLING_PATH,
				params: { path: { organization_id: 'org-9' } },
			},
		])
	})
})

describe('useReplaceBillingSettings', () => {
	it('sends PUT with the whole body and refreshes settings and numbering of that organization', async () => {
		const calls = installFakeApi(() => ({ data: BODY }))
		const { wrapper, queryClient } = setup()
		const { result } = renderHook(() => useReplaceBillingSettings('org-1'), {
			wrapper,
		})

		await act(async () => {
			await result.current.mutateAsync({
				path: { organization_id: 'org-1' },
				body: BODY,
			})
		})

		expect(calls).toEqual([
			{
				method: 'put',
				path: BILLING_PATH,
				params: { path: { organization_id: 'org-1' }, body: BODY },
			},
		])
		await waitFor(() => {
			expect(
				queryClient.getQueryState(key(BILLING_PATH, 'org-1'))?.isInvalidated,
			).toBe(true)
			expect(
				queryClient.getQueryState(key(NUMBERING_PATH, 'org-1'))?.isInvalidated,
			).toBe(true)
		})
		expect(
			queryClient.getQueryState(key(NUMBERING_PATH, 'org-2'))?.isInvalidated,
		).toBe(false)
	})
})

describe('useNumbering', () => {
	it('reads the numbering of the organization from the API', async () => {
		const calls = installFakeApi(() => ({ data: [] }))
		const { wrapper } = setup()

		const { result } = renderHook(() => useNumbering('org-9'), { wrapper })

		await waitFor(() => expect(result.current.data).toEqual({ data: [] }))
		expect(calls[0]).toEqual({
			method: 'get',
			path: NUMBERING_PATH,
			params: { path: { organization_id: 'org-9' } },
		})
	})
})

describe('useSetNextNumber', () => {
	it('sends PUT on the series path then refreshes the numbering', async () => {
		const calls = installFakeApi(() => ({ data: {} }))
		const { wrapper, queryClient } = setup()
		const { result } = renderHook(() => useSetNextNumber('org-1'), { wrapper })

		await act(async () => {
			await result.current.mutateAsync({
				path: { organization_id: 'org-1', kind: 'CREDIT_NOTE' },
				body: { next_number: 120 },
			})
		})

		expect(calls).toEqual([
			{
				method: 'put',
				path: SERIES_PATH,
				params: {
					path: { organization_id: 'org-1', kind: 'CREDIT_NOTE' },
					body: { next_number: 120 },
				},
			},
		])
		await waitFor(() =>
			expect(
				queryClient.getQueryState(key(NUMBERING_PATH, 'org-1'))?.isInvalidated,
			).toBe(true),
		)
	})

	it('surfaces a refusal and still refreshes the numbering', async () => {
		installFakeApi(() => {
			throw new Error('E_NUMBER_NOT_FORWARD')
		})
		const { wrapper, queryClient } = setup()
		const { result } = renderHook(() => useSetNextNumber('org-1'), { wrapper })

		await act(async () => {
			await result.current
				.mutateAsync({
					path: { organization_id: 'org-1', kind: 'QUOTE' },
					body: { next_number: 3 },
				})
				.catch(() => undefined)
		})

		await waitFor(() => {
			expect(result.current.error?.message).toBe('E_NUMBER_NOT_FORWARD')
			expect(
				queryClient.getQueryState(key(NUMBERING_PATH, 'org-1'))?.isInvalidated,
			).toBe(true)
		})
	})
})
