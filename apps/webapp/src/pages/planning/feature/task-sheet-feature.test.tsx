import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	TaskSheetFeature,
	type TaskSheetFeatureProps,
} from '#/pages/planning/feature/task-sheet-feature'

// jsdom has no ResizeObserver, which Radix primitives (Select, Popover,
// Tabs) probe defensively.
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

const TASKS_PATH = '/api/v1/organizations/{organization_id}/tasks'
const TASK_PATH = '/api/v1/organizations/{organization_id}/tasks/{task_id}'
const TASK_LABELS_PATH = '/api/v1/organizations/{organization_id}/task-labels'
const CUSTOMERS_PATH = '/api/v1/organizations/{organization_id}/customers'
const TASK_COMMENTS_PATH =
	'/api/v1/organizations/{organization_id}/tasks/{task_id}/comments'
const ASSIGNMENT_REPORTS_PATH =
	'/api/v1/organizations/{organization_id}/assignment-reports'
const ASSIGNMENT_REPORT_RESOLUTION_PATH =
	'/api/v1/assignment-reports/{assignment_report_id}/resolution'

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

const LABEL_REUNION = {
	id: 'label-1',
	organization_id: 'org-1',
	name: 'Réunion',
	color: '#2563EB',
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

/**
 * `configure` runs before `render()` — any GET a hook fetches on mount
 * (`useTask`'s task-in-edit-mode fetch, in particular) must already have a
 * handler registered by then, since the query fires from a mount effect
 * before the test body gets a chance to call `mockGet` afterwards (mirrors
 * `planning-team-feature.test.tsx`'s own `installFakeTanstackApi` — the
 * always-on-mount `PLANNING_PATH` fetch is baked in up front there too).
 */
function renderFeature(
	overrides: Partial<TaskSheetFeatureProps> = {},
	configure?: (api: ReturnType<typeof installFakeTanstackApi>) => void,
) {
	const api = installFakeTanstackApi()
	const { calls, mockGet, mockMutation } = api
	mockGet(TASK_LABELS_PATH, () => ({ data: [LABEL_REUNION], pagination: null }))
	mockGet(CUSTOMERS_PATH, () => ({ data: [], pagination: null }))
	configure?.(api)

	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
		)
	}

	const onOpenChange = vi.fn()
	const onNavigate = vi.fn()

	const props: TaskSheetFeatureProps = {
		organizationId: 'org-1',
		timeZone: 'Europe/Paris',
		resources: [RESOURCE_MEMBER],
		open: true,
		target: { mode: 'create', parentTaskId: null },
		onOpenChange,
		onNavigate,
		...overrides,
	}

	render(
		<Providers>
			<TaskSheetFeature {...props} />
		</Providers>,
	)

	return { calls, mockGet, mockMutation, onOpenChange, onNavigate }
}

/** `POST /tasks` calls — always `TASKS_PATH`. */
function postTaskCalls(
	calls: { method: string; path: string; params: unknown }[],
) {
	return calls.filter((c) => c.method === 'post' && c.path === TASKS_PATH)
}

/** `PATCH /tasks/{task_id}` calls — always `TASK_PATH`, distinct from `TASKS_PATH`'s list/create path. */
function patchTaskCalls(
	calls: { method: string; path: string; params: unknown }[],
) {
	return calls.filter((c) => c.method === 'patch' && c.path === TASK_PATH)
}

describe('TaskSheetFeature — creation without a customer', () => {
	it('sends a POST with customer_id/customer_context_id null and no follow-up PATCH', async () => {
		const user = userEvent.setup()
		const { calls, mockMutation, onOpenChange } = renderFeature()
		mockMutation('post', TASKS_PATH, () => ({
			data: { id: 'task-1', customer_id: null, customer_context_id: null },
			pagination: null,
		}))

		await user.type(screen.getByLabelText('Titre'), 'Réunion de projet')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		await waitFor(() => expect(postTaskCalls(calls)).toHaveLength(1))
		const body = (
			postTaskCalls(calls)[0].params as { body: Record<string, unknown> }
		).body
		expect(body.customer_id).toBeNull()
		expect(body.customer_context_id).toBeNull()

		expect(patchTaskCalls(calls)).toHaveLength(0)
		await waitFor(() => expect(onOpenChange).toHaveBeenCalledWith(false))
	})
})

