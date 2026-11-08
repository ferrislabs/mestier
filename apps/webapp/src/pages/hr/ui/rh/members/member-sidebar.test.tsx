import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { INITIAL_RH_STATE } from '#/pages/hr/feature/mocks/rh-seed'
import type { Member } from '#/pages/hr/lib/rh/types'
import { MemberSidebar } from '#/pages/hr/ui/rh/members/member-sidebar'
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

const NO_OVERRIDE_ACTIONS = {
	put: async () => {},
	clear: async () => {},
	plan: async () => {},
	clearRange: async () => {},
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

const MEMBER = INITIAL_RH_STATE.members[0] as Member

function renderSidebar(permissions: string[]) {
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
							state: INITIAL_RH_STATE,
							update: async () => {},
							profiles: NO_PROFILE_ACTIONS,
							contracts: NO_CONTRACT_ACTIONS,
							documents: NO_DOCUMENT_ACTIONS,
							absences: NO_ABSENCE_ACTIONS,
							absenceTypes: NO_ABSENCE_TYPE_ACTIONS,
							leave: NO_LEAVE_ACTIONS,
							overrides: NO_OVERRIDE_ACTIONS,
							organizationId: 'org-1',
							actorId: MEMBER.id,
							organizationSlug: 'atelier-bois',
						}}
					>
						<MemberSidebar
							state={INITIAL_RH_STATE}
							member={MEMBER}
							today="2026-11-05"
							tab="identite"
						/>
					</RhProvider>
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
	)
}

describe('MemberSidebar — real-only links', () => {
	it('links the work time page of the member', async () => {
		await renderSidebar([])

		const link = screen.getByRole('link', { name: /Temps de travail/ })
		expect(link.getAttribute('href')).toBe(
			`/o/atelier-bois/rh/membres/${MEMBER.id}/temps-de-travail`,
		)
	})

	it('links the cost history section for MANAGE_MEMBERS', async () => {
		await renderSidebar(['MANAGE_MEMBERS'])

		const link = await screen.findByRole('link', { name: /Historique du coût/ })
		expect(link.getAttribute('href')).toBe(
			`/o/atelier-bois/rh/membres/${MEMBER.id}/temps-de-travail#historique-cout`,
		)
	})

	it('hides the cost history link without MANAGE_MEMBERS', async () => {
		await renderSidebar(['MANAGE_PLANNING'])

		await screen.findByRole('link', { name: /Temps de travail/ })
		expect(
			screen.queryByRole('link', { name: /Historique du coût/ }),
		).toBeNull()
	})
})
