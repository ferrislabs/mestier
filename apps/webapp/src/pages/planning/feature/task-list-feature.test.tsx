import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { TaskListFeature } from '#/pages/planning/feature/task-list-feature'

// jsdom has no ResizeObserver, which Radix primitives (Select, Popover,
// Tabs) probe defensively — see the same stub in
// `planning-team-feature.test.tsx` and `task-sheet-feature.test.tsx`.
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
const TASKS_PATH = '/api/v1/organizations/{organization_id}/tasks'
const TASK_PATH = '/api/v1/organizations/{organization_id}/tasks/{task_id}'
const TASK_LABELS_PATH = '/api/v1/organizations/{organization_id}/task-labels'
const CUSTOMERS_PATH = '/api/v1/organizations/{organization_id}/customers'
const TASK_COMMENTS_PATH =
	'/api/v1/organizations/{organization_id}/tasks/{task_id}/comments'
const BULK_ASSIGN_TASKS_PATH =
	'/api/v1/organizations/{organization_id}/tasks/bulk-assign'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	slug: 'atelier-bois',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const RESOURCE_MEMBER = {
	resource_id: 'member:member-1',
	member_id: 'member-1',
	employee_id: 'employee-1',
	display_name: 'Alix Martin',
	hourly_rate_cents: 1500,
	weekly_contract_minutes: 2100,
}

