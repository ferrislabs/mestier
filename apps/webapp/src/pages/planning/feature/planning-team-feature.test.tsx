import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import {
	PlanningTeamFeature,
	type PlanningTeamFeatureProps,
} from '#/pages/planning/feature/planning-team-feature'
import type { PlanningResponse } from '#/pages/planning/types'

// jsdom has no ResizeObserver, which Radix primitives (e.g. `Tabs`) probe
// defensively. Stubbed locally — this workstream doesn't own
// `vitest.setup.ts`.
class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
const globalAny = globalThis as any
globalAny.ResizeObserver ??= ResizeObserverStub

const PLANNING_PATH = '/api/v1/organizations/{organization_id}/planning'
const AVAILABILITY_PATH =
	'/api/v1/organizations/{organization_id}/planning/availability'
const TASK_PATH = '/api/v1/organizations/{organization_id}/tasks/{task_id}'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	owner_id: 'user-1',
	slug: 'atelier-bois',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const RESOURCE_EMPLOYEE_1 = {
	resource_id: 'employee:employee-1',
	kind: 'employee' as const,
	employee_id: 'employee-1',
	user_id: null,
	display_name: 'Alix Martin',
	hourly_rate_cents: 1500,
	weekly_contract_minutes: 2100,
}

const RESOURCE_EMPLOYEE_2 = {
	resource_id: 'employee:employee-2',
	kind: 'employee' as const,
	employee_id: 'employee-2',
	user_id: null,
	display_name: 'Marie Leroy',
	hourly_rate_cents: 1600,
	weekly_contract_minutes: 2100,
}

const RESOURCE_MEMBER = {
	resource_id: 'member:user-9',
	kind: 'member' as const,
	employee_id: null,
	user_id: 'user-9',
	display_name: 'Jules Petit',
	hourly_rate_cents: null,
	weekly_contract_minutes: 0,
}

const TASK_ENTRY = {
	kind: 'task' as const,
	labels: [],
	title: 'Tâche',
	blocks_availability: true,
	child_count: 0,
	id: 'wo-1',
	starts_at: '2026-08-03T08:00:00+02:00',
	ends_at: '2026-08-03T10:00:00+02:00',
	all_day: false,
	status: 'PLANNED' as const,
	employee_ids: ['employee-1'],
	customer_name: 'Client Dupont',
	context_label: 'Chantier toiture',
}

const ABSENCE_ENTRY = {
	kind: 'absence' as const,
	id: 'ab-1',
	starts_at: '2026-08-03T00:00:00+02:00',
	ends_at: '2026-08-04T00:00:00+02:00',
	all_day: true,
	absence_kind: 'LEAVE' as const,
	employee_id: 'employee-1',
	note: 'Vacances',
}

function planningResponse(
	overrides: Partial<PlanningResponse> = {},
): PlanningResponse {
	return {
		timezone: 'Europe/Paris',
		resources: [RESOURCE_EMPLOYEE_1],
		entries: [],
		work_time: [],
		...overrides,
	}
}

/** jsdom has no `DataTransfer`; a plain object backed by a `Map` is enough for `setData`/`getData`. */
function fakeDataTransfer() {
	const store = new Map<string, string>()
	return {
		setData: (format: string, data: string) => store.set(format, data),
		getData: (format: string) => store.get(format) ?? '',
	}
}

type Handler = (params: unknown) => unknown

function installFakeTanstackApi(planning: PlanningResponse) {
	const calls: { method: string; path: string; params: unknown }[] = []
	const handlers = new Map<string, Handler>()

	function queryKeyFor(path: string, params: unknown) {
		const p = (params ?? {}) as { path?: unknown; query?: unknown }
		return [{ _id: path, path: p.path, query: p.query }]
	}

	function mock(method: string, path: string, handler: Handler) {
		handlers.set(`${method}:${path}`, handler)
	}

	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = queryKeyFor(path, params)
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params })
						if (path === PLANNING_PATH) {
							return { data: planning, pagination: null }
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
						calls.push({ method, path, params })
						const handler = handlers.get(`${method}:${path}`)
						if (!handler) {
							throw new Error(`unmocked ${method.toUpperCase()} ${path}`)
						}
						return handler(params)
					},
				},
			}
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return { calls, mock }
}

