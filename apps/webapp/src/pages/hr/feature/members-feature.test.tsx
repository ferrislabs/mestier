import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import type { Role } from '#/hooks/use-roles'
import { MembersFeature } from '#/pages/hr/feature/members-feature'
import { resetRhStore } from '#/pages/hr/feature/mocks/rh-store'
import { renderWithRouter } from '#/test/render-with-router'
import {
	ABSENCE_TYPES_PATH,
	ABSENCES_PATH,
	DEFAULT_TYPE_RECORDS,
} from '#/test/rh-absence-fixtures'
import { EXTRAS_PATH, SURCHARGES_PATH } from '#/test/rh-extra-fixtures'
import {
	DEFAULT_ACCOUNT_RECORDS,
	LEAVE_ACCOUNTS_PATH,
	LEAVE_ADJUSTMENTS_PATH,
} from '#/test/rh-leave-fixtures'

const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const EMPLOYEE_PROFILES_PATH =
	'/api/v1/organizations/{organization_id}/employee-profiles'
const PROFILES_PATH = '/api/v1/organizations/{organization_id}/member-profiles'
const CONTRACTS_PATH =
	'/api/v1/organizations/{organization_id}/member-contracts'
const DOCUMENTS_PATH =
	'/api/v1/organizations/{organization_id}/member-documents'
const INVITATIONS_PATH = '/api/v1/organizations/{organization_id}/invitations'
const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'
const ROLES_PATH = '/api/v1/organizations/{organization_id}/roles'
const MEMBER_ROLES_PATH = '/api/v1/members/{member_id}/roles'

function role(overrides: Partial<Role> = {}): Role {
	return {
		id: 'role-1',
		organization_id: 'org-1',
		name: 'Administrateur',
		permissions: [],
		is_seeded: false,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
	field_clock_enabled: false,
	vat_on_debits: false,
	mcp_enabled: false,
	missing_legal_identity_fields: [],
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
	postMember?: (params: unknown) => unknown
	onMemberCreated?: (memberId: string) => void
	deleteInvitation?: (params: unknown) => unknown
	employeeProfilesForbidden?: boolean
	permissions?: string[]
	roles?: Role[]
	memberRoleIds?: Record<string, string[]>
	assignRole?: (params: {
		path: { member_id: string }
		body: { role_id: string }
	}) => unknown
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
	const calls: { method: string; path: string; params: unknown }[] = []
	const invitations = handlers.invitations ?? []
	const permissions = handlers.permissions ?? ['MANAGE_MEMBERS', 'MANAGE_ROLES']
	const roles = handlers.roles ?? []
	const memberRoleIds = handlers.memberRoleIds ?? {}

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
							if (handlers.employeeProfilesForbidden) {
								throw Object.assign(new Error('Forbidden'), {
									status: 403,
									data: {
										code: 'E_FORBIDDEN',
										message: 'Forbidden',
										status: 403,
									},
								})
							}
							return { data: [], pagination: null }
						}
						if (path === ABSENCES_PATH) {
							return { data: [], pagination: null }
						}
						if (path === ABSENCE_TYPES_PATH) {
							return { data: DEFAULT_TYPE_RECORDS, pagination: null }
						}
						if (
							path === PROFILES_PATH ||
							path === CONTRACTS_PATH ||
							path === DOCUMENTS_PATH
						) {
							return { data: [], pagination: null }
						}
						if (path === INVITATIONS_PATH) {
							return { data: invitations, pagination: null }
						}
						if (path === MY_PERMISSIONS_PATH) {
							return { data: { permissions }, pagination: null }
						}
						if (path === ROLES_PATH) {
							return { data: roles, pagination: null }
						}
						if (path === MEMBER_ROLES_PATH) {
							const memberId = (
								params as { path?: { member_id?: string } } | undefined
							)?.path?.member_id
							return {
								data: { role_ids: memberRoleIds[memberId ?? ''] ?? [] },
								pagination: null,
							}
						}
						if (path === LEAVE_ACCOUNTS_PATH)
							return { data: DEFAULT_ACCOUNT_RECORDS, pagination: null }
						if (path === LEAVE_ADJUSTMENTS_PATH)
							return { data: [], pagination: null }
						if (path === SURCHARGES_PATH || path === EXTRAS_PATH)
							return { data: [], pagination: null }
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
						if (method === 'post' && path === MEMBERS_PATH) {
							if (!handlers.postMember) throw new Error('postMember not mocked')
							return handlers.postMember(params)
						}
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
						if (method === 'post' && path === MEMBER_ROLES_PATH) {
							if (!handlers.assignRole) {
								throw new Error('assignRole not mocked')
							}
							return handlers.assignRole(
								params as {
									path: { member_id: string }
									body: { role_id: string }
								},
							)
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
			<MembersFeature
				status="active"
				contract="all"
				onFilterChange={() => {}}
				onMemberCreated={handlers.onMemberCreated ?? (() => {})}
			/>
		</Providers>,
	).then((result) => ({ ...result, calls }))
}

beforeEach(() => {
	resetRhStore()
})

describe('MembersFeature — invite gesture', () => {
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

		expect(screen.getByText('Inviter Alix Nova')).toBeDefined()
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

		await user.click(screen.getByRole('button', { name: 'Fermer' }))
		await user.click(await screen.findByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Inviter/ }))

		expect(screen.queryByLabelText('Lien d’invitation')).toBeNull()
		expect(
			screen.getByRole('button', { name: /Générer le lien/ }),
		).toBeDefined()
	})
})

