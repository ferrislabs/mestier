import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { RolesSection } from '#/pages/settings/sections/roles-section'
import { renderWithRouter } from '#/test/render-with-router'

const ROLES_PATH = '/api/v1/organizations/{organization_id}/roles'
const ROLE_PATH = '/api/v1/roles/{role_id}'
const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	field_clock_enabled: false,
	vat_on_debits: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const OWNER_ROLE = {
	id: 'role-owner',
	organization_id: 'org-1',
	name: 'owner',
	permissions: [
		'MANAGE_ORG',
		'MANAGE_MEMBERS',
		'MANAGE_ROLES',
		'VIEW_PLANNING',
		'MANAGE_PLANNING',
		'VIEW_COST',
		'MANAGE_COST',
		'VIEW_REPORTS',
	],
	is_seeded: true,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const ADMIN_ROLE = {
	id: 'role-admin',
	organization_id: 'org-1',
	name: 'admin',
	permissions: ['MANAGE_MEMBERS', 'VIEW_PLANNING', 'MANAGE_PLANNING'],
	is_seeded: true,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const MEMBER_ROLE = {
	id: 'role-member',
	organization_id: 'org-1',
	name: 'member',
	permissions: ['VIEW_PLANNING'],
	is_seeded: true,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const CUSTOM_ROLE = {
	id: 'role-custom',
	organization_id: 'org-1',
	name: 'Chef de chantier',
	permissions: ['VIEW_PLANNING', 'MANAGE_PLANNING'],
	is_seeded: false,
	created_at: '2026-02-01T00:00:00Z',
	updated_at: '2026-02-01T00:00:00Z',
}

const DEFAULT_ROLES = [OWNER_ROLE, ADMIN_ROLE, MEMBER_ROLE, CUSTOM_ROLE]

interface FakeApiHandlers {
	roles?: unknown[]
	permissions?: string[]
	createRole?: (params: unknown) => unknown
	updateRole?: (params: unknown) => unknown
	deleteRole?: (params: unknown) => unknown
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
	const calls: { method: string; path: string; params: unknown }[] = []
	const roles = handlers.roles ?? DEFAULT_ROLES
	const permissions = handlers.permissions ?? ['MANAGE_ROLES']

	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = [
				{ _id: path, path: (params as { path?: unknown })?.path },
			]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params })
						if (path === ROLES_PATH) return { data: roles }
						if (path === MY_PERMISSIONS_PATH) {
							return { data: { permissions }, pagination: null }
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
						if (method === 'post' && path === ROLES_PATH) {
							if (!handlers.createRole) {
								throw new Error('createRole not mocked')
							}
							return handlers.createRole(params)
						}
						if (method === 'patch' && path === ROLE_PATH) {
							if (!handlers.updateRole) {
								throw new Error('updateRole not mocked')
							}
							return handlers.updateRole(params)
						}
						if (method === 'delete' && path === ROLE_PATH) {
							if (!handlers.deleteRole) {
								throw new Error('deleteRole not mocked')
							}
							return handlers.deleteRole(params)
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

async function renderSection(handlers: FakeApiHandlers = {}) {
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

	const result = await renderWithRouter(
		<Providers>
			<RolesSection />
		</Providers>,
	)
	return { ...result, calls }
}

describe('RolesSection — gating', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('shows the role list to a caller holding MANAGE_ROLES', async () => {
		await renderSection({ permissions: ['MANAGE_ROLES'] })

		expect(await screen.findByText('owner')).toBeDefined()
		expect(screen.getByText('Chef de chantier')).toBeDefined()
	})

	it('shows a fallback explanation, not the list, without MANAGE_ROLES', async () => {
		await renderSection({ permissions: [] })

		expect(
			await screen.findByText(
				"Vous n'avez pas la permission de gérer les rôles de cette organisation.",
			),
		).toBeDefined()
		expect(screen.queryByText('owner')).toBeNull()
		expect(screen.queryByRole('button', { name: 'Créer un rôle' })).toBeNull()
	})
})

describe('RolesSection — list', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('lists every role and shows the seeded badge for owner, admin and member', async () => {
		await renderSection({})

		for (const name of ['owner', 'admin', 'member', 'Chef de chantier']) {
			expect(await screen.findByText(name)).toBeDefined()
		}

		const seededBadges = screen.getAllByText('Rôle prédéfini')
		expect(seededBadges).toHaveLength(3)
	})
})

describe('RolesSection — create', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('creates a role with the checked permissions', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection({
			createRole: (params) => ({
				data: {
					id: 'role-new',
					organization_id: 'org-1',
					is_seeded: false,
					created_at: '2026-08-01T00:00:00Z',
					updated_at: '2026-08-01T00:00:00Z',
					...(params as { body: { name: string; permissions: string[] } }).body,
				},
			}),
		})

		await user.click(
			await screen.findByRole('button', { name: 'Créer un rôle' }),
		)

		const sheet = await screen.findByRole('dialog')
		await user.type(within(sheet).getByLabelText('Nom'), 'Poseur')
		await user.click(within(sheet).getByLabelText('Voir le planning'))
		await user.click(within(sheet).getByLabelText('Modifier le planning'))
		await user.click(within(sheet).getByRole('button', { name: 'Créer' }))

		await waitFor(() => {
			const call = calls.find(
				(c) => c.method === 'post' && c.path === ROLES_PATH,
			)
			expect(call?.params).toMatchObject({
				path: { organization_id: 'org-1' },
				body: {
					name: 'Poseur',
					permissions: ['VIEW_PLANNING', 'MANAGE_PLANNING'],
				},
			})
		})
	})

	it('groups the permission editor by area and surfaces a note next to VIEW_COST', async () => {
		const user = userEvent.setup()
		await renderSection({})

		await user.click(
			await screen.findByRole('button', { name: 'Créer un rôle' }),
		)
		const sheet = await screen.findByRole('dialog')

		expect(within(sheet).getByText('Planning')).toBeDefined()
		expect(within(sheet).getByText('Coûts')).toBeDefined()
		expect(within(sheet).getByText('Administration')).toBeDefined()

		expect(
			within(sheet).getByText(/Avec « Voir la rentabilité » mais sans ce bit/),
		).toBeDefined()
	})
})

