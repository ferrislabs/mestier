import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { PlanningCalendarFeature } from '#/pages/planning/feature/planning-calendar-feature'
import type { PlanningResponse } from '#/pages/planning/types'

// jsdom has no ResizeObserver, which Radix primitives (Popover, Select) probe
// defensively — same stub as `planning-team-feature.test.tsx`.
class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfill
const globalAny = globalThis as any
globalAny.ResizeObserver ??= ResizeObserverStub
Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

const PLANNING_PATH = '/api/v1/organizations/{organization_id}/planning'
const TASK_PATH = '/api/v1/organizations/{organization_id}/tasks/{task_id}'
const TASKS_PATH = '/api/v1/organizations/{organization_id}/tasks'
const TASK_LABELS_PATH = '/api/v1/organizations/{organization_id}/task-labels'
const CUSTOMERS_PATH = '/api/v1/organizations/{organization_id}/customers'
const QUOTES_PATH = '/api/v1/organizations/{organization_id}/quotes'
const TASK_COMMENTS_PATH =
	'/api/v1/organizations/{organization_id}/tasks/{task_id}/comments'
const ABSENCE_PATH =
	'/api/v1/organizations/{organization_id}/absences/{absence_id}'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	slug: 'atelier-bois',
	field_clock_enabled: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const RESOURCE_MEMBER_1 = {
	resource_id: 'member:member-1',
	member_id: 'member-1',
	employee_id: 'employee-1',
	display_name: 'Alix Martin',
	hourly_rate_cents: 1500,
	weekly_contract_minutes: 2100,
}

const TASK_ENTRY = {
	kind: 'task' as const,
	labels: [],
	title: 'Poser la terrasse',
	blocks_availability: true,
	child_count: 0,
	id: 'wo-1',
	starts_at: '2026-08-03T08:00:00+02:00',
	ends_at: '2026-08-03T10:00:00+02:00',
	all_day: false,
	status: 'PLANNED' as const,
	member_ids: ['member-1'],
	customer_name: null,
	context_label: null,
}

const ABSENCE_ENTRY = {
	kind: 'absence' as const,
	id: 'ab-1',
	starts_at: '2026-08-03T00:00:00+02:00',
	ends_at: '2026-08-04T00:00:00+02:00',
	all_day: true,
	absence_kind: 'LEAVE' as const,
	member_id: 'member-1',
	note: 'Vacances',
}

function planningResponse(
	overrides: Partial<PlanningResponse> = {},
): PlanningResponse {
	return {
		timezone: 'Europe/Paris',
		resources: [RESOURCE_MEMBER_1],
		entries: [],
		work_time: [],
		...overrides,
	}
}

type Handler = (params: unknown) => unknown

function installFakeTanstackApi() {
	const calls: { method: string; path: string; params: unknown }[] = []
	const getHandlers = new Map<string, Handler>()
	const mutationHandlers = new Map<string, Handler>()

	function queryKeyFor(path: string, params: unknown) {
		const p = (params ?? {}) as { path?: unknown; query?: unknown }
		return [{ _id: path, path: p.path, query: p.query }]
	}

	function mockGet(path: string, handler: Handler) {
		getHandlers.set(path, handler)
	}

	function mockMutation(method: string, path: string, handler: Handler) {
		mutationHandlers.set(`${method}:${path}`, handler)
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
						const handler = getHandlers.get(path)
						if (!handler) throw new Error(`unmocked GET ${path}`)
						return handler(params)
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
						const handler = mutationHandlers.get(`${method}:${path}`)
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

	return { calls, mockGet, mockMutation }
}

function renderFeature(
	configure: (api: ReturnType<typeof installFakeTanstackApi>) => void,
) {
	const api = installFakeTanstackApi()
	const { mockGet } = api
	// Reference data `TaskSheetFeature` needs whenever the "Modifier en
	// détail" door opens it — empty by default, individual tests override
	// what they actually exercise.
	mockGet(TASK_LABELS_PATH, () => ({ data: [], pagination: null }))
	mockGet(CUSTOMERS_PATH, () => ({ data: [], pagination: null }))
	mockGet(QUOTES_PATH, () => ({ data: [], pagination: null }))
	mockGet(TASKS_PATH, () => ({ data: [], pagination: null }))
	mockGet(TASK_COMMENTS_PATH, () => ({
		data: [],
		pagination: {
			current_page: 1,
			first_page: 1,
			is_empty: true,
			last_page: 1,
			next_page: null,
			per_page: 20,
			prev_page: null,
			total: 0,
		},
	}))
	configure(api)

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
			<PlanningCalendarFeature
				view="week"
				date="2026-08-07"
				onViewChange={onViewChange}
				onDateChange={onDateChange}
			/>
		</Providers>,
	)

	return { ...api, onViewChange, onDateChange }
}

