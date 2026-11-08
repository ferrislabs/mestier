import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useState } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { TooltipProvider } from '#/components/ui/tooltip'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { ContractRecord } from '#/pages/hr/lib/rh/member-wire'
import type { DayOverrideRecord } from '#/pages/hr/lib/rh/override-wire'
import type { RhState } from '#/pages/hr/lib/rh/types'
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
import {
	DAY_OVERRIDE_PATH,
	DAY_OVERRIDES_PATH,
	overrideRecord,
	PLANNING_FEED_PATH,
} from '#/test/rh-override-fixtures'
import { MemberDetailFeature } from './member-detail-feature'
import { PlanningFeature, type PlanningView } from './planning-feature'

const composed = vi.hoisted(() => ({ latest: null as RhState | null }))

vi.mock('#/pages/hr/lib/rh/compose', async (importOriginal) => {
	const original =
		await importOriginal<typeof import('#/pages/hr/lib/rh/compose')>()
	return {
		...original,
		composeRhState: (
			sources: Parameters<typeof original.composeRhState>[0],
		) => {
			const state = original.composeRhState(sources)
			composed.latest = state
			return state
		},
	}
})

function getRhState(): RhState {
	if (!composed.latest) throw new Error('no state composed yet')
	return composed.latest
}

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

const MEMBERS = [member('u-1', 'Alix', 'Nova'), member('u-2', 'Bao', 'Tran')]

interface Write {
	method: string
	path: string
	params: { path?: Record<string, string>; body?: Record<string, unknown> }
}

interface Read {
	path: string
	query: Record<string, string>
}

let overrides: DayOverrideRecord[]
let contracts: ContractRecord[]
let writes: Write[]
let reads: Read[]
let refuse: { status: number; data: Record<string, unknown> } | null
let feedForbidden: boolean
let permissions: string[]

function within_(row: DayOverrideRecord, query: Record<string, string>) {
	return (
		row.date >= query.from &&
		row.date <= query.to &&
		(!query.member_id || row.member_id === query.member_id)
	)
}

function installFakeApi() {
	const fakeApi = {
		get(path: string, params: unknown) {
			const p = (params ?? {}) as {
				path?: unknown
				query?: Record<string, string>
			}
			const queryKey = [{ _id: path, path: p.path, query: p.query }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === MEMBERS_PATH)
							return { data: MEMBERS, pagination: null }
						if (path === CONTRACTS_PATH)
							return { data: structuredClone(contracts), pagination: null }
						if (
							path === PROFILES_PATH ||
							path === DOCUMENTS_PATH ||
							path === ABSENCES_PATH ||
							path === LEAVE_ADJUSTMENTS_PATH
						)
							return { data: [], pagination: null }
						if (path === ABSENCE_TYPES_PATH)
							return { data: DEFAULT_TYPE_RECORDS, pagination: null }
						if (path === LEAVE_ACCOUNTS_PATH)
							return { data: DEFAULT_ACCOUNT_RECORDS, pagination: null }
						if (path === PERMISSIONS_PATH)
							return { data: { permissions }, pagination: null }
						if (path === PLANNING_FEED_PATH || path === DAY_OVERRIDES_PATH) {
							const query = p.query ?? { from: '', to: '' }
							reads.push({ path, query })
							if (path === PLANNING_FEED_PATH && feedForbidden)
								throw Object.assign(new Error('Forbidden'), { status: 403 })
							const rows = structuredClone(
								overrides.filter((row) => within_(row, query)),
							)
							return path === PLANNING_FEED_PATH
								? {
										data: {
											members: [],
											contracts: [],
											absences: [],
											overrides: rows,
											absence_types: [],
										},
									}
								: { data: rows, pagination: null }
						}
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
					mutationFn: async (params: Write['params']) => {
						writes.push({ method, path, params })
						if (refuse)
							throw Object.assign(new Error('refused'), {
								status: refuse.status,
								data: refuse.data,
							})
						const memberId = params.path?.member_id ?? ''
						const date = params.path?.date ?? ''
						overrides = overrides.filter(
							(row) => row.member_id !== memberId || row.date !== date,
						)
						if (method === 'put' && path === DAY_OVERRIDE_PATH)
							overrides = [
								...overrides,
								{
									...overrideRecord(memberId, date),
									...(params.body as object),
								} as DayOverrideRecord,
							]
						return { data: {} }
					},
				},
			}
		},
	}
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi
}

function providers(children: React.ReactNode) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	return (
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION as never]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION as never}>
					<TooltipProvider>{children}</TooltipProvider>
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>
	)
}