describe('RolesSection — edit', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('disables the name field for a seeded role', async () => {
		const user = userEvent.setup()
		await renderSection({})

		const ownerRow = (await screen.findByText('owner')).closest('li')
		expect(ownerRow).not.toBeNull()
		await user.click(
			within(ownerRow as HTMLElement).getByRole('button', { name: 'Modifier' }),
		)

		const sheet = await screen.findByRole('dialog')
		expect(within(sheet).getByLabelText('Nom')).toHaveProperty('disabled', true)
		expect(
			within(sheet).getByText(
				"Le nom d'un rôle prédéfini ne peut pas être modifié.",
			),
		).toBeDefined()
	})

	it('leaves the name field editable for a custom role', async () => {
		const user = userEvent.setup()
		await renderSection({})

		const customRow = (await screen.findByText('Chef de chantier')).closest(
			'li',
		)
		expect(customRow).not.toBeNull()
		await user.click(
			within(customRow as HTMLElement).getByRole('button', {
				name: 'Modifier',
			}),
		)

		const sheet = await screen.findByRole('dialog')
		expect(within(sheet).getByLabelText('Nom')).toHaveProperty(
			'disabled',
			false,
		)
	})
})

describe('RolesSection — delete', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('offers no delete control for a seeded role', async () => {
		await renderSection({})

		const ownerRow = (await screen.findByText('owner')).closest('li')
		expect(ownerRow).not.toBeNull()
		expect(
			within(ownerRow as HTMLElement).queryByRole('button', {
				name: /Supprimer/,
			}),
		).toBeNull()
	})

	it('confirms then calls the delete mutation for a custom role', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection({
			deleteRole: () => ({ data: null }),
		})

		const customRow = (await screen.findByText('Chef de chantier')).closest(
			'li',
		)
		expect(customRow).not.toBeNull()
		await user.click(
			within(customRow as HTMLElement).getByRole('button', {
				name: /Supprimer/,
			}),
		)

		const confirmDialog = await screen.findByRole('alertdialog')
		await user.click(
			within(confirmDialog).getByRole('button', { name: 'Supprimer' }),
		)

		await waitFor(() => {
			const call = calls.find(
				(c) => c.method === 'delete' && c.path === ROLE_PATH,
			)
			expect(call?.params).toMatchObject({ path: { role_id: 'role-custom' } })
		})
	})

	it('shows the fallback sentence on a 409 rather than the raw backend text', async () => {
		const user = userEvent.setup()
		await renderSection({
			deleteRole: () => {
				throw { status: 409, message: 'role still assigned to a member' }
			},
		})

		const customRow = (await screen.findByText('Chef de chantier')).closest(
			'li',
		)
		expect(customRow).not.toBeNull()
		await user.click(
			within(customRow as HTMLElement).getByRole('button', {
				name: /Supprimer/,
			}),
		)

		const confirmDialog = await screen.findByRole('alertdialog')
		await user.click(
			within(confirmDialog).getByRole('button', { name: 'Supprimer' }),
		)

		expect(
			await screen.findByText(
				'Ce rôle est encore attribué à au moins un membre ; retirez-le de ce rôle avant de le supprimer.',
			),
		).toBeDefined()
	})
})
