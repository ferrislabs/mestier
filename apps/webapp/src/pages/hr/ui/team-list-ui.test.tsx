import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Role } from '#/hooks/use-roles'
import type {
	MemberDraft,
	PendingInvitationRow,
	TeamMemberRow,
} from '#/pages/hr/ui/team-list-ui'
import { TeamListUI } from '#/pages/hr/ui/team-list-ui'
import { renderWithRouter as renderWithRouterBase } from '#/test/render-with-router'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
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
	/** member id -> role ids held. */
	memberRoleIds?: Record<string, string[]>
	onAssignRole?: (params: {
		path: { member_id: string }
		body: { role_id: string }
	}) => unknown
	onUnassignRole?: (params: {
		path: { member_id: string; role_id: string }
	}) => unknown
}

/**
 * `TeamListUI` renders its "Ajouter une personne" action, and the per-row
 * role assignment control, behind `RequirePermission` (#307, #308), and
 * resolves a member's role badges through `useRoles`/`useMemberRoleIds`
 * directly (see `MemberRoleCell`) — all three need a fake `tanstackApi`.
 * Grants every bit by default since most of these tests don't exercise
 * gating itself (covered by `require-permission.test.tsx`).
 */
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

function member(overrides: Partial<TeamMemberRow> = {}): TeamMemberRow {
	return {
		id: 'member-1',
		displayName: 'Martin Alix',
		access: 'none',
		hourlyRateCents: 1500,
		isSalaried: false,
		monthlyCostCents: null,
		effectiveHourlyRateCents: 1500,
		weeklyContractMinutes: 2100,
		...overrides,
	}
}

function baseProps() {
	return {
		organizationSlug: 'atelier-bois',
		isLoading: false,
		error: null,
		hrDataRestricted: false,
		members: [member()],
		search: '',
		onSearchChange: vi.fn(),
		createForm: {
			values: {
				lastName: '',
				firstName: '',
				hourlyRate: '',
				isSalaried: false,
				monthlyCost: '',
			},
			isPending: false,
			onChange: vi.fn(),
			onSubmit: vi.fn(),
		},
		createMemberDialogOpen: false,
		onCreateMemberDialogOpenChange: vi.fn(),
		draft: null as MemberDraft | null,
		isSaving: false,
		onEdit: vi.fn(),
		onDraftChange: vi.fn(),
		onCancelEdit: vi.fn(),
		onSaveEdit: vi.fn(),
		onDeleteMember: vi.fn().mockResolvedValue(undefined),
		onInvite: vi.fn(),
		pendingInvitations: [] as PendingInvitationRow[],
		revokingInvitationId: null as string | null,
		onRevokeInvitation: vi.fn(),
	}
}

describe('TeamListUI — no UUID anywhere', () => {
	it('never renders the member id', async () => {
		await renderWithRouter(<TeamListUI {...baseProps()} />)

		expect(screen.getByText('Martin Alix')).toBeDefined()
		expect(screen.queryByText(/member-1/)).toBeNull()
	})
})

describe('TeamListUI — access column', () => {
	it('shows "Aucun accès" for a seat with no account', async () => {
		await renderWithRouter(
			<TeamListUI {...baseProps()} members={[member({ access: 'none' })]} />,
		)

		expect(screen.getByText('Aucun accès')).toBeDefined()
	})

	it('shows "Compte lié" for a seat with a linked account', async () => {
		await renderWithRouter(
			<TeamListUI
				{...baseProps()}
				members={[member({ access: 'linkedAccount' })]}
			/>,
		)

		expect(screen.getByText('Compte lié')).toBeDefined()
	})
})

describe('TeamListUI — rate and contract', () => {
	it('shows a placeholder in the contract column when the seat has no profile', async () => {
		await renderWithRouter(
			<TeamListUI
				{...baseProps()}
				members={[
					member({ hourlyRateCents: null, weeklyContractMinutes: null }),
				]}
			/>,
		)

		expect(screen.getByText('Non renseigné')).toBeDefined()
		expect(screen.getByText('Sans profil RH')).toBeDefined()
	})
})

describe('TeamListUI — HR data forbidden (#371)', () => {
	it('shows a neutral notice instead of the red error banner', async () => {
		await renderWithRouter(
			<TeamListUI {...baseProps()} hrDataRestricted={true} />,
		)

		expect(
			screen.getByText(/n’avez pas la permission de consulter/),
		).toBeDefined()
	})

	it('says "Non consultable", never "Sans profil RH", when HR data is restricted', async () => {
		await renderWithRouter(
			<TeamListUI
				{...baseProps()}
				hrDataRestricted={true}
				members={[
					member({ hourlyRateCents: null, weeklyContractMinutes: null }),
				]}
			/>,
		)

		expect(screen.getByText('Non consultable')).toBeDefined()
		expect(screen.queryByText('Sans profil RH')).toBeNull()
	})

	it('still shows the real error banner when there is an actual failure', async () => {
		await renderWithRouter(
			<TeamListUI
				{...baseProps()}
				error="Impossible de contacter le serveur"
				hrDataRestricted={true}
			/>,
		)

		expect(screen.getByText('Impossible de contacter le serveur')).toBeDefined()
		expect(
			screen.queryByText(/n’avez pas la permission de consulter/),
		).toBeNull()
	})
})