function renderFeature(
	options: {
		planning?: PlanningResponse
		overrides?: Partial<PlanningTeamFeatureProps>
	} = {},
) {
	const planning = options.planning ?? planningResponse()
	const { calls, mock } = installFakeTanstackApi(planning)
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

	const onViewChange = vi.fn()
	const onDateChange = vi.fn()

	render(
		<Providers>
			<PlanningTeamFeature
				view="week"
				date="2026-08-07"
				onViewChange={onViewChange}
				onDateChange={onDateChange}
				{...options.overrides}
			/>
		</Providers>,
	)

	return { calls, mock, onViewChange, onDateChange }
}

function patchCallsFor(
	calls: { method: string; path: string; params: unknown }[],
) {
	return calls.filter((c) => c.method === 'patch' && c.path === TASK_PATH)
}

describe('PlanningTeamFeature', () => {
	it('loads the planning with the from/to window derived from the view and the date', async () => {
		const { calls } = renderFeature()

		expect(await screen.findByText('Alix Martin')).toBeDefined()
		const call = calls.find((c) => c.path === PLANNING_PATH)
		expect(call?.params).toMatchObject({
			path: { organization_id: 'org-1' },
			query: { from: '2026-08-03', to: '2026-08-09' },
		})
	})

	it('recomputes the window for the month view', async () => {
		const { calls } = renderFeature({ overrides: { view: 'month' } })

		await screen.findByText('Alix Martin')
		const call = calls.find((c) => c.path === PLANNING_PATH)
		expect(call?.params).toMatchObject({
			query: { from: '2026-08-01', to: '2026-08-31' },
		})
	})

	it('reports the view change to the parent rather than handling it itself', async () => {
		const user = userEvent.setup()
		const { onViewChange } = renderFeature()

		await screen.findByText('Alix Martin')
		await user.click(screen.getByRole('tab', { name: 'Jour' }))

		expect(onViewChange).toHaveBeenCalledWith('day')
	})

	it('reports the date change to the parent', async () => {
		const user = userEvent.setup()
		const { onDateChange } = renderFeature()

		await screen.findByText('Alix Martin')
		await user.click(screen.getByRole('button', { name: 'Période suivante' }))

		expect(onDateChange).toHaveBeenCalledWith('2026-08-14')
	})
})

describe('PlanningTeamFeature — drag & drop without a warning', () => {
	it('a drop changing both day and row emits a single PATCH with starts_at/ends_at and the full assignee list', async () => {
		const { calls, mock } = renderFeature({
			planning: planningResponse({
				resources: [RESOURCE_EMPLOYEE_1, RESOURCE_EMPLOYEE_2],
				entries: [TASK_ENTRY],
			}),
		})
		mock('get', AVAILABILITY_PATH, () => ({
			data: { resources: [] },
			pagination: null,
		}))
		mock('patch', TASK_PATH, () => ({
			data: { task: TASK_ENTRY, created_employees: [] },
			pagination: null,
		}))

		await screen.findByText('Alix Martin')
		const segment = screen.getByTestId('grid-segment')
		const dataTransfer = fakeDataTransfer()
		fireEventDragStart(segment, dataTransfer)

		const targetRow = screen
			.getAllByTestId('grid-row')
			.find(
				(row) => row.getAttribute('data-resource-id') === 'employee:employee-2',
			)
		if (!targetRow) throw new Error('target row not found')
		const targetCell = within(targetRow)
			.getAllByTestId('grid-cell')
			.find((cell) => cell.getAttribute('data-date') === '2026-08-05')
		if (!targetCell) throw new Error('target cell not found')

		fireEventDrop(targetCell, dataTransfer)

		await waitFor(() => expect(patchCallsFor(calls)).toHaveLength(1))
		const patch = patchCallsFor(calls)[0]
		expect(patch.params).toMatchObject({
			path: { organization_id: 'org-1', task_id: 'wo-1' },
			body: {
				assignees: [{ kind: 'employee', employee_id: 'employee-2' }],
			},
		})
		const body = (
			patch.params as { body: { starts_at?: string; ends_at?: string } }
		).body
		expect(body.starts_at).toBeDefined()
		expect(body.ends_at).toBeDefined()
		expect(body.starts_at).not.toBe(TASK_ENTRY.starts_at)

		// Availability was checked once for the drop, not stacked into extra calls.
		const availabilityCalls = calls.filter(
			(c) => c.method === 'get' && c.path === AVAILABILITY_PATH,
		)
		expect(availabilityCalls).toHaveLength(1)
	})

	it('a drop changing neither day nor row emits no call', async () => {
		const { calls } = renderFeature({
			planning: planningResponse({ entries: [TASK_ENTRY] }),
		})

		await screen.findByText('Alix Martin')
		dropOnResourceDate('employee:employee-1', '2026-08-03')

		await new Promise((resolve) => setTimeout(resolve, 0))
		expect(calls.some((c) => c.path === AVAILABILITY_PATH)).toBe(false)
		expect(calls.some((c) => c.path === TASK_PATH)).toBe(false)
	})
})

