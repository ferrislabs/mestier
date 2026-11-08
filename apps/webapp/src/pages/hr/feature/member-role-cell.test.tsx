import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import type { Role } from '#/hooks/use-roles'
import { MemberRoleCell } from '#/pages/hr/feature/member-role-cell'
import { renderWithRouter as renderWithRouterBase } from '#/test/render-with-router'

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

const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'
const ROLES_PATH = '/api/v1/organizations/{organization_id}/roles'
const MEMBER_ROLES_PATH = '/api/v1/members/{member_id}/roles'
const MEMBER_ROLE_PATH = '/api/v1/members/{member_id}/roles/{role_id}'

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

interface FakeApiOptions {
	permissions?: string[]
	roles?: Role[]
	memberRoleIds?: Record<string, string[]>
	onAssignRole?: (params: {
		path: { member_id: string }
		body: { role_id: string }
	}) => unknown
	onUnassignRole?: (params: {
		path: { member_id: string; role_id: string }
	}) => unknown
}

function installFakePermissionsApi(options: FakeApiOptions = {}) {
	const permissions = options.permissions ?? ['MANAGE_MEMBERS', 'MANAGE_ROLES']
	const roles = options.roles ?? []
	const memberRoleIds = options.memberRoleIds ?? {}

	const fakeApi = {
		get(path: string, params?: { path?: { member_id?: string } }) {
			const queryKey = [{ _id: path, path: params?.path }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === MY_PERMISSIONS_PATH) {
							return { data: { permissions }, pagination: null }
						}
						if (path === ROLES_PATH) {
							return { data: roles, pagination: null }
						}
						if (path === MEMBER_ROLES_PATH) {
							const memberId = params?.path?.member_id ?? ''
							return {
								data: { role_ids: memberRoleIds[memberId] ?? [] },
								pagination: null,
							}
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
						if (method === 'post' && path === MEMBER_ROLES_PATH) {
							if (!options.onAssignRole) {
								throw new Error('onAssignRole not mocked')
							}
							return options.onAssignRole(
								params as {
									path: { member_id: string }
									body: { role_id: string }
								},
							)
						}
						if (method === 'delete' && path === MEMBER_ROLE_PATH) {
							if (!options.onUnassignRole) {
								throw new Error('onUnassignRole not mocked')
							}
							return options.onUnassignRole(
								params as {
									path: { member_id: string; role_id: string }
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
}

function renderWithRouter(
	ui: ReactNode,
	initialPath?: string,
	apiOptions?: FakeApiOptions,
) {
	installFakePermissionsApi(apiOptions)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	return renderWithRouterBase(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					{ui}
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
		initialPath,
	)
}

describe('MemberRoleCell — role assignment (#308)', () => {
	it("renders a member's held roles as badges, resolved against the organization's role list", async () => {
		await renderWithRouter(<MemberRoleCell memberId="member-1" />, undefined, {
			roles: [
				role({ id: 'role-admin', name: 'Administrateur' }),
				role({ id: 'role-compta', name: 'Comptabilité' }),
			],
			memberRoleIds: { 'member-1': ['role-admin'] },
		})

		expect(await screen.findByText('Administrateur')).toBeDefined()
		expect(screen.queryByText('Comptabilité')).toBeNull()
	})

	it('says "Aucun rôle" outright for a member holding none, rather than a blank cell', async () => {
		await renderWithRouter(<MemberRoleCell memberId="member-1" />, undefined, {
			roles: [role({ id: 'role-admin', name: 'Administrateur' })],
			memberRoleIds: {},
		})

		expect(await screen.findByText('Aucun rôle')).toBeDefined()
	})

	it('offers the assign control when the fixture grants MANAGE_ROLES', async () => {
		await renderWithRouter(<MemberRoleCell memberId="member-1" />, undefined, {
			permissions: ['MANAGE_ROLES'],
			roles: [role({ id: 'role-admin', name: 'Administrateur' })],
		})

		expect(
			await screen.findByRole('button', { name: 'Assigner un rôle' }),
		).toBeDefined()
	})

	it('hides the assign control when the fixture does not grant MANAGE_ROLES', async () => {
		await renderWithRouter(<MemberRoleCell memberId="member-1" />, undefined, {
			permissions: ['MANAGE_MEMBERS'],
			roles: [role({ id: 'role-admin', name: 'Administrateur' })],
		})

		expect(await screen.findByText('Aucun rôle')).toBeDefined()
		expect(
			screen.queryByRole('button', { name: 'Assigner un rôle' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: /déjà tous les rôles/ }),
		).toBeNull()
	})

	it('calls the assign mutation with the right member and role id, and the row picks up the new badge', async () => {
		const user = userEvent.setup()
		const memberRoleIds: Record<string, string[]> = { 'member-1': [] }
		const onAssignRole = vi.fn(
			(params: { path: { member_id: string }; body: { role_id: string } }) => {
				memberRoleIds[params.path.member_id] = [
					...(memberRoleIds[params.path.member_id] ?? []),
					params.body.role_id,
				]
				return { data: undefined }
			},
		)

		await renderWithRouter(<MemberRoleCell memberId="member-1" />, undefined, {
			permissions: ['MANAGE_ROLES'],
			roles: [role({ id: 'role-compta', name: 'Comptabilité' })],
			memberRoleIds,
			onAssignRole,
		})

		await user.click(
			await screen.findByRole('button', { name: 'Assigner un rôle' }),
		)
		await user.click(
			await screen.findByRole('button', { name: 'Comptabilité' }),
		)

		await waitFor(() => {
			expect(onAssignRole).toHaveBeenCalledWith({
				path: { member_id: 'member-1' },
				body: { role_id: 'role-compta' },
			})
		})

		expect(await screen.findByText('Comptabilité')).toBeDefined()
		expect(screen.queryByText('Aucun rôle')).toBeNull()
	})

	it('does not offer an empty picker once a member already holds every role', async () => {
		const user = userEvent.setup()
		await renderWithRouter(<MemberRoleCell memberId="member-1" />, undefined, {
			permissions: ['MANAGE_ROLES'],
			roles: [role({ id: 'role-admin', name: 'Administrateur' })],
			memberRoleIds: { 'member-1': ['role-admin'] },
		})

		const button = await screen.findByRole('button', {
			name: 'Ce membre a déjà tous les rôles',
		})
		expect((button as HTMLButtonElement).disabled).toBe(true)
		expect(
			screen.queryByRole('button', { name: 'Assigner un rôle' }),
		).toBeNull()

		await user.click(button)
		expect(
			screen.queryByText('Administrateur', { selector: 'button' }),
		).toBeNull()
	})
})

describe('MemberRoleCell — role unassignment (#401, #402)', () => {
	it('offers a remove control on a held role when the fixture grants MANAGE_ROLES', async () => {
		await renderWithRouter(<MemberRoleCell memberId="member-1" />, undefined, {
			permissions: ['MANAGE_ROLES'],
			roles: [role({ id: 'role-admin', name: 'Administrateur' })],
			memberRoleIds: { 'member-1': ['role-admin'] },
		})

		expect(
			await screen.findByRole('button', {
				name: 'Retirer le rôle Administrateur',
			}),
		).toBeDefined()
	})

	it('hides the remove control when the fixture does not grant MANAGE_ROLES', async () => {
		await renderWithRouter(<MemberRoleCell memberId="member-1" />, undefined, {
			permissions: ['MANAGE_MEMBERS'],
			roles: [role({ id: 'role-admin', name: 'Administrateur' })],
			memberRoleIds: { 'member-1': ['role-admin'] },
		})

		expect(await screen.findByText('Administrateur')).toBeDefined()
		expect(screen.queryByRole('button', { name: /Retirer le rôle/ })).toBeNull()
	})

	it('calls the unassign mutation with the right member and role id, and the badge disappears', async () => {
		const user = userEvent.setup()
		const memberRoleIds: Record<string, string[]> = {
			'member-1': ['role-admin'],
		}
		const onUnassignRole = vi.fn(
			(params: { path: { member_id: string; role_id: string } }) => {
				memberRoleIds[params.path.member_id] = (
					memberRoleIds[params.path.member_id] ?? []
				).filter((id) => id !== params.path.role_id)
				return { data: undefined }
			},
		)

		await renderWithRouter(<MemberRoleCell memberId="member-1" />, undefined, {
			permissions: ['MANAGE_ROLES'],
			roles: [role({ id: 'role-admin', name: 'Administrateur' })],
			memberRoleIds,
			onUnassignRole,
		})

		await user.click(
			await screen.findByRole('button', {
				name: 'Retirer le rôle Administrateur',
			}),
		)

		await waitFor(() => {
			expect(onUnassignRole).toHaveBeenCalledWith({
				path: { member_id: 'member-1', role_id: 'role-admin' },
			})
		})

		expect(await screen.findByText('Aucun rôle')).toBeDefined()
	})

	it('shows a search box in the assign popover once there are more than five assignable roles', async () => {
		const user = userEvent.setup()
		const roles = Array.from({ length: 6 }, (_, index) =>
			role({ id: `role-${index}`, name: `Rôle ${index}` }),
		)
		await renderWithRouter(<MemberRoleCell memberId="member-1" />, undefined, {
			permissions: ['MANAGE_ROLES'],
			roles,
		})

		await user.click(
			await screen.findByRole('button', { name: 'Assigner un rôle' }),
		)

		await user.type(
			screen.getByPlaceholderText('Rechercher un rôle…'),
			'Rôle 3',
		)

		expect(screen.getByRole('button', { name: 'Rôle 3' })).toBeDefined()
		expect(screen.queryByRole('button', { name: 'Rôle 0' })).toBeNull()
	})

	it('does not show a search box in the assign popover for five or fewer assignable roles', async () => {
		const user = userEvent.setup()
		await renderWithRouter(<MemberRoleCell memberId="member-1" />, undefined, {
			permissions: ['MANAGE_ROLES'],
			roles: [role({ id: 'role-admin', name: 'Administrateur' })],
		})

		await user.click(
			await screen.findByRole('button', { name: 'Assigner un rôle' }),
		)

		expect(screen.queryByPlaceholderText('Rechercher un rôle…')).toBeNull()
	})
})
