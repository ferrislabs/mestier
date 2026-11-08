import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { act, renderHook, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it } from 'vitest'
import {
	useArchiveMember,
	useMemberProfiles,
	usePutMemberProfile,
	useRestoreMember,
} from '#/hooks/use-member-profiles'

const PROFILES_PATH = '/api/v1/organizations/{organization_id}/member-profiles'

interface Call {
	method: string
	path: string
	params: unknown
}

let calls: Call[]
let listReads: number

function installFakeApi() {
	calls = []
	listReads = 0
	const fakeApi = {
		get(path: string, params: unknown) {
			const p = (params ?? {}) as { path?: unknown }
			const queryKey = [{ _id: path, path: p.path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === PROFILES_PATH) {
							listReads += 1
							return { data: [], pagination: null }
						}
						throw new Error(`unmocked GET ${path}`)
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

describe('member profile hooks', () => {
	beforeEach(installFakeApi)

	it('reads the profile list of the organization', async () => {
		const { result } = renderHook(() => useMemberProfiles('org-1'), {
			wrapper: wrapper(),
		})

		await waitFor(() => expect(result.current.isSuccess).toBe(true))
		expect(listReads).toBe(1)
	})

	it('does not read without an organization', () => {
		renderHook(() => useMemberProfiles(''), { wrapper: wrapper() })

		expect(listReads).toBe(0)
	})

	it.each([
		['put', 'put', '/profile', usePutMemberProfile],
		['archive', 'post', '/archive', useArchiveMember],
		['restore', 'post', '/restore', useRestoreMember],
	] as const)('%s calls the API and refreshes the list', async (_name, method, tail, useHook) => {
		const { result } = renderHook(
			() => ({ list: useMemberProfiles('org-1'), write: useHook() }),
			{ wrapper: wrapper() },
		)
		await waitFor(() => expect(result.current.list.isSuccess).toBe(true))

		await act(async () => {
			await result.current.write.mutateAsync({
				path: PATH,
				body: {},
			} as never)
		})

		expect(calls).toHaveLength(1)
		expect(calls[0]?.method).toBe(method)
		expect(calls[0]?.path.endsWith(tail)).toBe(true)
		await waitFor(() => expect(listReads).toBe(2))
	})
})