describe('MembersFeature — pending invitations', () => {
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
		expect(screen.getAllByText('Alix Nova').length).toBeGreaterThan(0)

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

describe('MembersFeature — employee profiles forbidden (#371)', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('shows the team and a neutral notice instead of a fatal error banner', async () => {
		await renderFeature({ employeeProfilesForbidden: true })

		expect(await screen.findByText('Alix Nova')).toBeDefined()
		expect(
			screen.getByText(/n’avez pas la permission de consulter/),
		).toBeDefined()
		expect(screen.queryByText('Forbidden')).toBeNull()
	})

	it('says "Non consultable" rather than the misleading "Sans profil RH"', async () => {
		await renderFeature({ employeeProfilesForbidden: true })

		expect(await screen.findByText('Non consultable')).toBeDefined()
		expect(screen.queryByText('Sans profil RH')).toBeNull()
	})
})

describe('MembersFeature — role assignment (#308)', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('shows the held role and assigns a new one through the whole page', async () => {
		const user = userEvent.setup()
		const memberRoleIds: Record<string, string[]> = {
			'member-1': ['role-admin'],
		}
		const { calls } = await renderFeature({
			roles: [
				role({ id: 'role-admin', name: 'Administrateur' }),
				role({ id: 'role-compta', name: 'Comptabilité' }),
			],
			memberRoleIds,
			assignRole: (params) => {
				memberRoleIds[params.path.member_id] = [
					...(memberRoleIds[params.path.member_id] ?? []),
					params.body.role_id,
				]
				return { data: undefined }
			},
		})

		expect(await screen.findByText('Administrateur')).toBeDefined()

		await user.click(
			await screen.findByRole('button', { name: 'Assigner un rôle' }),
		)
		await user.click(
			await screen.findByRole('button', { name: 'Comptabilité' }),
		)

		await waitFor(() => {
			const assignCall = calls.find(
				(c) => c.method === 'post' && c.path === MEMBER_ROLES_PATH,
			)
			expect(assignCall?.params).toMatchObject({
				path: { member_id: 'member-1' },
				body: { role_id: 'role-compta' },
			})
		})

		expect(await screen.findByText('Comptabilité')).toBeDefined()
	})
})

describe('MembersFeature — the prototype list', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('links each member to the detail page and shows the prototype columns', async () => {
		await renderFeature()

		const link = await screen.findByRole('link', { name: 'Alix Nova' })
		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/rh/membres/member-1',
		)
		expect(screen.getByText('Semaine type')).toBeDefined()
		expect(screen.getByText('Responsable')).toBeDefined()
		expect(screen.getByText('1 membre actif')).toBeDefined()
	})

	it('narrows the list on the search box and offers to reset when nothing matches', async () => {
		const user = userEvent.setup()
		await renderFeature()

		await screen.findByRole('link', { name: 'Alix Nova' })
		await user.type(
			screen.getByRole('textbox', { name: 'Rechercher un membre' }),
			'zzz',
		)

		expect(screen.getByText('Aucun membre ne correspond.')).toBeDefined()
		await user.click(
			screen.getByRole('button', { name: 'Réinitialiser les filtres' }),
		)
		expect(await screen.findByRole('link', { name: 'Alix Nova' })).toBeDefined()
	})

	it('creates a real member and hands its id to the route', async () => {
		const user = userEvent.setup()
		const onMemberCreated = vi.fn()
		const { calls } = await renderFeature({
			onMemberCreated,
			postMember: () => ({ data: { id: 'member-9' } }),
		})

		await user.click(
			await screen.findByRole('button', { name: /Nouveau membre/ }),
		)
		await user.type(screen.getByLabelText('Nom'), 'Durand')
		await user.click(screen.getByRole('button', { name: 'Ajouter' }))

		await waitFor(() =>
			expect(onMemberCreated).toHaveBeenCalledWith('member-9'),
		)
		const post = calls.find(
			(c) => c.method === 'post' && c.path === MEMBERS_PATH,
		)
		expect(post?.params).toMatchObject({
			path: { organization_id: 'org-1' },
			body: { last_name: 'Durand', first_name: null },
		})
	})

	it('links the work time page of the member from the row menu', async () => {
		const user = userEvent.setup()
		await renderFeature()

		await user.click(await screen.findByRole('button', { name: 'Actions' }))
		const link = screen.getByRole('menuitem', { name: /Temps de travail/ })
		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/rh/membres/member-1/temps-de-travail',
		)
	})
})

describe('MembersFeature — permission gating', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('hides creation and editing without MANAGE_MEMBERS', async () => {
		const user = userEvent.setup()
		await renderFeature({ permissions: [] })

		await screen.findByRole('link', { name: 'Alix Nova' })
		expect(screen.queryByRole('button', { name: /Nouveau membre/ })).toBeNull()
		await user.click(screen.getByRole('button', { name: 'Actions' }))
		expect(screen.queryByRole('menuitem', { name: 'Modifier' })).toBeNull()
		expect(screen.queryByRole('menuitem', { name: /Supprimer/ })).toBeNull()
	})
})

function paramsBody(params: unknown) {
	return (params as { body?: unknown }).body ?? {}
}