describe('TeamListUI — access to the work time screen', () => {
	it("exposes a link to the member's work time, keyed on the team route", async () => {
		const user = userEvent.setup()
		await renderWithRouter(<TeamListUI {...baseProps()} />)

		await user.click(screen.getByRole('button', { name: 'Actions' }))

		// Radix's `asChild` merges the anchor into the menu item and overrides
		// its role to `menuitem` — it is still a real `<a href>` underneath.
		const link = screen.getByRole('menuitem', { name: /Temps de travail/ })
		expect(link.tagName).toBe('A')
		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/hr/team/member-1/work-time',
		)
	})
})

describe('TeamListUI — invite action', () => {
	it('offers "Inviter" for a seat with no access, and calls onInvite with the row', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(
			<TeamListUI {...props} members={[member({ access: 'none' })]} />,
		)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Inviter/ }))

		expect(props.onInvite).toHaveBeenCalledWith(member({ access: 'none' }))
	})

	it('hides "Inviter" once a seat is invited or linked', async () => {
		const user = userEvent.setup()
		await renderWithRouter(
			<TeamListUI {...baseProps()} members={[member({ access: 'invited' })]} />,
		)

		await user.click(screen.getByRole('button', { name: 'Actions' }))

		expect(screen.queryByRole('menuitem', { name: /Inviter/ })).toBeNull()
	})
})

describe('TeamListUI — pending invitations panel', () => {
	it('renders nothing when there is no pending invitation', async () => {
		await renderWithRouter(<TeamListUI {...baseProps()} />)

		expect(screen.queryByText(/Invitations en attente/)).toBeNull()
	})

	it('lists a pending invitation and revokes it on demand', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		props.pendingInvitations = [
			{ id: 'invitation-1', memberName: 'Alix Nova', expiresAt: '2026-08-20' },
		]
		await renderWithRouter(<TeamListUI {...props} />)

		expect(screen.getByText('Invitations en attente (1)')).toBeDefined()
		expect(screen.getByText('Alix Nova')).toBeDefined()

		await user.click(screen.getByRole('button', { name: /Révoquer/ }))

		expect(props.onRevokeInvitation).toHaveBeenCalledWith('invitation-1')
	})
})

describe('TeamListUI — deletion goes through a confirmation dialog', () => {
	it('does not call onDeleteMember until the confirmation is accepted', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(<TeamListUI {...props} />)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Supprimer/ }))

		expect(props.onDeleteMember).not.toHaveBeenCalled()
		expect(
			screen.getByRole('alertdialog', { name: /Supprimer Martin Alix/ }),
		).toBeDefined()

		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		expect(props.onDeleteMember).toHaveBeenCalledWith(member())
	})
})

describe('TeamListUI — create member modal', () => {
	it('is closed by default, with no create fields visible on the page', async () => {
		await renderWithRouter(<TeamListUI {...baseProps()} />)

		expect(screen.queryByLabelText('Nom')).toBeNull()
	})

	it('"Ajouter une personne" asks the feature to open the modal', async () => {
		const user = userEvent.setup()
		const props = baseProps()
		await renderWithRouter(<TeamListUI {...props} />)

		await user.click(
			screen.getByRole('button', { name: 'Ajouter une personne' }),
		)

		expect(props.onCreateMemberDialogOpenChange).toHaveBeenCalledWith(true)
	})

	it('shows the create fields once open, and submits through the create form binding', async () => {
		const user = userEvent.setup()
		const props = { ...baseProps(), createMemberDialogOpen: true }
		await renderWithRouter(<TeamListUI {...props} />)

		expect(screen.getByLabelText('Nom')).toBeDefined()

		await user.click(screen.getByRole('button', { name: 'Ajouter' }))

		expect(props.createForm.onSubmit).toHaveBeenCalled()
	})

	/**
	 * A salaried person has no hourly figure — toggling the switch must clear
	 * whatever the rate field held, not leave a stale value behind it.
	 */
	it('clears the hourly rate when "Salarié" is toggled on', async () => {
		const user = userEvent.setup()
		const props = {
			...baseProps(),
			createMemberDialogOpen: true,
			createForm: {
				...baseProps().createForm,
				values: {
					lastName: '',
					firstName: '',
					hourlyRate: '35',
					isSalaried: false,
					monthlyCost: '',
				},
			},
		}
		await renderWithRouter(<TeamListUI {...props} />)

		await user.click(screen.getByRole('switch', { name: /Salarié/ }))

		expect(props.createForm.onChange).toHaveBeenCalledWith({
			isSalaried: true,
			hourlyRate: '',
			monthlyCost: '',
		})
	})

	it('disables the hourly rate field while "Salarié" is on', async () => {
		const props = {
			...baseProps(),
			createMemberDialogOpen: true,
			createForm: {
				...baseProps().createForm,
				values: {
					lastName: '',
					firstName: '',
					hourlyRate: '',
					isSalaried: true,
					monthlyCost: '',
				},
			},
		}
		await renderWithRouter(<TeamListUI {...props} />)

		expect(
			(screen.getByLabelText('Taux horaire') as HTMLInputElement).disabled,
		).toBe(true)
	})
})