function task(overrides: Record<string, unknown> = {}) {
	return {
		id: 'root-1',
		organization_id: 'org-1',
		title: 'Projet toiture',
		description: null,
		status: 'PLANNED' as const,
		all_day: false,
		blocks_availability: true,
		parent_task_id: null,
		child_count: 0,
		customer_id: null,
		customer_context_id: null,
		quote_id: null,
		starts_at: '2026-08-10T07:00:00+02:00',
		ends_at: '2026-08-10T09:00:00+02:00',
		member_ids: ['member-1'],
		labels: [],
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
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
						if (!handler)
							throw new Error(`unmocked ${method.toUpperCase()} ${path}`)
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

function tasksHandler(rootTasks: ReturnType<typeof task>[]) {
	return (params: unknown) => {
		const query = (params as { query?: { parent_task_id?: string } }).query
		if (query?.parent_task_id) {
			return { data: [], pagination: null }
		}
		return {
			data: rootTasks,
			pagination: {
				current_page: 1,
				first_page: 1,
				is_empty: rootTasks.length === 0,
				last_page: 1,
				next_page: null,
				per_page: 20,
				prev_page: null,
				total: rootTasks.length,
			},
		}
	}
}

function renderFeature(
	configure: (api: ReturnType<typeof installFakeTanstackApi>) => void,
) {
	const api = installFakeTanstackApi()
	const { calls, mockGet, mockMutation } = api
	mockGet(PLANNING_PATH, () => ({
		data: {
			timezone: 'Europe/Paris',
			resources: [RESOURCE_MEMBER],
			entries: [],
			work_time: [],
		},
		pagination: null,
	}))
	mockGet(TASK_LABELS_PATH, () => ({ data: [], pagination: null }))
	mockGet(CUSTOMERS_PATH, () => ({ data: [], pagination: null }))
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

	render(
		<Providers>
			<TaskListFeature />
		</Providers>,
	)

	return { calls, mockGet, mockMutation }
}

describe('TaskListFeature — chargement', () => {
	it("loads page 1's root tasks, with no parent_task_id", async () => {
		const { calls } = renderFeature((api) =>
			api.mockGet(TASKS_PATH, tasksHandler([task()])),
		)

		expect(await screen.findByText('Projet toiture')).toBeDefined()
		const call = calls.find(
			(c) =>
				c.path === TASKS_PATH &&
				!(c.params as { query?: { parent_task_id?: string } }).query
					?.parent_task_id,
		)
		expect(call?.params).toMatchObject({
			path: { organization_id: 'org-1' },
			query: { page: 1, per_page: 20 },
		})
	})
})

describe('TaskListFeature — lazy expansion', () => {
	it('loads subtasks only on expansion, not on the initial render', async () => {
		const { calls } = renderFeature((api) =>
			api.mockGet(TASKS_PATH, (params) => {
				const query = (params as { query?: { parent_task_id?: string } }).query
				if (query?.parent_task_id === 'root-1') {
					return {
						data: [
							task({
								id: 'sub-1',
								title: 'Poser les tuiles',
								parent_task_id: 'root-1',
								starts_at: null,
								ends_at: null,
							}),
						],
						pagination: null,
					}
				}
				return tasksHandler([task({ child_count: 1 })])(params)
			}),
		)

		await screen.findByText('Projet toiture')
		expect(
			calls.some(
				(c) =>
					c.path === TASKS_PATH &&
					(c.params as { query?: { parent_task_id?: string } }).query
						?.parent_task_id === 'root-1',
			),
		).toBe(false)

		const user = userEvent.setup()
		await user.click(
			screen.getByRole('button', { name: 'Afficher les sous-tâches' }),
		)

		expect(await screen.findByText('Poser les tuiles')).toBeDefined()
		expect(
			calls.some(
				(c) =>
					c.path === TASKS_PATH &&
					(c.params as { query?: { parent_task_id?: string } }).query
						?.parent_task_id === 'root-1',
			),
		).toBe(true)
	})
})

describe('TaskListFeature — pagination', () => {
	it('asks for the next page when "Suivant" is clicked', async () => {
		const rootTasks = [task()]
		const { calls } = renderFeature((api) =>
			api.mockGet(TASKS_PATH, (params) => {
				const query = (
					params as { query?: { parent_task_id?: string; page?: number } }
				).query
				if (query?.parent_task_id) return { data: [], pagination: null }
				return {
					data: rootTasks,
					pagination: {
						current_page: query?.page ?? 1,
						first_page: 1,
						is_empty: false,
						last_page: 2,
						next_page: query?.page === 1 ? 2 : null,
						per_page: 20,
						prev_page: query?.page === 2 ? 1 : null,
						total: 21,
					},
				}
			}),
		)

		await screen.findByText('Projet toiture')
		const user = userEvent.setup()
		await user.click(screen.getByRole('button', { name: 'Suivant' }))

		await waitFor(() => {
			const call = calls.find(
				(c) =>
					c.path === TASKS_PATH &&
					(c.params as { query?: { page?: number; parent_task_id?: string } })
						.query?.page === 2 &&
					!(c.params as { query?: { parent_task_id?: string } }).query
						?.parent_task_id,
			)
			expect(call).toBeDefined()
		})
	})
})

describe('TaskListFeature — reference data loading', () => {
	it('renders the table only once tasks AND reference data are loaded, never with an unresolved assignee', async () => {
		let resolvePlanning: (value: unknown) => void = () => {}
		const planningPromise = new Promise((resolve) => {
			resolvePlanning = resolve
		})

		renderFeature((api) => {
			api.mockGet(TASKS_PATH, tasksHandler([task()]))
			api.mockGet(PLANNING_PATH, () => planningPromise)
		})

		// The roster fetch is still pending — the table (and any assignee
		// cell) must not render yet, even though the root tasks themselves
		// may already have resolved.
		expect(screen.getByText(/Chargement/)).toBeDefined()
		expect(screen.queryByTestId('task-row-root-1')).toBeNull()
		expect(screen.queryByText('Assigné inconnu')).toBeNull()

		resolvePlanning({
			data: {
				timezone: 'Europe/Paris',
				resources: [RESOURCE_MEMBER],
				entries: [],
				work_time: [],
			},
			pagination: null,
		})

		expect(await screen.findByTestId('task-row-root-1')).toBeDefined()
		expect(screen.getByText('Alix Martin')).toBeDefined()
		expect(screen.queryByText('Assigné inconnu')).toBeNull()
	})
})

describe('TaskListFeature — full day', () => {
	it('a subtask with no dates of its own, under a full-day root, shows the inherited date with no time', async () => {
		const { mockGet } = renderFeature((api) =>
			api.mockGet(TASKS_PATH, (params) => {
				const query = (params as { query?: { parent_task_id?: string } }).query
				if (query?.parent_task_id === 'root-1') {
					return {
						data: [
							task({
								id: 'sub-1',
								title: 'Livraison matériaux',
								parent_task_id: 'root-1',
								starts_at: null,
								ends_at: null,
								all_day: false,
							}),
						],
						pagination: null,
					}
				}
				return tasksHandler([
					task({
						child_count: 1,
						all_day: true,
						starts_at: '2026-08-09T22:00:00.000Z',
						ends_at: '2026-08-10T22:00:00.000Z',
					}),
				])(params)
			}),
		)
		mockGet(TASK_PATH, (params) => ({
			data: task({
				id: (params as { path: { task_id: string } }).path.task_id,
			}),
			pagination: null,
		}))

		await screen.findByText('Projet toiture')
		const user = userEvent.setup()
		await user.click(
			screen.getByRole('button', { name: 'Afficher les sous-tâches' }),
		)

		const subtaskRow = await screen.findByTestId('subtask-row-sub-1')
		expect(subtaskRow.textContent).toContain('10/08/2026')
		expect(subtaskRow.textContent).not.toContain('00:00')
	})
})

describe('TaskListFeature — bulk assign', () => {
	it('shows the bulk-assign bar once a row is selected, and hides it again on cancel', async () => {
		renderFeature((api) => api.mockGet(TASKS_PATH, tasksHandler([task()])))

		await screen.findByText('Projet toiture')
		expect(screen.queryByText(/tâche sélectionnée/)).toBeNull()

		const user = userEvent.setup()
		await user.click(
			screen.getByRole('checkbox', { name: 'Sélectionner Projet toiture' }),
		)
		expect(await screen.findByText('1 tâche sélectionnée')).toBeDefined()

		await user.click(screen.getByRole('button', { name: 'Annuler' }))
		expect(screen.queryByText(/tâche sélectionnée/)).toBeNull()
	})

	it('applies the picked assignee to every selected task via one PATCH per task, merged with each task’s own existing assignees — never the replace-everything bulk-assign route', async () => {
		const { calls } = renderFeature((api) => {
			api.mockGet(
				TASKS_PATH,
				tasksHandler([
					task({ member_ids: [] }),
					task({
						id: 'root-2',
						title: 'Réunion projet',
						member_ids: ['member-9'],
					}),
				]),
			)
			api.mockMutation('patch', TASK_PATH, (params) => ({
				data: {
					task: task({
						id: (params as { path: { task_id: string } }).path.task_id,
					}),
				},
			}))
		})

		await screen.findByText('Projet toiture')
		const user = userEvent.setup()

		// Select both rows, then open the picker and pick the one available
		// resource.
		await user.click(
			screen.getByRole('checkbox', { name: 'Sélectionner Projet toiture' }),
		)
		await user.click(
			screen.getByRole('checkbox', { name: 'Sélectionner Réunion projet' }),
		)
		await user.click(screen.getByRole('button', { name: /Personne assigné/ }))
		await user.click(screen.getByRole('option', { name: 'Alix Martin' }))
		await user.click(screen.getByRole('button', { name: 'Assigner' }))

		await waitFor(() => {
			const patchCalls = calls.filter(
				(c) => c.method === 'patch' && c.path === TASK_PATH,
			)
			expect(patchCalls).toHaveLength(2)
		})

		expect(calls.some((c) => c.path === BULK_ASSIGN_TASKS_PATH)).toBe(false)

		const patchCalls = calls.filter(
			(c) => c.method === 'patch' && c.path === TASK_PATH,
		)
		const assigneesByTaskId = new Map(
			patchCalls.map((call) => [
				(call.params as { path: { task_id: string } }).path.task_id,
				(call.params as { body: { assignees: { member_id: string }[] } }).body
					.assignees,
			]),
		)
		// root-1 had no assignee: the picked one is simply added.
		expect(assigneesByTaskId.get('root-1')).toEqual([{ member_id: 'member-1' }])
		// root-2 already had member-9: the picked assignee is appended, not
		// used to replace it.
		expect(assigneesByTaskId.get('root-2')).toEqual([
			{ member_id: 'member-9' },
			{ member_id: 'member-1' },
		])

		await waitFor(() => {
			expect(screen.queryByText(/tâche sélectionnée/)).toBeNull()
		})
	})

	it('names the task that failed instead of a generic error, and keeps the selection so the user can retry', async () => {
		renderFeature((api) => {
			api.mockGet(
				TASKS_PATH,
				tasksHandler([task(), task({ id: 'root-2', title: 'Réunion projet' })]),
			)
			api.mockMutation('patch', TASK_PATH, (params) => {
				const taskId = (params as { path: { task_id: string } }).path.task_id
				if (taskId === 'root-2') {
					throw new Error('HTTP 409: Conflict')
				}
				return { data: { task: task({ id: taskId }) } }
			})
		})

		await screen.findByText('Projet toiture')
		const user = userEvent.setup()

		await user.click(
			screen.getByRole('checkbox', { name: 'Sélectionner Projet toiture' }),
		)
		await user.click(
			screen.getByRole('checkbox', { name: 'Sélectionner Réunion projet' }),
		)
		await user.click(screen.getByRole('button', { name: /Personne assigné/ }))
		await user.click(screen.getByRole('option', { name: 'Alix Martin' }))
		await user.click(screen.getByRole('button', { name: 'Assigner' }))

		const error = await screen.findByText(/Échec de l'affectation pour/)
		expect(error.textContent).toContain('Réunion projet')
		expect(error.textContent).not.toContain('Projet toiture')

		// The bar stays open — the successful task's PATCH already went
		// through, but the selection is kept so the failed one can be retried.
		expect(screen.getByText('2 tâches sélectionnées')).toBeDefined()
	})
})

describe('TaskListFeature — opening the Sheet', () => {
	it('opens the Sheet on the clicked task', async () => {
		const { calls, mockGet } = renderFeature((api) =>
			api.mockGet(TASKS_PATH, tasksHandler([task()])),
		)
		mockGet(TASK_PATH, (params) => ({
			data: task({
				id: (params as { path: { task_id: string } }).path.task_id,
			}),
			pagination: null,
		}))

		await screen.findByText('Projet toiture')
		const user = userEvent.setup()
		await user.click(screen.getByTestId('task-row-root-1'))

		expect(await screen.findByText('Modifier la tâche')).toBeDefined()
		await waitFor(() => {
			expect(
				calls.some(
					(c) =>
						c.path === TASK_PATH &&
						(c.params as { path: { task_id: string } }).path.task_id ===
							'root-1',
				),
			).toBe(true)
		})
	})

	it('opens the Sheet on the right subtask', async () => {
		const { calls, mockGet } = renderFeature((api) =>
			api.mockGet(TASKS_PATH, (params) => {
				const query = (params as { query?: { parent_task_id?: string } }).query
				if (query?.parent_task_id === 'root-1') {
					return {
						data: [
							task({
								id: 'sub-1',
								title: 'Poser les tuiles',
								parent_task_id: 'root-1',
								starts_at: null,
								ends_at: null,
							}),
						],
						pagination: null,
					}
				}
				return tasksHandler([task({ child_count: 1 })])(params)
			}),
		)
		mockGet(TASK_PATH, (params) => ({
			data: task({
				id: (params as { path: { task_id: string } }).path.task_id,
				title: 'Poser les tuiles',
			}),
			pagination: null,
		}))

		await screen.findByText('Projet toiture')
		const user = userEvent.setup()
		await user.click(
			screen.getByRole('button', { name: 'Afficher les sous-tâches' }),
		)
		await user.click(await screen.findByTestId('subtask-row-sub-1'))

		expect(await screen.findByText('Modifier la tâche')).toBeDefined()
		await waitFor(() => {
			expect(
				calls.some(
					(c) =>
						c.path === TASK_PATH &&
						(c.params as { path: { task_id: string } }).path.task_id ===
							'sub-1',
				),
			).toBe(true)
		})
	})
})