describe('TaskSheetFeature — creation with assignees and labels', () => {
	it('chains a follow-up PATCH carrying the full list of chosen assignees and labels', async () => {
		const user = userEvent.setup()
		const { calls, mockMutation } = renderFeature()
		mockMutation('post', TASKS_PATH, () => ({
			data: { id: 'task-1' },
			pagination: null,
		}))
		mockMutation('patch', TASK_PATH, () => ({
			data: { task: { id: 'task-1' } },
			pagination: null,
		}))

		await user.type(screen.getByLabelText('Titre'), 'Projet toiture')

		await user.click(screen.getByRole('button', { name: /Aucun label/ }))
		await user.click(screen.getByRole('option', { name: /Réunion/ }))

		await user.click(screen.getByRole('button', { name: /Personne assigné/ }))
		await user.click(screen.getByRole('option', { name: /Alix Martin/ }))

		await user.click(screen.getByRole('button', { name: 'Créer' }))

		await waitFor(() => expect(patchTaskCalls(calls)).toHaveLength(1))
		const patchBody = (
			patchTaskCalls(calls)[0].params as { body: Record<string, unknown> }
		).body
		expect(patchBody.label_ids).toEqual(['label-1'])
		expect(patchBody.assignees).toEqual([{ member_id: 'member-1' }])
	})
})

describe('TaskSheetFeature — editing, labels', () => {
	function editTarget() {
		return { mode: 'edit' as const, taskId: 'task-1' }
	}

	function mockEditTask(
		mockGet: ReturnType<typeof installFakeTanstackApi>['mockGet'],
	) {
		mockGet(TASK_PATH, () => ({
			data: {
				id: 'task-1',
				title: 'Projet toiture',
				description: null,
				all_day: false,
				starts_at: '2026-08-10T07:00:00.000Z',
				ends_at: '2026-08-10T09:00:00.000Z',
				blocks_availability: true,
				status: 'PLANNED',
				parent_task_id: null,
				customer_id: null,
				customer_context_id: null,
				child_count: 0,
				labels: [LABEL_REUNION],
				member_ids: [],
			},
			pagination: null,
		}))
		mockGet(TASKS_PATH, () => ({ data: [], pagination: null }))
	}

	it('sends an empty label_ids to strip every existing label', async () => {
		const user = userEvent.setup()
		const { calls, mockMutation } = renderFeature(
			{ target: editTarget() },
			(api) => mockEditTask(api.mockGet),
		)
		mockMutation('patch', TASK_PATH, () => ({
			data: { task: { id: 'task-1' } },
			pagination: null,
		}))

		expect(await screen.findByDisplayValue('Projet toiture')).toBeDefined()
		// The one seeded label starts selected — deselect it.
		await user.click(screen.getByRole('button', { name: /Réunion/ }))
		await user.click(screen.getByRole('option', { name: /Réunion/ }))

		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => expect(patchTaskCalls(calls)).toHaveLength(1))
		const body = (
			patchTaskCalls(calls)[0].params as { body: Record<string, unknown> }
		).body
		expect(body.label_ids).toEqual([])
	})
})