describe('PlanningTeamFeature — avertissements', () => {
	it('a drop on a resource on leave opens the dialog; cancelling emits no PATCH', async () => {
		const { calls, mock } = renderFeature({
			planning: planningResponse({
				resources: [RESOURCE_EMPLOYEE_1, RESOURCE_EMPLOYEE_2],
				entries: [TASK_ENTRY],
			}),
		})
		mock('get', AVAILABILITY_PATH, () => ({
			data: {
				resources: [
					{
						resource_id: 'employee:employee-2',
						available: false,
						conflicts: [
							{
								kind: 'absence',
								reason: 'LEAVE',
								note: 'Vacances',
								starts_at: '2026-08-05T00:00:00+02:00',
								ends_at: '2026-08-06T00:00:00+02:00',
							},
						],
					},
				],
			},
			pagination: null,
		}))

		await screen.findByText('Alix Martin')
		dropOnResourceDate('employee:employee-2', '2026-08-05')

		expect(await screen.findByRole('alertdialog')).toBeDefined()
		expect(screen.getByText(/Absence : Congé/)).toBeDefined()

		const user = userEvent.setup()
		await user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(screen.queryByRole('alertdialog')).toBeNull()
		expect(patchCallsFor(calls)).toHaveLength(0)
	})

	it('confirming applies the mutation — a resource on leave stays assignable once confirmed', async () => {
		const { calls, mock } = renderFeature({
			planning: planningResponse({
				resources: [RESOURCE_EMPLOYEE_1, RESOURCE_EMPLOYEE_2],
				entries: [TASK_ENTRY],
			}),
		})
		mock('get', AVAILABILITY_PATH, () => ({
			data: {
				resources: [
					{
						resource_id: 'employee:employee-2',
						available: false,
						conflicts: [
							{
								kind: 'absence',
								reason: 'LEAVE',
								note: 'Vacances',
								starts_at: '2026-08-05T00:00:00+02:00',
								ends_at: '2026-08-06T00:00:00+02:00',
							},
						],
					},
				],
			},
			pagination: null,
		}))
		mock('patch', TASK_PATH, () => ({
			data: { task: TASK_ENTRY, created_employees: [] },
			pagination: null,
		}))

		await screen.findByText('Alix Martin')
		dropOnResourceDate('employee:employee-2', '2026-08-05')
		await screen.findByRole('alertdialog')

		const user = userEvent.setup()
		await user.click(screen.getByRole('button', { name: /Confirmer/ }))

		await waitFor(() => expect(patchCallsFor(calls)).toHaveLength(1))
		expect(patchCallsFor(calls)[0].params).toMatchObject({
			body: { assignees: [{ kind: 'employee', employee_id: 'employee-2' }] },
		})
		await waitFor(() => expect(screen.queryByRole('alertdialog')).toBeNull())
	})

	it('a drop on a member resource with no record shows missing_employee_record; confirming creates the record and warns about the hourly rate', async () => {
		const { calls, mock } = renderFeature({
			planning: planningResponse({
				resources: [RESOURCE_EMPLOYEE_1, RESOURCE_MEMBER],
				entries: [TASK_ENTRY],
			}),
		})
		mock('get', AVAILABILITY_PATH, () => ({
			data: { resources: [] },
			pagination: null,
		}))
		mock('patch', TASK_PATH, () => ({
			data: {
				task: TASK_ENTRY,
				created_employees: [
					{
						id: 'employee-9',
						organization_id: 'org-1',
						// The on-the-fly provisioning path always leaves `first_name`
						// unset — the account's `display_name` is a single free-text
						// field, exactly like the pre-split `employees.name` was.
						last_name: 'Jules Petit',
						first_name: null,
						user_id: 'user-9',
						hourly_rate_cents: null,
						weekly_contract_minutes: 0,
						created_at: '2026-08-07T00:00:00Z',
						updated_at: '2026-08-07T00:00:00Z',
					},
				],
			},
			pagination: null,
		}))

		await screen.findByText('Alix Martin')
		dropOnResourceDate('member:user-9', '2026-08-03')

		expect(await screen.findByRole('alertdialog')).toBeDefined()
		expect(screen.getByText(/Aucune fiche employé/)).toBeDefined()

		const user = userEvent.setup()
		await user.click(screen.getByRole('button', { name: /Confirmer/ }))

		await waitFor(() => expect(patchCallsFor(calls)).toHaveLength(1))
		expect(patchCallsFor(calls)[0].params).toMatchObject({
			body: { assignees: [{ kind: 'member', user_id: 'user-9' }] },
		})

		expect(
			await screen.findByText(/Fiche employé créée.*Jules Petit/),
		).toBeDefined()
		expect(screen.getByText(/taux horaire/)).toBeDefined()
	})
})