function NavigablePlanning({ start }: { start: string }) {
	const [date, setDate] = useState(start)
	const view: PlanningView = 'week'
	return (
		<PlanningFeature
			view={view}
			date={date}
			onChange={(patch) => patch.date && setDate(patch.date)}
		/>
	)
}

async function renderPlanning(date = '2026-11-05') {
	installFakeApi()
	return renderWithRouter(providers(<NavigablePlanning start={date} />))
}

async function renderMember() {
	installFakeApi()
	return renderWithRouter(
		providers(
			<MemberDetailFeature
				memberId="u-1"
				tab="emploi-du-temps"
				month="2026-11-01"
				onChange={() => undefined}
			/>,
		),
	)
}

function readsOf(path: string) {
	return reads.filter((read) => read.path === path)
}

async function openPlanDialog() {
	const user = userEvent.setup()
	await user.click(
		(await screen.findAllByRole('button', { name: /Planifier des jours/ }))[0],
	)
	const dialog = await screen.findByRole('dialog')
	return { user, dialog }
}

function setDates(dialog: HTMLElement, from: string, to: string) {
	fireEvent.change(within(dialog).getByLabelText(/^Du/), {
		target: { value: from },
	})
	fireEvent.change(within(dialog).getByLabelText(/^Au/), {
		target: { value: to },
	})
}

