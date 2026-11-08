import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import { fullName } from '#/pages/hr/lib/rh/members'
import type { Member, RhState } from '#/pages/hr/lib/rh/types'
import {
	type MemberRealInfo,
	NO_REAL_INFO,
} from '#/pages/hr/ui/rh/members/member-access'
import { MembersTable } from '#/pages/hr/ui/rh/members/members-table'
import { RhProvider } from '#/pages/hr/ui/rh/shared/rh-context'
import { renderWithRouter } from '#/test/render-with-router'

const NO_CONTRACT_ACTIONS = {
	add: async () => {},
	update: async () => {},
	remove: async () => {},
}

const NO_ABSENCE_ACTIONS = {
	create: async () => {},
	approve: async () => {},
	reject: async () => {},
	cancel: async () => {},
}

const NO_ABSENCE_TYPE_ACTIONS = {
	save: async () => {},
	archive: async () => {},
	restore: async () => {},
}

const NO_LEAVE_ACTIONS = {
	saveAccount: async () => {},
	archiveAccount: async () => {},
	restoreAccount: async () => {},
	addAdjustment: async () => {},
	removeAdjustment: async () => {},
}

const NO_DOCUMENT_ACTIONS = {
	add: async () => {},
	remove: async () => {},
}

const NO_PROFILE_ACTIONS = {
	save: async () => {},
	archive: async () => {},
	restore: async () => {},
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

const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

function installFakePermissions(permissions: string[]) {
	const fakeApi = {
		get(path: string, params?: unknown) {
			const queryKey = [
				{ _id: path, path: (params as { path?: unknown })?.path },
			]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === MY_PERMISSIONS_PATH)
							return { data: { permissions }, pagination: null }
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation() {
			return { mutationKey: [], mutationOptions: { mutationKey: [] } }
		},
	}
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi
}

const TODAY = '2026-11-05'
const SEED_MEMBER = INITIAL_RH_STATE.members[0] as Member

function info(overrides: Partial<MemberRealInfo> = {}): MemberRealInfo {
	return { ...NO_REAL_INFO, hourlyRateCents: 1500, ...overrides }
}

function renderTable(
	options: {
		member?: Member
		info?: MemberRealInfo
		hrDataRestricted?: boolean
		permissions?: string[]
	} = {},
) {
	installFakePermissions(options.permissions ?? ['MANAGE_MEMBERS'])
	const member = options.member ?? SEED_MEMBER
	const state: RhState = { ...INITIAL_RH_STATE, members: [member] }
	const handlers = {
		onEdit: vi.fn(),
		onInvite: vi.fn(),
		onDelete: vi.fn(),
	}
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	return renderWithRouter(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					<RhProvider
						value={{
							state,
							update: async () => {},
							profiles: NO_PROFILE_ACTIONS,
							contracts: NO_CONTRACT_ACTIONS,
							documents: NO_DOCUMENT_ACTIONS,
							absences: NO_ABSENCE_ACTIONS,
							absenceTypes: NO_ABSENCE_TYPE_ACTIONS,
							leave: NO_LEAVE_ACTIONS,
							actorId: member.id,
							organizationSlug: 'atelier-bois',
						}}
					>
						<MembersTable
							state={state}
							members={[member]}
							today={TODAY}
							realInfo={new Map([[member.id, options.info ?? info()]])}
							hrDataRestricted={options.hrDataRestricted ?? false}
							renderRoles={() => <span>roles</span>}
							{...handlers}
						/>
					</RhProvider>
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
	).then((result) => ({ ...result, ...handlers, member }))
}

describe('MembersTable — prototype columns', () => {
	it('links the name to the member detail page', async () => {
		await renderTable()

		const link = screen.getByRole('link', { name: fullName(SEED_MEMBER) })
		expect(link.getAttribute('href')).toBe(
			`/o/atelier-bois/rh/membres/${SEED_MEMBER.id}`,
		)
	})

	it('marks an archived member and keeps it listed', async () => {
		await renderTable({ member: { ...SEED_MEMBER, status: 'archived' } })

		expect(screen.getByText('Archivé')).toBeDefined()
	})
})

describe('MembersTable — access column', () => {
	it('shows "Aucun accès" for a seat with no account', async () => {
		await renderTable({ info: info({ access: 'none' }) })

		expect(screen.getByText('Aucun accès')).toBeDefined()
	})

	it('shows "Compte lié" for a seat with a linked account', async () => {
		await renderTable({ info: info({ access: 'linkedAccount' }) })

		expect(screen.getByText('Compte lié')).toBeDefined()
	})
})

describe('MembersTable — rate and contract base', () => {
	it('says "Sans profil RH" when the seat has no profile', async () => {
		await renderTable({
			info: info({ hourlyRateCents: null, weeklyContractMinutes: null }),
		})

		expect(screen.getByText('Non renseigné')).toBeDefined()
		expect(screen.getByText('Sans profil RH')).toBeDefined()
	})

	it('says "Non consultable", never "Sans profil RH", when HR data is restricted', async () => {
		await renderTable({
			hrDataRestricted: true,
			info: info({ hourlyRateCents: null, weeklyContractMinutes: null }),
		})

		expect(screen.getByText('Non consultable')).toBeDefined()
		expect(screen.queryByText('Sans profil RH')).toBeNull()
	})

	it('shows a salaried member on both bases: monthly, and the hourly equivalent', async () => {
		await renderTable({
			info: info({
				isSalaried: true,
				hourlyRateCents: null,
				monthlyCostCents: 350_000,
				effectiveHourlyRateCents: 2_308,
			}),
		})

		expect(screen.getByText(/3 500,00/)).toBeDefined()
		expect(screen.getByText(/soit/)).toBeDefined()
		expect(screen.getByText(/23,08/)).toBeDefined()
	})

	it('says outright when a salaried member has no amount entered', async () => {
		await renderTable({
			info: info({
				isSalaried: true,
				hourlyRateCents: null,
				monthlyCostCents: null,
				effectiveHourlyRateCents: null,
			}),
		})

		expect(screen.getByText(/coût mensuel non renseigné/i)).toBeDefined()
	})

	it('links to the work time page when the contract cannot divide the salary', async () => {
		await renderTable({
			info: info({
				isSalaried: true,
				hourlyRateCents: null,
				monthlyCostCents: 350_000,
				effectiveHourlyRateCents: null,
				weeklyContractMinutes: 0,
			}),
		})

		const link = screen.getByRole('link', {
			name: /base contractuelle à renseigner/i,
		})
		expect(link.getAttribute('href')).toContain('/temps-de-travail')
	})
})

describe('MembersTable — row actions', () => {
	it("exposes a link to the member's work time", async () => {
		const user = userEvent.setup()
		await renderTable()

		await user.click(screen.getByRole('button', { name: 'Actions' }))

		const link = screen.getByRole('menuitem', { name: /Temps de travail/ })
		expect(link.tagName).toBe('A')
		expect(link.getAttribute('href')).toBe(
			`/o/atelier-bois/rh/membres/${SEED_MEMBER.id}/temps-de-travail`,
		)
	})

	it('offers "Inviter" for a seat with no access and calls onInvite with the member', async () => {
		const user = userEvent.setup()
		const { onInvite, member } = await renderTable({
			info: info({ access: 'none' }),
		})

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Inviter/ }))

		expect(onInvite).toHaveBeenCalledWith(member)
	})

	it('hides "Inviter" once a seat is invited or linked', async () => {
		const user = userEvent.setup()
		await renderTable({ info: info({ access: 'invited' }) })

		await user.click(screen.getByRole('button', { name: 'Actions' }))

		expect(screen.queryByRole('menuitem', { name: /Inviter/ })).toBeNull()
	})

	it('hides "Inviter" for an archived member', async () => {
		const user = userEvent.setup()
		await renderTable({ member: { ...SEED_MEMBER, status: 'archived' } })

		await user.click(screen.getByRole('button', { name: 'Actions' }))

		expect(screen.queryByRole('menuitem', { name: /Inviter/ })).toBeNull()
	})

	it('calls onEdit from "Modifier"', async () => {
		const user = userEvent.setup()
		const { onEdit, member } = await renderTable()

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(await screen.findByRole('menuitem', { name: 'Modifier' }))

		expect(onEdit).toHaveBeenCalledWith(member)
	})

	it('does not call onDelete until the confirmation is accepted', async () => {
		const user = userEvent.setup()
		const { onDelete, member } = await renderTable()

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(await screen.findByRole('menuitem', { name: /Supprimer/ }))

		expect(onDelete).not.toHaveBeenCalled()
		expect(
			screen.getByRole('alertdialog', {
				name: new RegExp(`Supprimer ${fullName(member)}`),
			}),
		).toBeDefined()

		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		expect(onDelete).toHaveBeenCalledWith(member)
	})

	it('hides "Modifier" and "Supprimer" without MANAGE_MEMBERS', async () => {
		const user = userEvent.setup()
		await renderTable({ permissions: [] })

		await user.click(screen.getByRole('button', { name: 'Actions' }))

		expect(screen.queryByRole('menuitem', { name: 'Modifier' })).toBeNull()
		expect(screen.queryByRole('menuitem', { name: /Supprimer/ })).toBeNull()
		expect(
			screen.getByRole('menuitem', { name: /Temps de travail/ }),
		).toBeDefined()
	})
})