describe('PlanningTeamFeature — removing an assignee', () => {
	it('the remove button sends a PATCH straight away with the full list minus the removed resource, no dialog', async () => {
		const { calls, mock } = renderFeature({
			planning: planningResponse({
				resources: [RESOURCE_EMPLOYEE_1, RESOURCE_EMPLOYEE_2],
				entries: [
					{ ...TASK_ENTRY, employee_ids: ['employee-1', 'employee-2'] },
				],
			}),
		})
		mock('patch', TASK_PATH, () => ({
			data: { task: TASK_ENTRY, created_employees: [] },
			pagination: null,
		}))

		await screen.findByText('Alix Martin')
		const user = userEvent.setup()
		const removeButtons = screen.getAllByRole('button', {
			name: /Retirer cette personne/,
		})
		await user.click(removeButtons[0])

		await waitFor(() => expect(patchCallsFor(calls)).toHaveLength(1))
		expect(patchCallsFor(calls)[0].params).toMatchObject({
			body: { assignees: [{ kind: 'employee', employee_id: 'employee-2' }] },
		})
		expect(screen.queryByRole('alertdialog')).toBeNull()
		expect(calls.some((c) => c.path === AVAILABILITY_PATH)).toBe(false)
	})
})

describe('PlanningTeamFeature — absences (affichage lecture seule)', () => {
	it('still draws an absence segment on the grid, but with no button and no sheet — management moved to the HR module', async () => {
		const { calls } = renderFeature({
			planning: planningResponse({ entries: [ABSENCE_ENTRY] }),
		})

		await screen.findByText('Alix Martin')
		const segment = screen.getByTestId('grid-segment')
		expect(segment.getAttribute('data-tone')).toBe('absence')

		expect(
			screen.queryByRole('button', { name: /Ajouter une absence/ }),
		).toBeNull()

		const user = userEvent.setup()
		await user.click(segment)
		expect(screen.queryByRole('dialog')).toBeNull()

		// No absence endpoint is ever hit from the planning screen.
		expect(calls.some((c) => c.path.includes('/absences'))).toBe(false)
	})
})

// -- Helpers de simulation drag & drop --------------------------------------

function fireEventDragStart(element: Element, dataTransfer: unknown) {
	const event = new Event('dragstart', { bubbles: true, cancelable: true })
	Object.assign(event, { dataTransfer })
	element.dispatchEvent(event)
}

function fireEventDrop(element: Element, dataTransfer: unknown) {
	const event = new Event('drop', { bubbles: true, cancelable: true })
	Object.assign(event, { dataTransfer })
	element.dispatchEvent(event)
}

function dropOnResourceDate(resourceId: string, date: string) {
	const segment = screen.getByTestId('grid-segment')
	const dataTransfer = fakeDataTransfer()
	fireEventDragStart(segment, dataTransfer)

	const targetRow = screen
		.getAllByTestId('grid-row')
		.find((row) => row.getAttribute('data-resource-id') === resourceId)
	if (!targetRow) throw new Error(`target row not found: ${resourceId}`)
	const targetCell = within(targetRow)
		.getAllByTestId('grid-cell')
		.find((cell) => cell.getAttribute('data-date') === date)
	if (!targetCell) throw new Error(`target cell not found: ${date}`)

	fireEventDrop(targetCell, dataTransfer)
}
