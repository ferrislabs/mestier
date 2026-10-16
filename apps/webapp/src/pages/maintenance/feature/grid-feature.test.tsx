import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { PERMISSION_CATALOG } from '#/lib/permission-catalog'
import { GridFeature } from '#/pages/maintenance/feature/grid-feature'
import { isoWeekOf } from '#/pages/maintenance/lib/weeks'
import type { Grid, GridContract, GridLine } from '#/pages/maintenance/types'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

const GRID_PATH = '/api/v1/organizations/{organization_id}/maintenance/grid'
const RECORD_PATH =
	'/api/v1/organizations/{organization_id}/maintenance/lines/{line_id}/interventions'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Treyve Paysages',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	slug: 'treyve',
	field_clock_enabled: false,
	vat_on_debits: false,
	mcp_enabled: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

/** A Thursday of S43, the day the recap is read. */
const TODAY = '2026-10-22'

function addDays(iso: string, days: number): string {
	const [year, month, day] = iso.split('-').map(Number)
	return new Date(Date.UTC(year, month - 1, day + days))
		.toISOString()
		.slice(0, 10)
}

/**
 * Stands in for the backend's due date, only as far as this scenario needs:
 * a 14-day rolling cadence is due 14 days after the last validated visit. The
 * screen itself never does this sum, which is the whole point of the test.
 */
function fakeDueFor(line: GridLine): GridLine['due'] {
	const validated = line.visits
		.filter((visit) => visit.status === 'VALIDATED')
		.map((visit) => visit.done_on)
		.sort()
	const last = validated.at(-1)
	if (!last) return { status: 'THIS_WEEK', on: '2026-03-23' }
	const on = addDays(last, 14)
	const dueWeek = isoWeekOf(on).week
	const thisWeek = isoWeekOf(TODAY).week
	const status =
		dueWeek < thisWeek
			? 'OVERDUE'
			: dueWeek === thisWeek
				? 'THIS_WEEK'
				: dueWeek === thisWeek + 1
					? 'NEXT_WEEK'
					: 'LATER'

	return { status, on }
}

function contract(lines: GridLine[]): GridContract {
	return {
		id: 'contract-gaume',
		reference: 'C25029',
		customer_id: 'customer-gaume',
		customer_context_id: 'context-gaume',
		site: {
			customer_name: 'GAUME',
			site_label: 'Jardin',
			city: 'Vichy',
		},
		season: { year: 2026, first_week: 40, last_week: 46 },
		status: 'ACTIVE',
		lines,
	}
}

function tonte(): GridLine {
	const line: GridLine = {
		id: 'line-tonte',
		label: 'Tonte',
		cadence: { kind: 'ROLLING', every_days: 14 },
		visits: [
			{
				id: 'visit-1',
				done_on: '2026-10-07',
				week: 41,
				status: 'VALIDATED',
			},
		],
		due: { status: 'THIS_WEEK' },
	}
	return { ...line, due: fakeDueFor(line) }
}

function taille(): GridLine {
	return {
		id: 'line-taille',
		label: 'Taille',
		cadence: {
			kind: 'WINDOWS',
			windows: [{ from_week: 44, to_week: 45 }],
		},
		visits: [],
		due: { status: 'LATER', on: '2026-10-26' },
	}
}

interface RecordedCall {
	lineId: string
	doneOn: string
}

function installFakeApi(grid: { current: Grid }) {
	const recorded: RecordedCall[] = []
	let failWith: Error | null = null

	const fakeApi = {
		get(path: string, params: unknown) {
			const { path: pathParams, query } = (params ?? {}) as {
				path?: unknown
				query?: unknown
			}
			const queryKey = [{ _id: path, path: pathParams, query }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === GRID_PATH) {
							return { data: grid.current, pagination: null }
						}
						if (path.endsWith('/members/me/permissions')) {
							return {
								data: {
									permissions: PERMISSION_CATALOG.map((entry) => entry.name),
								},
								pagination: null,
							}
						}
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: unknown) => {
						if (path !== RECORD_PATH) throw new Error(`unmocked ${path}`)
						const { path: ids, body } = params as {
							path: { line_id: string }
							body: { done_on: string }
						}
						if (failWith) throw failWith
						recorded.push({ lineId: ids.line_id, doneOn: body.done_on })
						grid.current = {
							...grid.current,
							contracts: grid.current.contracts.map((entry) => ({
								...entry,
								lines: entry.lines.map((line) => {
									if (line.id !== ids.line_id) return line
									const next: GridLine = {
										...line,
										visits: [
											...line.visits,
											{
												id: 'visit-new',
												done_on: body.done_on,
												week: isoWeekOf(body.done_on).week,
												status: 'VALIDATED',
											},
										],
									}
									return { ...next, due: fakeDueFor(next) }
								}),
							})),
						}
						return { data: {}, pagination: null }
					},
				},
			}
		},
	}
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return {
		recorded,
		failNextWith(error: Error) {
			failWith = error
		},
	}
}

function renderGrid(initial: Grid) {
	const grid = { current: initial }
	const api = installFakeApi(grid)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCacheForOrganization(
		queryClient,
		ORGANIZATION.id,
		PERMISSION_CATALOG.map((entry) => entry.name),
	)

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

	render(
		<Providers>
			<GridFeature season={2026} onSeasonChange={() => {}} today={TODAY} />
		</Providers>,
	)

	return api
}