describe('TeamListUI — salaried row display', () => {
	/**
	 * The regression. This column used to read just "Salarié", which said nothing
	 * about cost, and profitability counted their hours at 0,00 €. It now shows
	 * the monthly amount and the hourly figure derived from it.
	 */
	it('shows a salaried member on both bases: monthly, and the hourly equivalent', async () => {
		const props = {
			...baseProps(),
			members: [
				member({
					isSalaried: true,
					hourlyRateCents: null,
					monthlyCostCents: 350_000,
					effectiveHourlyRateCents: 2_308,
				}),
			],
		}
		await renderWithRouter(<TeamListUI {...props} />)

		expect(screen.getByText(/3 500,00/)).toBeDefined()
		expect(screen.getByText(/soit/)).toBeDefined()
		expect(screen.getByText(/23,08/)).toBeDefined()
	})

	it('says outright when a salaried member has no amount entered', async () => {
		const props = {
			...baseProps(),
			members: [
				member({
					isSalaried: true,
					hourlyRateCents: null,
					monthlyCostCents: null,
					effectiveHourlyRateCents: null,
				}),
			],
		}
		await renderWithRouter(<TeamListUI {...props} />)

		expect(screen.getByText(/coût mensuel non renseigné/i)).toBeDefined()
	})

	/** A salary needs a contract to be spread over. */
	it('says outright when the contract cannot divide the salary', async () => {
		const props = {
			...baseProps(),
			members: [
				member({
					isSalaried: true,
					hourlyRateCents: null,
					monthlyCostCents: 350_000,
					effectiveHourlyRateCents: null,
					weeklyContractMinutes: 0,
				}),
			],
		}
		await renderWithRouter(<TeamListUI {...props} />)

		// A link, not just a sentence: the contract is edited on another screen and
		// the row has to say which one.
		const link = screen.getByRole('link', {
			name: /base contractuelle à renseigner/i,
		})
		expect(link.getAttribute('href')).toContain('/work-time')
	})
})

describe('TeamListUI — role assignment (#308)', () => {
	it("renders a member's held roles as badges, resolved against the organization's role list", async () => {
		await renderWithRouter(<TeamListUI {...baseProps()} />, undefined, {
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
		await renderWithRouter(<TeamListUI {...baseProps()} />, undefined, {
			roles: [role({ id: 'role-admin', name: 'Administrateur' })],
			memberRoleIds: {},
		})

		expect(await screen.findByText('Aucun rôle')).toBeDefined()
	})

	it('offers the assign control when the fixture grants MANAGE_ROLES', async () => {
		await renderWithRouter(<TeamListUI {...baseProps()} />, undefined, {
			permissions: ['MANAGE_ROLES'],
			roles: [role({ id: 'role-admin', name: 'Administrateur' })],
		})

		expect(
			await screen.findByRole('button', { name: 'Assigner un rôle' }),
		).toBeDefined()
	})

	it('hides the assign control when the fixture does not grant MANAGE_ROLES', async () => {
		await renderWithRouter(<TeamListUI {...baseProps()} />, undefined, {
			permissions: ['MANAGE_MEMBERS'],
			roles: [role({ id: 'role-admin', name: 'Administrateur' })],
		})

		// Let the roles read resolve, then assert the control never appears —
		// there is nothing else to `findBy` on to synchronize with here.
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

		await renderWithRouter(<TeamListUI {...baseProps()} />, undefined, {
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
		await renderWithRouter(<TeamListUI {...baseProps()} />, undefined, {
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

		// Disabled: clicking it must not open a picker or call anything.
		await user.click(button)
		expect(
			screen.queryByText('Administrateur', { selector: 'button' }),
		).toBeNull()
	})
})

describe('TeamListUI — role unassignment (#401, #402)', () => {
	it('offers a remove control on a held role when the fixture grants MANAGE_ROLES', async () => {
		await renderWithRouter(<TeamListUI {...baseProps()} />, undefined, {
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
		await renderWithRouter(<TeamListUI {...baseProps()} />, undefined, {
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

		await renderWithRouter(<TeamListUI {...baseProps()} />, undefined, {
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
		await renderWithRouter(<TeamListUI {...baseProps()} />, undefined, {
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
		await renderWithRouter(<TeamListUI {...baseProps()} />, undefined, {
			permissions: ['MANAGE_ROLES'],
			roles: [role({ id: 'role-admin', name: 'Administrateur' })],
		})

		await user.click(
			await screen.findByRole('button', { name: 'Assigner un rôle' }),
		)

		expect(screen.queryByPlaceholderText('Rechercher un rôle…')).toBeNull()
	})
})
