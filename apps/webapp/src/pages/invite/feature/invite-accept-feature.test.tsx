import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { InviteAcceptFeature } from '#/pages/invite/feature/invite-accept-feature'
import { renderWithRouter } from '#/test/render-with-router'

const ACCEPT_PATH = '/api/v1/invitations/{token}/accept'
const MY_ORGS_PATH = '/api/v1/users/@me/organizations'

interface FakeApiHandlers {
	accept?: (params: unknown) => unknown | Promise<unknown>
	organizations?: unknown[]
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
	const calls: { method: string; path: string; params: unknown }[] = []
	const organizations = handlers.organizations ?? []

	const fakeApi = {
		get(path: string) {
			const queryKey = [{ _id: path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params: undefined })
						if (path === MY_ORGS_PATH) {
							return { data: organizations, pagination: null }
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
						if (method === 'post' && path === ACCEPT_PATH) {
							if (!handlers.accept) throw new Error('accept not mocked')
							return handlers.accept(params)
						}
						throw new Error(`unmocked mutation ${method} ${path}`)
					},
				},
			}
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return calls
}

async function renderFeature(handlers: FakeApiHandlers = {}) {
	const calls = installFakeTanstackApi(handlers)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
		)
	}

	const result = await renderWithRouter(
		<Providers>
			<InviteAcceptFeature token="abc123" />
		</Providers>,
	)

	return { ...result, calls }
}

describe('InviteAcceptFeature — success', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('accepts once, then navigates into the joined organization', async () => {
		const { calls, router } = await renderFeature({
			accept: () => ({
				data: { id: 'member-1', organization_id: 'org-1' },
			}),
			organizations: [
				{ id: 'org-1', name: 'Atelier Bois', slug: 'atelier-bois' },
			],
		})

		await waitFor(() => {
			expect(router.state.location.pathname).toBe('/o/atelier-bois')
		})

		const acceptCalls = calls.filter(
			(c) => c.method === 'post' && c.path === ACCEPT_PATH,
		)
		expect(acceptCalls).toHaveLength(1)
		expect(acceptCalls[0]?.params).toMatchObject({ path: { token: 'abc123' } })
	})
})

describe('InviteAcceptFeature — error states', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('shows a generic message for an invalid, expired, or already-consumed token (404)', async () => {
		await renderFeature({
			accept: () => {
				throw Object.assign(new Error('not found'), { status: 404 })
			},
		})

		expect(await screen.findByText(/n’est plus valide/)).toBeDefined()
	})

	it('shows a distinct message when the caller is already a member (409)', async () => {
		await renderFeature({
			accept: () => {
				throw Object.assign(new Error('conflict'), { status: 409 })
			},
		})

		expect(
			await screen.findByText('Vous êtes déjà membre de cette organisation.'),
		).toBeDefined()
	})
})
