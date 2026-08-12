import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { TeamListFeature } from '#/pages/hr/feature/team-list-feature'
import { renderWithRouter } from '#/test/render-with-router'

const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const EMPLOYEE_PROFILES_PATH =
	'/api/v1/organizations/{organization_id}/employee-profiles'
const INVITATIONS_PATH = '/api/v1/organizations/{organization_id}/invitations'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
}

const MEMBER = {
	id: 'member-1',
	organization_id: 'org-1',
	last_name: 'Nova',
	first_name: 'Alix',
	display_name: 'Nova Alix',
	account: null,
	joined_at: null,
	created_at: '2026-01-01T00:00:00Z',
}

interface FakeApiHandlers {
	invitations?: unknown[]
	postInvitation?: (params: unknown) => unknown
	deleteInvitation?: (params: unknown) => unknown
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
	const calls: { method: string; path: string; params: unknown }[] = []
	const invitations = handlers.invitations ?? []

	function queryKeyFor(path: string, params: unknown) {
		const p = (params ?? {}) as { path?: unknown; query?: unknown }
		return [{ _id: path, path: p.path, query: p.query }]
	}

	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = queryKeyFor(path, params)
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params })
						if (path === MEMBERS_PATH) {
							return { data: [MEMBER], pagination: null }
						}
						if (path === EMPLOYEE_PROFILES_PATH) {
							return { data: [], pagination: null }
						}
						if (path === INVITATIONS_PATH) {
							return { data: invitations, pagination: null }
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
						if (method === 'post' && path === INVITATIONS_PATH) {
							if (!handlers.postInvitation) {
								throw new Error('postInvitation not mocked')
							}
							return handlers.postInvitation(params)
						}
						if (
							method === 'delete' &&
							path === '/api/v1/invitations/{invitation_id}'
						) {
							if (!handlers.deleteInvitation) {
								throw new Error('deleteInvitation not mocked')
							}
							return handlers.deleteInvitation(params)
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

function renderFeature(handlers: FakeApiHandlers = {}) {
	const calls = installFakeTanstackApi(handlers)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>
				<OrganizationListProvider organizations={[ORGANIZATION]}>
					<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
						{children}
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>
		)
	}

	return renderWithRouter(
		<Providers>
			<TeamListFeature />
		</Providers>,
	).then((result) => ({ ...result, calls }))
}

describe('TeamListFeature — invite gesture', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('generates a link for the targeted seat and never resurrects it once closed and reopened', async () => {
		const user = userEvent.setup()
		const { calls } = await renderFeature({
			postInvitation: (params) => ({
				data: {
					id: 'invitation-1',
					token: 'clear-token-abc',
					...paramsBody(params),
				},
			}),
		})

		await user.click(await screen.findByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Inviter/ }))

		expect(screen.getByText('Inviter Nova Alix')).toBeDefined()
		await user.click(screen.getByRole('button', { name: /Générer le lien/ }))

		await waitFor(() => {
			expect(screen.getByLabelText('Lien d’invitation')).toBeDefined()
		})
		const postCall = calls.find(
			(c) => c.method === 'post' && c.path === INVITATIONS_PATH,
		)
		expect(postCall?.params).toMatchObject({
			path: { organization_id: 'org-1' },
			body: { member_id: 'member-1' },
		})

		// Close, then reopen "Inviter" for the same seat — the token must not
		// come back on its own; only a fresh "Générer le lien" click would
		// produce one, and the backend never returns the same clear value
		// twice regardless.
		await user.click(screen.getByRole('button', { name: 'Fermer' }))
		await user.click(await screen.findByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Inviter/ }))

		expect(screen.queryByLabelText('Lien d’invitation')).toBeNull()
		expect(
			screen.getByRole('button', { name: /Générer le lien/ }),
		).toBeDefined()
	})
})

describe('TeamListFeature — pending invitations', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('lists a pending invitation for the seat it targets and revokes it', async () => {
		const user = userEvent.setup()
		const { calls } = await renderFeature({
			invitations: [
				{
					id: 'invitation-1',
					organization_id: 'org-1',
					member_id: 'member-1',
					expires_at: '2026-08-20T00:00:00Z',
					created_at: '2026-08-13T00:00:00Z',
				},
			],
			deleteInvitation: () => ({ data: undefined }),
		})

		expect(await screen.findByText('Invitations en attente (1)')).toBeDefined()
		expect(screen.getAllByText('Nova Alix').length).toBeGreaterThan(0)

		await user.click(screen.getByRole('button', { name: /Révoquer/ }))

		await waitFor(() => {
			const deleteCall = calls.find(
				(c) =>
					c.method === 'delete' &&
					c.path === '/api/v1/invitations/{invitation_id}',
			)
			expect(deleteCall?.params).toMatchObject({
				path: { invitation_id: 'invitation-1' },
			})
		})
	})
})

function paramsBody(params: unknown) {
	return (params as { body?: unknown }).body ?? {}
}