describe('day overrides and the planning feed bound to the API', () => {
	beforeEach(() => {
		overrides = []
		contracts = []
		writes = []
		reads = []
		refuse = null
		feedForbidden = false
		permissions = ['MANAGE_PLANNING']
		composed.latest = null
	})

	it('spec_rh_5_4 the planning screen reads the feed for its visible period, not the store', async () => {
		overrides = [
			overrideRecord('u-1', '2026-11-04', {
				kind: 'OFF',
				start: null,
				end: null,
				lunch: null,
				note: 'Pont',
			}),
			overrideRecord('u-1', '2026-12-04'),
		]

		await renderPlanning('2026-11-05')

		await waitFor(() =>
			expect(getRhState().overrides.map((row) => row.date)).toEqual([
				'2026-11-04',
			]),
		)
		expect(readsOf(PLANNING_FEED_PATH).map((read) => read.query)).toEqual([
			{ from: '2026-11-02', to: '2026-11-08' },
		])
		expect(readsOf(DAY_OVERRIDES_PATH)).toEqual([])
		expect(getRhState().overrides[0]).toMatchObject({
			memberId: 'u-1',
			kind: 'off',
			note: 'Pont',
		})
	})

	it('spec_rh_5_4 another period queries its own range and keeps the prior one cached', async () => {
		overrides = [
			overrideRecord('u-1', '2026-11-04'),
			overrideRecord('u-2', '2026-11-11'),
		]
		await renderPlanning('2026-11-05')
		await waitFor(() => expect(getRhState().overrides).toHaveLength(1))

		await userEvent.click(
			await screen.findByRole('button', { name: 'Semaine suivante' }),
		)

		await waitFor(() => expect(getRhState().overrides).toHaveLength(2))
		expect(readsOf(PLANNING_FEED_PATH).map((read) => read.query.from)).toEqual([
			'2026-11-02',
			'2026-11-09',
		])
	})

	it('a 403 on the feed leaves the screen without overrides and without an error', async () => {
		feedForbidden = true
		overrides = [overrideRecord('u-1', '2026-11-04')]
		permissions = []

		await renderPlanning('2026-11-05')

		expect(await screen.findByText('Alix Nova')).toBeDefined()
		await waitFor(() => expect(readsOf(PLANNING_FEED_PATH)).toHaveLength(1))
		expect(getRhState().overrides).toEqual([])
		expect(screen.queryByRole('alert')).toBeNull()
	})

	it('the member calendar reads the overrides of its padded month for that member', async () => {
		overrides = [
			overrideRecord('u-1', '2026-11-10'),
			overrideRecord('u-2', '2026-11-10'),
		]

		await renderMember()

		await waitFor(() =>
			expect(getRhState().overrides.map((row) => row.memberId)).toEqual([
				'u-1',
			]),
		)
		expect(
			readsOf(DAY_OVERRIDES_PATH).map((read) => read.query),
		).toContainEqual({
			from: '2026-10-26',
			to: '2026-12-06',
			member_id: 'u-1',
		})
	})

	it('spec_rh_5_1 the plan dialog upserts one override per day through the API', async () => {
		await renderMember()
		const { user, dialog } = await openPlanDialog()
		setDates(dialog, '2026-11-09', '2026-11-10')

		await user.click(within(dialog).getByRole('button', { name: 'Planifier' }))

		await waitFor(() => expect(writes).toHaveLength(2))
		expect(writes.map((write) => write.method)).toEqual(['put', 'put'])
		expect(writes.map((write) => write.params.path)).toEqual([
			{ organization_id: 'org-1', member_id: 'u-1', date: '2026-11-09' },
			{ organization_id: 'org-1', member_id: 'u-1', date: '2026-11-10' },
		])
		expect(writes[0]?.params.body).toMatchObject({
			kind: 'WORK',
			start: '08:00',
			end: '17:00',
		})
		await waitFor(() =>
			expect(
				getRhState()
					.overrides.map((row) => row.date)
					.sort(),
			).toEqual(['2026-11-09', '2026-11-10']),
		)
	})

	it('deleting a planned period deletes each planned day through the API', async () => {
		overrides = [
			overrideRecord('u-1', '2026-11-09'),
			overrideRecord('u-1', '2026-11-10'),
			overrideRecord('u-1', '2026-11-12'),
		]
		await renderMember()
		await waitFor(() => expect(getRhState().overrides).toHaveLength(3))
		const { user, dialog } = await openPlanDialog()
		setDates(dialog, '2026-11-09', '2026-11-10')

		await user.click(
			await within(dialog).findByRole('button', {
				name: 'Effacer la planification de cette période',
			}),
		)
		await user.click(
			within(dialog).getByRole('button', { name: 'Confirmer l’effacement' }),
		)

		await waitFor(() => expect(writes).toHaveLength(2))
		expect(writes.map((write) => write.method)).toEqual(['delete', 'delete'])
		expect(writes.map((write) => write.params.path?.date)).toEqual([
			'2026-11-09',
			'2026-11-10',
		])
		await waitFor(() =>
			expect(getRhState().overrides.map((row) => row.date)).toEqual([
				'2026-11-12',
			]),
		)
	})

	it('a refusal of the API shows inside the plan dialog and keeps it open', async () => {
		refuse = {
			status: 422,
			data: { message: 'end', details: { field: 'end' } },
		}
		await renderMember()
		const { user, dialog } = await openPlanDialog()
		setDates(dialog, '2026-11-09', '2026-11-09')

		await user.click(within(dialog).getByRole('button', { name: 'Planifier' }))

		expect((await within(dialog).findByRole('alert')).textContent).toMatch(
			/heure de fin/,
		)
		expect(screen.getByRole('dialog')).toBeDefined()
	})

	it('spec_rh_5_1 the day popover puts then clears the override of a day through the API', async () => {
		const day = { start: '08:00', end: '17:00' }
		contracts = [
			{
				id: 'c-1',
				organization_id: 'org-1',
				member_id: 'u-1',
				type: 'CDI',
				start_date: '2026-01-01',
				end_date: null,
				trial_end_date: null,
				schedule: {
					kind: 'weekly',
					week: [day, day, day, day, day, null, null],
				},
			},
		]
		overrides = [overrideRecord('u-1', '2026-11-04')]
		await renderPlanning('2026-11-05')
		await waitFor(() => expect(getRhState().overrides).toHaveLength(1))
		const user = userEvent.setup()

		await user.click(
			await screen.findByRole('button', {
				name: /Alix Nova, mercredi 4 novembre/,
			}),
		)
		await user.click(await screen.findByRole('radio', { name: 'Repos' }))
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]).toMatchObject({
			method: 'put',
			params: {
				path: { member_id: 'u-1', date: '2026-11-04' },
				body: { kind: 'OFF' },
			},
		})
		await waitFor(() => expect(getRhState().overrides[0]?.kind).toBe('off'))

		await user.click(
			await screen.findByRole('button', {
				name: /Alix Nova, mercredi 4 novembre/,
			}),
		)
		await user.click(
			await screen.findByRole('button', { name: /Rétablir la semaine type/ }),
		)

		await waitFor(() => expect(writes).toHaveLength(2))
		expect(writes[1]).toMatchObject({
			method: 'delete',
			params: { path: { member_id: 'u-1', date: '2026-11-04' } },
		})
		await waitFor(() => expect(getRhState().overrides).toEqual([]))
	})

	it('without MANAGE_PLANNING the plan control is absent', async () => {
		permissions = []

		await renderMember()

		expect(
			await screen.findByRole('heading', { name: 'Alix Nova' }),
		).toBeDefined()
		expect(
			screen.queryByRole('button', { name: /Planifier des jours/ }),
		).toBeNull()
	})
})