describe('TaskSheetFeature — paginated comment thread', () => {
	it('asks only for the first page on open, never the whole history', async () => {
		const { calls } = renderFeature(
			{ target: { mode: 'edit', taskId: 'task-1' } },
			(api) => {
				api.mockGet(TASK_PATH, () => ({
					data: {
						id: 'task-1',
						title: 'Projet',
						description: null,
						all_day: false,
						starts_at: '2026-08-10T07:00:00.000Z',
						ends_at: '2026-08-10T09:00:00.000Z',
						blocks_availability: true,
						status: 'PLANNED',
						parent_task_id: null,
						customer_id: null,
						customer_context_id: null,
						child_count: 0,
						labels: [],
						member_ids: [],
					},
					pagination: null,
				}))
				api.mockGet(TASKS_PATH, () => ({ data: [], pagination: null }))
				api.mockGet(TASK_COMMENTS_PATH, () => ({
					data: [
						{
							id: 'c1',
							task_id: 'task-1',
							organization_id: 'org-1',
							author: { id: 'u1', display_name: 'Bob Martin' },
							author_is_self: false,
							body: 'Salut',
							created_at: '2026-08-10T08:00:00Z',
							updated_at: '2026-08-10T08:00:00Z',
						},
					],
					pagination: {
						current_page: 1,
						first_page: 1,
						is_empty: false,
						per_page: 20,
						last_page: 3,
						next_page: 2,
						total: 45,
					},
				}))
			},
		)

		await screen.findByDisplayValue('Projet')
		const user = userEvent.setup()
		await user.click(screen.getByRole('tab', { name: /Commentaires/ }))

		expect(await screen.findByText('Salut')).toBeDefined()
		const commentCalls = calls.filter(
			(c) => c.method === 'get' && c.path === TASK_COMMENTS_PATH,
		)
		expect(commentCalls).toHaveLength(1)
		expect(
			(commentCalls[0].params as { query: { page: number; per_page: number } })
				.query,
		).toEqual({ page: 1, per_page: 20 })

		// The "load more" control must be present rather than the thread
		// silently fetching page 2 itself.
		expect(screen.getByRole('button', { name: /plus récents/i })).toBeDefined()
	})
})

