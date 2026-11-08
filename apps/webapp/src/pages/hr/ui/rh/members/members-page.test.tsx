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
import { DEFAULT_FILTERS } from '#/pages/hr/lib/rh/members-filter'
import type { Member, RhState } from '#/pages/hr/lib/rh/types'
import {
	type MemberRealInfo,
	NO_REAL_INFO,
} from '#/pages/hr/ui/rh/members/member-access'
import {
	MembersPage,
	type MembersPageProps,
} from '#/pages/hr/ui/rh/members/members-page'
import { RhProvider } from '#/pages/hr/ui/rh/shared/rh-context'
import { renderWithRouter } from '#/test/render-with-router'

const NO_CONTRACT_ACTIONS = {
	add: async () => {},
	update: async () => {},
	remove: async () => {},
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

function props(overrides: Partial<MembersPageProps> = {}): MembersPageProps {
	const state: RhState = { ...INITIAL_RH_STATE, members: [SEED_MEMBER] }
	return {
		state,
		members: [SEED_MEMBER],
		today: TODAY,
		filters: DEFAULT_FILTERS,
		onFiltersChange: vi.fn(),
		error: null,
		hrDataRestricted: false,
		realInfo: new Map<string, MemberRealInfo>([[SEED_MEMBER.id, NO_REAL_INFO]]),
		pendingInvitations: [],
		revokingInvitationId: null,
		renderRoles: () => <span>roles</span>,
		onCreate: vi.fn(),
		onEdit: vi.fn(),
		onInvite: vi.fn(),
		onDelete: vi.fn(),
		onRevokeInvitation: vi.fn(),
		dialogs: null,
		...overrides,
	}
}

function render(page: MembersPageProps, permissions = ['MANAGE_MEMBERS']) {
	installFakePermissions(permissions)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false } },
	})
	return renderWithRouter(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					<RhProvider
						value={{
							state: page.state,
							update: async () => {},
							profiles: NO_PROFILE_ACTIONS,
							contracts: NO_CONTRACT_ACTIONS,
							documents: NO_DOCUMENT_ACTIONS,
							actorId: SEED_MEMBER.id,
							organizationSlug: 'atelier-bois',
						}}
					>
						<MembersPage {...page} />
					</RhProvider>
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
	)
}

describe('MembersPage — header and notices', () => {
	it('never renders the member id', async () => {
		await render(props())

		expect(screen.queryByText(new RegExp(SEED_MEMBER.id))).toBeNull()
	})

	it('asks the feature to open the create modal from "Nouveau membre"', async () => {
		const user = userEvent.setup()
		const page = props()
		await render(page)

		await user.click(
			await screen.findByRole('button', { name: 'Nouveau membre' }),
		)

		expect(page.onCreate).toHaveBeenCalled()
	})

	it('hides "Nouveau membre" without MANAGE_MEMBERS', async () => {
		await render(props(), [])

		expect(screen.queryByRole('button', { name: 'Nouveau membre' })).toBeNull()
	})

	it('shows a neutral notice instead of the red error banner when HR data is restricted', async () => {
		await render(props({ hrDataRestricted: true }))

		expect(
			screen.getByText(/n’avez pas la permission de consulter/),
		).toBeDefined()
	})

	it('still shows the real error banner when there is an actual failure', async () => {
		await render(
			props({
				error: 'Impossible de contacter le serveur',
				hrDataRestricted: true,
			}),
		)

		expect(screen.getByText('Impossible de contacter le serveur')).toBeDefined()
		expect(
			screen.queryByText(/n’avez pas la permission de consulter/),
		).toBeNull()
	})

	it('offers to reset the filters when nothing matches', async () => {
		const user = userEvent.setup()
		const page = props({
			members: [],
			filters: { ...DEFAULT_FILTERS, query: 'zzz' },
		})
		await render(page)

		await user.click(
			screen.getByRole('button', { name: 'Réinitialiser les filtres' }),
		)

		expect(page.onFiltersChange).toHaveBeenCalledWith(DEFAULT_FILTERS)
	})

	it('lists a pending invitation and revokes it on demand', async () => {
		const user = userEvent.setup()
		const page = props({
			pendingInvitations: [
				{
					id: 'invitation-1',
					memberName: 'Alix Nova',
					expiresAt: '2026-08-20',
				},
			],
		})
		await render(page)

		expect(screen.getByText('Invitations en attente (1)')).toBeDefined()
		await user.click(screen.getByRole('button', { name: /Révoquer/ }))

		expect(page.onRevokeInvitation).toHaveBeenCalledWith('invitation-1')
	})

	it('renders no invitation panel when none is pending', async () => {
		await render(props())

		expect(screen.queryByText(/Invitations en attente/)).toBeNull()
	})
})
