import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it } from 'vitest'
import {
	useCreateMemberContract,
	useDeleteMemberContract,
	useMemberContracts,
	useUpdateMemberContract,
} from '#/hooks/use-member-contracts'
import {
	useAddMemberDocument,
	useDeleteMemberDocument,
	useMemberDocuments,
} from '#/hooks/use-member-documents'

const CONTRACTS_PATH =
	'/api/v1/organizations/{organization_id}/member-contracts'
const DOCUMENTS_PATH =
	'/api/v1/organizations/{organization_id}/member-documents'

interface Call {
	method: string
	path: string
	params: unknown
}

let calls: Call[]
let reads: Record<string, number>

function installFakeApi() {
	calls = []
	reads = {}
	const fakeApi = {
		get(path: string, params: unknown) {
			const p = (params ?? {}) as { path?: unknown }
			const queryKey = [{ _id: path, path: p.path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						reads[path] = (reads[path] ?? 0) + 1
						return { data: [], pagination: null }
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
						return { data: {} }
					},
				},
			}
		},
	}
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi
}

function wrapper() {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	return ({ children }: { children: ReactNode }) => (
		<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
	)
}

const PATH = { organization_id: 'org-1', member_id: 'm-1' }

describe('member contract and document hooks', () => {
	beforeEach(installFakeApi)

	it('spec_rh_2_6 reads the contract and document lists, and nothing without an organization', async () => {
		const { result } = renderHook(
			() => ({
				contracts: useMemberContracts('org-1'),
				documents: useMemberDocuments('org-1'),
			}),
			{ wrapper: wrapper() },
		)
		await waitFor(() => expect(result.current.contracts.isSuccess).toBe(true))
		await waitFor(() => expect(result.current.documents.isSuccess).toBe(true))
		expect(reads[CONTRACTS_PATH]).toBe(1)
		expect(reads[DOCUMENTS_PATH]).toBe(1)

		reads = {}
		renderHook(() => useMemberContracts(''), { wrapper: wrapper() })
		expect(reads[CONTRACTS_PATH]).toBeUndefined()
	})

	it.each([
		['create', 'post', '/contracts', useCreateMemberContract],
		['update', 'put', '/contracts/{contract_id}', useUpdateMemberContract],
		['delete', 'delete', '/contracts/{contract_id}', useDeleteMemberContract],
	] as const)('spec_rh_2_6 contract %s calls the API and refreshes the list', async (_name, method, tail, useHook) => {
		const { result } = renderHook(
			() => ({ list: useMemberContracts('org-1'), write: useHook() }),
			{ wrapper: wrapper() },
		)
		await waitFor(() => expect(result.current.list.isSuccess).toBe(true))

		await act(async () => {
			await result.current.write.mutateAsync({ path: PATH, body: {} } as never)
		})

		expect(calls).toHaveLength(1)
		expect(calls[0]?.method).toBe(method)
		expect(calls[0]?.path.endsWith(tail)).toBe(true)
		await waitFor(() => expect(reads[CONTRACTS_PATH]).toBe(2))
	})

	it.each([
		['add', 'post', '/documents', useAddMemberDocument],
		['delete', 'delete', '/documents/{document_id}', useDeleteMemberDocument],
	] as const)('spec_rh_2_6 document %s calls the API and refreshes the list', async (_name, method, tail, useHook) => {
		const { result } = renderHook(
			() => ({ list: useMemberDocuments('org-1'), write: useHook() }),
			{ wrapper: wrapper() },
		)
		await waitFor(() => expect(result.current.list.isSuccess).toBe(true))

		await act(async () => {
			await result.current.write.mutateAsync({ path: PATH, body: {} } as never)
		})

		expect(calls[0]?.method).toBe(method)
		expect(calls[0]?.path.endsWith(tail)).toBe(true)
		await waitFor(() => expect(reads[DOCUMENTS_PATH]).toBe(2))
	})
})