describe('PlanningCalendarFeature — quick action failures', () => {
	it('surfaces a failed status change in the popover instead of leaving it silently unresponsive', async () => {
		const { mockMutation } = renderFeature((api) => {
			api.mockGet(PLANNING_PATH, () => ({
				data: planningResponse({ entries: [TASK_ENTRY] }),
				pagination: null,
			}))
		})
		mockMutation('patch', TASK_PATH, () => {
			throw new Error('HTTP 409: Conflict')
		})

		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', { name: /Poser la terrasse/ }),
		)
		await user.click(await screen.findByRole('button', { name: 'En cours' }))

		expect(await screen.findByText('HTTP 409: Conflict')).toBeDefined()
	})

	it('surfaces a failed absence deletion in the popover', async () => {
		const { mockMutation } = renderFeature((api) => {
			api.mockGet(PLANNING_PATH, () => ({
				data: planningResponse({ entries: [ABSENCE_ENTRY] }),
				pagination: null,
			}))
		})
		mockMutation('delete', ABSENCE_PATH, () => {
			throw new Error('HTTP 500: Internal Server Error')
		})

		const user = userEvent.setup()
		// Exact match: the toolbar's own nature filter is labelled "Congés"
		// (plural) and would otherwise also match a loose /Congé/ pattern.
		await user.click(await screen.findByRole('button', { name: 'Congé' }))
		await user.click(
			await screen.findByRole('button', { name: 'Supprimer cette absence' }),
		)

		expect(
			await screen.findByText('HTTP 500: Internal Server Error'),
		).toBeDefined()
	})

	it('does not carry a stale error from one entry’s popover into another’s', async () => {
		const otherTask = {
			...TASK_ENTRY,
			id: 'wo-2',
			title: 'Tailler la haie',
			starts_at: '2026-08-04T08:00:00+02:00',
			ends_at: '2026-08-04T10:00:00+02:00',
		}
		const { mockMutation } = renderFeature((api) => {
			api.mockGet(PLANNING_PATH, () => ({
				data: planningResponse({ entries: [TASK_ENTRY, otherTask] }),
				pagination: null,
			}))
		})
		mockMutation('patch', TASK_PATH, (params) => {
			const taskId = (params as { path: { task_id: string } }).path.task_id
			if (taskId === 'wo-1') throw new Error('HTTP 409: Conflict')
			return { data: { task: otherTask } }
		})

		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', { name: /Poser la terrasse/ }),
		)
		await user.click(await screen.findByRole('button', { name: 'En cours' }))
		expect(await screen.findByText('HTTP 409: Conflict')).toBeDefined()

		// Close this popover (Escape) and open the other task's — its own
		// status change never failed, so no error should show for it.
		await user.keyboard('{Escape}')
		await user.click(
			await screen.findByRole('button', { name: /Tailler la haie/ }),
		)

		expect(screen.queryByText('HTTP 409: Conflict')).toBeNull()
	})
})

describe('PlanningCalendarFeature — door into the full task sheet', () => {
	it('opens the same TaskSheetFeature the Team and Task-list views use, from the calendar popover', async () => {
		renderFeature((api) => {
			api.mockGet(PLANNING_PATH, () => ({
				data: planningResponse({ entries: [TASK_ENTRY] }),
				pagination: null,
			}))
			api.mockGet(TASK_PATH, (params) => ({
				data: {
					...TASK_ENTRY,
					id: (params as { path: { task_id: string } }).path.task_id,
					organization_id: 'org-1',
					description: null,
					parent_task_id: null,
					customer_id: null,
					customer_context_id: null,
					quote_id: null,
					created_at: '2026-08-01T00:00:00Z',
					updated_at: '2026-08-01T00:00:00Z',
				},
				pagination: null,
			}))
		})

		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', { name: /Poser la terrasse/ }),
		)
		await user.click(
			await screen.findByRole('button', { name: 'Modifier en détail' }),
		)

		expect(await screen.findByText('Modifier la tâche')).toBeDefined()
	})
})