function cellOf(label: RegExp, week: number): HTMLElement {
	const row = screen.getByRole('row', { name: label })
	return within(row).getByRole('cell', { name: new RegExp(`^S${week}\\b`) })
}

function stateOf(label: RegExp, week: number): string | null {
	return cellOf(label, week).getAttribute('data-state')
}

describe('GridFeature', () => {
	it('@spec-ent-13 records a visit by clicking a cell, and the next due date moves', async () => {
		const user = userEvent.setup()
		const api = renderGrid({
			year: 2026,
			contracts: [contract([tonte(), taille()])],
		})

		await screen.findByText('C25029 - GAUME - Jardin')

		// The sheet opens with the last visit validated in S41 and the next
		// one due in S43 (7 October + 14 days), as the backend computed.
		expect(stateOf(/Tonte/, 41)).toBe('validated')
		expect(cellOf(/Tonte/, 41).textContent).toBe('7')
		expect(stateOf(/Tonte/, 43)).toBe('due')

		await user.click(screen.getByRole('button', { name: /Tonte.*GAUME.*S43/ }))
		const dialog = await screen.findByRole('dialog')
		await user.type(within(dialog).getByLabelText('Jour du mois'), '21')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() => expect(stateOf(/Tonte/, 43)).toBe('validated'))
		expect(cellOf(/Tonte/, 43).textContent).toBe('21')
		expect(stateOf(/Tonte/, 45)).toBe('due')
		expect(api.recorded).toEqual([
			{ lineId: 'line-tonte', doneOn: '2026-10-21' },
		])
		expect(screen.queryByRole('dialog')).toBeNull()
	})

	it('shows the legend of the five states', async () => {
		renderGrid({ year: 2026, contracts: [contract([tonte()])] })
		await screen.findByText('C25029 - GAUME - Jardin')

		const legend = screen.getByRole('list', { name: 'Légende' })
		for (const label of [
			'Validée',
			'Proposée',
			'Planifiée',
			'Échéance',
			'En retard',
		]) {
			expect(within(legend).getByText(label)).toBeTruthy()
		}
	})

	it('does not offer to record a visit where one is already recorded, or outside the season', async () => {
		renderGrid({
			year: 2026,
			contracts: [
				contract([tonte()]),
				{
					...contract([taille()]),
					id: 'contract-short',
					reference: 'C25030',
					season: { year: 2026, first_week: 42, last_week: 46 },
				},
			],
		})
		await screen.findByText('C25029 - GAUME - Jardin')

		expect(screen.queryByRole('button', { name: /Tonte.*S41/ })).toBeNull()
		expect(stateOf(/Taille/, 40)).toBe('out-of-season')
		expect(screen.queryByRole('button', { name: /Taille.*S40/ })).toBeNull()
		expect(screen.getByRole('button', { name: /Taille.*S42/ })).toBeTruthy()
	})

	it('refuses a day the week does not contain, without calling the API', async () => {
		const user = userEvent.setup()
		const api = renderGrid({ year: 2026, contracts: [contract([tonte()])] })
		await screen.findByText('C25029 - GAUME - Jardin')

		await user.click(screen.getByRole('button', { name: /Tonte.*S43/ }))
		const dialog = await screen.findByRole('dialog')
		await user.type(within(dialog).getByLabelText('Jour du mois'), '5')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		expect(
			await within(dialog).findByText("Le 5 n'est pas dans la semaine S43."),
		).toBeTruthy()
		expect(api.recorded).toEqual([])
	})

	it("shows the API's refusal and keeps the dialog open", async () => {
		const user = userEvent.setup()
		const api = renderGrid({ year: 2026, contracts: [contract([tonte()])] })
		api.failNextWith(new Error('Intervention dans le futur'))
		await screen.findByText('C25029 - GAUME - Jardin')

		await user.click(screen.getByRole('button', { name: /Tonte.*S44/ }))
		const dialog = await screen.findByRole('dialog')
		await user.type(within(dialog).getByLabelText('Jour du mois'), '28')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		expect(
			await within(dialog).findByText('Intervention dans le futur'),
		).toBeTruthy()
		expect(screen.getByRole('dialog')).toBeTruthy()
	})

	it('says so when there is no contract for the season', async () => {
		renderGrid({ year: 2026, contracts: [] })

		expect(
			await screen.findByText('Aucun contrat pour la saison 2026.'),
		).toBeTruthy()
	})

	it('does not show a draft contract: it is reviewed in the Contrats tab', async () => {
		renderGrid({
			year: 2026,
			contracts: [
				contract([tonte()]),
				{
					...contract([taille()]),
					id: 'contract-draft',
					reference: 'C99999',
					status: 'DRAFT',
					site: { customer_name: 'Mairie', site_label: 'Parc', city: 'Cusset' },
				},
			],
		})

		await screen.findByText('C25029 - GAUME - Jardin')

		expect(screen.queryByText(/C99999/)).toBeNull()
		expect(screen.queryByRole('row', { name: /Taille/ })).toBeNull()
	})
})