describe('TaskSheetFeature — correction loop', () => {
	function editTarget() {
		return { mode: 'edit' as const, taskId: 'task-1' }
	}

	const PENDING_REPORT = {
		id: 'report-1',
		organization_id: 'org-1',
		task_assignment_id: 'assignment-1',
		reported_minutes: 300,
		comment: 'Chantier plus long que prévu',
		reported_by: 'member-1',
		resolution: 'PENDING',
		resolved_by: null,
		resolved_at: null,
		resolution_note: null,
		created_at: '2026-08-10T08:00:00Z',
		updated_at: '2026-08-10T08:00:00Z',
	}

	function mockEditTaskWithAssignment(
		mockGet: ReturnType<typeof installFakeTanstackApi>['mockGet'],
	) {
		mockGet(TASK_PATH, () => ({
			data: {
				id: 'task-1',
				title: 'Projet toiture',
				description: null,
				all_day: false,
				starts_at: '2026-08-10T07:00:00.000Z',
				ends_at: '2026-08-10T11:00:00.000Z',
				blocks_availability: true,
				status: 'PLANNED',
				parent_task_id: null,
				customer_id: null,
				customer_context_id: null,
				child_count: 0,
				labels: [],
				member_ids: ['member-1'],
				assignments: [{ id: 'assignment-1', member_id: 'member-1' }],
			},
			pagination: null,
		}))
		mockGet(TASKS_PATH, () => ({ data: [], pagination: null }))
	}

	function resolutionCalls(
		calls: { method: string; path: string; params: unknown }[],
	) {
		return calls.filter(
			(c) =>
				c.method === 'patch' && c.path === ASSIGNMENT_REPORT_RESOLUTION_PATH,
		)
	}

	it('shows the pending report with what was planned and what was reported', async () => {
		renderFeature({ target: editTarget() }, (api) => {
			mockEditTaskWithAssignment(api.mockGet)
			api.mockGet(ASSIGNMENT_REPORTS_PATH, () => ({
				data: [PENDING_REPORT],
				pagination: null,
			}))
		})

		expect(await screen.findByText(/prévu : 4 h 00/i)).toBeDefined()
		expect(screen.getByText(/déclaré : 5 h 00/i)).toBeDefined()
		expect(screen.getByText(/par alix martin/i)).toBeDefined()
	})

	/**
	 * The issue's own acceptance criterion: two calls, task then report, and
	 * the report is only marked applied once the task edit actually
	 * succeeded.
	 */
	it('applying prefills the end time, then PATCHes the task before resolving the report as applied', async () => {
		const user = userEvent.setup()
		const { calls, mockMutation } = renderFeature(
			{ target: editTarget() },
			(api) => {
				mockEditTaskWithAssignment(api.mockGet)
				api.mockGet(ASSIGNMENT_REPORTS_PATH, () => ({
					data: [PENDING_REPORT],
					pagination: null,
				}))
			},
		)
		mockMutation('patch', TASK_PATH, () => ({
			data: { task: { id: 'task-1' } },
			pagination: null,
		}))
		mockMutation('patch', ASSIGNMENT_REPORT_RESOLUTION_PATH, () => ({
			data: { ...PENDING_REPORT, resolution: 'APPLIED' },
			pagination: null,
		}))

		await screen.findByText(/prévu : 4 h 00/i)
		await user.click(screen.getByRole('button', { name: /^appliquer$/i }))

		// 07:00 UTC + 300 minutes (5h) in Europe/Paris (UTC+2 in August) = 14:00.
		await waitFor(() =>
			expect(
				screen.getByRole('combobox', { name: 'Heure de fin' }).textContent,
			).toBe('14:00'),
		)

		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => expect(resolutionCalls(calls)).toHaveLength(1))
		const resolveCall = resolutionCalls(calls)[0]
		expect(resolveCall.params).toMatchObject({
			path: { assignment_report_id: 'report-1' },
			body: { resolution: 'APPLIED' },
		})

		// The two calls landed in order: the task PATCH before the resolve.
		const taskPatchIndex = calls.findIndex(
			(c) => c.method === 'patch' && c.path === TASK_PATH,
		)
		const resolveIndex = calls.findIndex(
			(c) =>
				c.method === 'patch' && c.path === ASSIGNMENT_REPORT_RESOLUTION_PATH,
		)
		expect(taskPatchIndex).toBeGreaterThanOrEqual(0)
		expect(taskPatchIndex).toBeLessThan(resolveIndex)
	})

	/** The failure mode the issue explicitly designs against: a report must
	 * never read as applied against a task that never moved — asserted here
	 * from the other side, a task PATCH failure must never even attempt the
	 * resolve call. */
	it('never resolves the report when the task PATCH itself fails', async () => {
		const user = userEvent.setup()
		const { calls, mockMutation } = renderFeature(
			{ target: editTarget() },
			(api) => {
				mockEditTaskWithAssignment(api.mockGet)
				api.mockGet(ASSIGNMENT_REPORTS_PATH, () => ({
					data: [PENDING_REPORT],
					pagination: null,
				}))
			},
		)
		mockMutation('patch', TASK_PATH, () => {
			throw new Error('Conflict: boom')
		})
		mockMutation('patch', ASSIGNMENT_REPORT_RESOLUTION_PATH, () => ({
			data: { ...PENDING_REPORT, resolution: 'APPLIED' },
			pagination: null,
		}))

		await screen.findByText(/prévu : 4 h 00/i)
		await user.click(screen.getByRole('button', { name: /^appliquer$/i }))
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() =>
			expect(
				calls.some((c) => c.method === 'patch' && c.path === TASK_PATH),
			).toBe(true),
		)
		expect(resolutionCalls(calls)).toHaveLength(0)
	})

	it('dismissing sends the note and resolves as dismissed, without touching the task', async () => {
		const user = userEvent.setup()
		const { calls, mockMutation } = renderFeature(
			{ target: editTarget() },
			(api) => {
				mockEditTaskWithAssignment(api.mockGet)
				api.mockGet(ASSIGNMENT_REPORTS_PATH, () => ({
					data: [PENDING_REPORT],
					pagination: null,
				}))
			},
		)
		mockMutation('patch', ASSIGNMENT_REPORT_RESOLUTION_PATH, () => ({
			data: { ...PENDING_REPORT, resolution: 'DISMISSED' },
			pagination: null,
		}))

		await screen.findByText(/prévu : 4 h 00/i)
		await user.click(screen.getByRole('button', { name: /^rejeter$/i }))
		await user.type(
			screen.getByPlaceholderText(/note pour le déclarant/i),
			'Déjà couvert par un avenant',
		)
		await user.click(
			screen.getByRole('button', { name: /confirmer le rejet/i }),
		)

		await waitFor(() => expect(resolutionCalls(calls)).toHaveLength(1))
		expect(resolutionCalls(calls)[0].params).toMatchObject({
			path: { assignment_report_id: 'report-1' },
			body: {
				resolution: 'DISMISSED',
				resolution_note: 'Déjà couvert par un avenant',
			},
		})
		expect(
			calls.some((c) => c.method === 'patch' && c.path === TASK_PATH),
		).toBe(false)
	})
})
