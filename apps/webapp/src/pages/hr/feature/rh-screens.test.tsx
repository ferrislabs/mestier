import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, within } from '@testing-library/react'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it } from 'vitest'
import { TooltipProvider } from '#/components/ui/tooltip'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { renderWithRouter } from '#/test/render-with-router'
import {
	ABSENCE_TYPES_PATH,
	ABSENCES_PATH,
	absenceRecord,
	DEFAULT_TYPE_RECORDS,
} from '#/test/rh-absence-fixtures'
import { MY_MEMBER_PATH, myMemberResponse } from '#/test/rh-current-member'
import { EXTRAS_PATH, SURCHARGES_PATH } from '#/test/rh-extra-fixtures'
import {
	DEFAULT_ACCOUNT_RECORDS,
	LEAVE_ACCOUNTS_PATH,
	LEAVE_ADJUSTMENTS_PATH,
} from '#/test/rh-leave-fixtures'
import { AbsencesFeature } from './absences-feature'
import { ExtraFeature } from './extra-feature'
import { MemberDetailFeature } from './member-detail-feature'
import { PlanningFeature } from './planning-feature'
import { RightsFeature } from './rights-feature'

const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const PROFILES_PATH = '/api/v1/organizations/{organization_id}/member-profiles'
const CONTRACTS_PATH =
	'/api/v1/organizations/{organization_id}/member-contracts'
const DOCUMENTS_PATH =
	'/api/v1/organizations/{organization_id}/member-documents'
const PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
}

function member(id: string, firstName: string, lastName: string) {
	return {
		id,
		organization_id: 'org-1',
		last_name: lastName,
		first_name: firstName,
		display_name: `${lastName} ${firstName}`,
		account: null,
		joined_at: null,
		created_at: '2026-01-01T00:00:00Z',
	}
}

const MEMBERS = [
	member('u-1', 'Alix', 'Nova'),
	member('u-2', 'Bao', 'Tran'),
	member('u-3', 'Cleo', 'Dupont'),
]

function installFakeApi(absences: unknown[] = []) {
	const fakeApi = {
		get(path: string, params: unknown) {
			const p = (params ?? {}) as { path?: unknown; query?: unknown }
			const queryKey = [{ _id: path, path: p.path, query: p.query }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === MEMBERS_PATH)
							return { data: MEMBERS, pagination: null }
						if (path === ABSENCE_TYPES_PATH)
							return { data: DEFAULT_TYPE_RECORDS, pagination: null }
						if (path === ABSENCES_PATH)
							return { data: absences, pagination: null }
						if (
							path === PROFILES_PATH ||
							path === CONTRACTS_PATH ||
							path === DOCUMENTS_PATH
						)
							return { data: [], pagination: null }
						if (path === MY_MEMBER_PATH) return myMemberResponse()
						if (path === PERMISSIONS_PATH)
							return {
								data: {
									permissions: [
										'MANAGE_REFERENCE',
										'MANAGE_MEMBERS',
										'MANAGE_PLANNING',
									],
								},
								pagination: null,
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
					mutationFn: async () => {
						throw new Error(`unmocked mutation ${method} ${path}`)
					},
				},
			}
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi
}

function renderScreen(ui: ReactNode, absences: unknown[] = []) {
	installFakeApi(absences)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})

	return renderWithRouter(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION as never]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION as never}>
					<TooltipProvider>{ui}</TooltipProvider>
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
	)
}

describe('RH screens over the real members', () => {
	beforeEach(() => {})

	it('rights: lists the real members in the balances table', async () => {
		await renderScreen(
			<RightsFeature accountId="cp" onAccountChange={() => undefined} />,
		)

		expect(
			await screen.findByRole('heading', { name: 'Droits et RTT' }),
		).toBeDefined()
		expect(await screen.findAllByText('Alix Nova')).not.toHaveLength(0)
		expect(screen.queryByText('Baptiste Parmantier')).toBeNull()
	})

	it('extra: shows the pending declarations header', async () => {
		await renderScreen(
			<ExtraFeature
				tab="declarations"
				onTabChange={() => undefined}
				status="all"
				memberId="all"
				onFiltersChange={() => undefined}
				creating={false}
				onCreatingChange={() => undefined}
			/>,
		)

		expect(
			await screen.findByRole('heading', { name: 'Heures extra' }),
		).toBeDefined()
		expect(
			screen.getAllByRole('button', { name: /Déclarer des heures/ }).length,
		).toBeGreaterThan(0)
	})

	it('planning: renders the grid for the real members', async () => {
		await renderScreen(
			<PlanningFeature
				view="week"
				date="2026-11-05"
				onChange={() => undefined}
			/>,
		)

		expect(await screen.findByText('Alix Nova')).toBeDefined()
		expect(screen.getByText('Semaine')).toBeDefined()
	})

	it('member detail: opens the real member and its identity rubrique', async () => {
		await renderScreen(
			<MemberDetailFeature
				memberId="u-2"
				tab="identite"
				month={null}
				onChange={() => undefined}
			/>,
		)

		expect(
			await screen.findByRole('heading', { name: 'Bao Tran' }),
		).toBeDefined()
		expect(
			screen.getByRole('navigation', { name: 'Rubriques du membre' }),
		).toBeDefined()
	})

	it('member detail: says so when the member does not exist', async () => {
		await renderScreen(
			<MemberDetailFeature
				memberId="inconnu"
				tab="identite"
				month={null}
				onChange={() => undefined}
			/>,
		)

		expect(await screen.findByText('Ce membre est introuvable.')).toBeDefined()
	})

	it('absences: lists the API absences with their type', async () => {
		await renderScreen(
			<AbsencesFeature
				tab="demandes"
				onTabChange={() => undefined}
				historyStatus="all"
				historyMemberId="all"
				onHistoryFiltersChange={() => undefined}
				creating={false}
				onCreatingChange={() => undefined}
			/>,
			[
				absenceRecord('abs-real-1', 'u-3', 't-maladie', {
					kind: 'SICK',
					status: 'APPROVED',
					start_date: '2026-11-09',
					end_date: '2026-11-10',
					note: 'Grippe',
				}),
			],
		)

		const history = await screen.findByRole('region', { name: 'Historique' })
		expect(await within(history).findAllByText('Cleo Dupont')).not.toHaveLength(
			0,
		)
		expect(
			within(history).getAllByText('Arrêt maladie').length,
		).toBeGreaterThan(0)
	})
})
