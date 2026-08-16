import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { WorkTimeOverviewFeature } from '#/pages/hr/feature/work-time-overview-feature'
import { renderWithRouter } from '#/test/render-with-router'

const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const EMPLOYEE_PROFILES_PATH =
	'/api/v1/organizations/{organization_id}/employee-profiles'
const ABSENCES_PATH = '/api/v1/organizations/{organization_id}/absences'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	owner_id: 'user-1',
	slug: 'atelier-bois',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

interface FakeMember {
	id: string
	organization_id: string
	last_name: string
	first_name: string | null
	display_name: string
	account: null
	joined_at: string | null
	created_at: string
}

interface FakeEmployeeProfile {
	id: string
	organization_id: string
	member_id: string
	hourly_rate_cents: number | null
	weekly_contract_minutes: number
	created_at: string
	updated_at: string
}

interface FakeAbsence {
	id: string
	organization_id: string
	member_id: string
	kind: 'LEAVE' | 'SICK' | 'UNAVAILABLE'
	all_day: boolean
	starts_at: string
	ends_at: string
	note: string | null
	created_at: string
	updated_at: string
}

interface FakeApiHandlers {
	members?: FakeMember[]
	employeeProfiles?: FakeEmployeeProfile[]
	absences?: FakeAbsence[]
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
	const members = handlers.members ?? []
	const employeeProfiles = handlers.employeeProfiles ?? []
	const absences = handlers.absences ?? []

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
						if (path === MEMBERS_PATH) {
							return { data: members, pagination: null }
						}
						if (path === EMPLOYEE_PROFILES_PATH) {
							return { data: employeeProfiles, pagination: null }
						}
						if (path === ABSENCES_PATH) {
							return { data: absences, pagination: null }
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

function member(overrides: Partial<FakeMember> = {}): FakeMember {
	return {
		id: 'member-1',
		organization_id: 'org-1',
		last_name: 'Nova',
		first_name: 'Alix',
		display_name: 'Nova Alix',
		account: null,
		joined_at: null,
		created_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

function profile(
	overrides: Partial<FakeEmployeeProfile> = {},
): FakeEmployeeProfile {
	return {
		id: 'employee-1',
		organization_id: 'org-1',
		member_id: 'member-1',
		hourly_rate_cents: 1500,
		weekly_contract_minutes: 2100,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
		...overrides,
	}
}

function absence(overrides: Partial<FakeAbsence> = {}): FakeAbsence {
	return {
		id: 'absence-1',
		organization_id: 'org-1',
		member_id: 'member-1',
		kind: 'LEAVE',
		all_day: true,
		starts_at: '2026-08-20T00:00:00Z',
		ends_at: '2026-08-21T00:00:00Z',
		note: null,
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}

function renderFeature(handlers: FakeApiHandlers = {}) {
	installFakeTanstackApi(handlers)
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
			<WorkTimeOverviewFeature />
		</Providers>,
	)
}

describe('WorkTimeOverviewFeature — weekly contract duration', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('shows the formatted weekly contract base for a seat with a profile', async () => {
		await renderFeature({
			members: [member()],
			employeeProfiles: [profile({ weekly_contract_minutes: 2100 })],
		})

		expect(await screen.findByText('Nova Alix')).toBeDefined()
		expect(screen.getByText('35h00')).toBeDefined()
	})

	it('marks a seat without an employee profile instead of showing a duration', async () => {
		await renderFeature({ members: [member()], employeeProfiles: [] })

		expect(await screen.findByText('Nova Alix')).toBeDefined()
		expect(screen.getByText('Sans profil RH')).toBeDefined()
	})
})

describe('WorkTimeOverviewFeature — next upcoming absence', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('shows the soonest absence that starts today or later, ignoring past ones', async () => {
		await renderFeature({
			members: [member()],
			employeeProfiles: [profile()],
			absences: [
				absence({
					id: 'past',
					starts_at: '2000-01-01T00:00:00Z',
					ends_at: '2000-01-02T00:00:00Z',
				}),
				absence({
					id: 'soonest-future',
					starts_at: '2100-08-20T00:00:00Z',
					ends_at: '2100-08-21T00:00:00Z',
				}),
				absence({
					id: 'later-future',
					starts_at: '2100-09-01T00:00:00Z',
					ends_at: '2100-09-02T00:00:00Z',
				}),
			],
		})

		expect(await screen.findByText('Nova Alix')).toBeDefined()
		expect(screen.getByText('20/08/2100')).toBeDefined()
		expect(screen.queryByText('01/09/2100')).toBeNull()
		expect(screen.queryByText('01/01/2000')).toBeNull()
	})

	it('shows a placeholder when the member has no upcoming absence', async () => {
		await renderFeature({
			members: [member()],
			employeeProfiles: [profile()],
			absences: [
				absence({
					starts_at: '2000-01-01T00:00:00Z',
					ends_at: '2000-01-02T00:00:00Z',
				}),
			],
		})

		expect(await screen.findByText('Nova Alix')).toBeDefined()
		expect(screen.getByText('—')).toBeDefined()
	})

	it('never mixes up absences between two different members', async () => {
		await renderFeature({
			members: [
				member(),
				member({ id: 'member-2', display_name: 'Bernard Léa' }),
			],
			employeeProfiles: [
				profile(),
				profile({ id: 'employee-2', member_id: 'member-2' }),
			],
			absences: [
				absence({
					id: 'for-member-2',
					member_id: 'member-2',
					starts_at: '2100-08-18T00:00:00Z',
					ends_at: '2100-08-19T00:00:00Z',
				}),
			],
		})

		expect(await screen.findByText('Nova Alix')).toBeDefined()
		expect(screen.getByText('Bernard Léa')).toBeDefined()

		const rows = screen.getAllByRole('row')
		const novaRow = rows.find((row) => row.textContent?.includes('Nova Alix'))
		const bernardRow = rows.find((row) =>
			row.textContent?.includes('Bernard Léa'),
		)
		expect(novaRow?.textContent).toContain('—')
		expect(bernardRow?.textContent).toContain('18/08/2100')
	})
})
