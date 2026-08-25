import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import { OrganizationListProvider } from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { FieldDayFeature } from '#/pages/field/feature/field-day-feature'

const TASKS_PATH = '/api/v1/organizations/{organization_id}/field/tasks'
const CURRENT_PATH = '/api/v1/organizations/{organization_id}/field/current'
const ENTRIES_PATH =
	'/api/v1/organizations/{organization_id}/field/time-entries'
const STOP_PATH = '/api/v1/field/time-entries/{time_entry_id}/stop'
const ASSIGNMENT_REPORTS_PATH =
	'/api/v1/organizations/{organization_id}/field/assignment-reports'
const REPORT_ASSIGNMENT_PATH =
	'/api/v1/organizations/{organization_id}/field/assignments/{task_assignment_id}/report'
const ASSIGNMENT_REPORT_PATH =
	'/api/v1/field/assignment-reports/{assignment_report_id}'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Paysages Bonnal',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	slug: 'paysages-bonnal',
	// Every existing test in this suite predates the clock demotion and
	// exercises the clock itself — see the dedicated
	// "pointeuse désactivée" describe block below for the demoted case.
	field_clock_enabled: true,
	vat_on_debits: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

type Handler = (params: unknown) => unknown

/**
 * A stand-in for `window.tanstackApi`, minimal on purpose: only `get` and
 * `mutation` are used by the field feature, and asserting through real HTTP
 * would mean asserting through `api.fetch.ts`'s error-shaping too, which is
 * exercised well enough by its own tests.
 */
function installFakeTanstackApi() {
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

	return { mockGet, mockMutation }
}

function renderFeature(
	configure: (api: ReturnType<typeof installFakeTanstackApi>) => void,
	organizationOverrides: Partial<Organization> = {},
) {
	const organization = { ...ORGANIZATION, ...organizationOverrides }
	const api = installFakeTanstackApi()
	const { mockGet } = api
	mockGet(TASKS_PATH, () => ({ data: [], pagination: null }))
	mockGet(CURRENT_PATH, () => ({
		data: { running: null, day_ended_at: null },
		pagination: null,
	}))
	mockGet(ASSIGNMENT_REPORTS_PATH, () => ({ data: [], pagination: null }))
	configure(api)

	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>
				<OrganizationListProvider organizations={[organization]}>
					{children}
				</OrganizationListProvider>
			</QueryClientProvider>
		)
	}

	render(
		<Providers>
			<FieldDayFeature organizationSlug={organization.slug} />
		</Providers>,
	)

	return api
}

function task(overrides: Record<string, unknown> = {}) {
	return {
		id: 'task-1',
		title: 'Taille de haie',
		description: null,
		starts_at: '2026-08-19T06:00:00Z',
		ends_at: '2026-08-19T14:00:00Z',
		all_day: false,
		status: 'PLANNED',
		customer_id: null,
		customer_context_id: null,
		task_assignment_id: 'assignment-1',
		...overrides,
	}
}

function assignmentReport(overrides: Record<string, unknown> = {}) {
	return {
		id: 'report-1',
		organization_id: 'org-1',
		task_assignment_id: 'assignment-1',
		reported_minutes: 300,
		comment: null,
		reported_by: 'member-1',
		resolution: 'PENDING',
		resolved_by: null,
		resolved_at: null,
		resolution_note: null,
		created_at: '2026-08-19T14:00:00Z',
		updated_at: '2026-08-19T14:00:00Z',
		...overrides,
	}
}

// Two hours ago rather than a fixed date: the feature reads `Date.now()`
// itself (there is no `now` prop to control here, unlike `FieldDayUI`'s own
// tests), and a hardcoded date would read as "from an earlier day" — the
// forgotten clock-off flow — the moment this suite outlives that date.
function entry(overrides: Record<string, unknown> = {}) {
	return {
		id: 'entry-1',
		organization_id: 'org-1',
		task_id: 'task-1',
		employee_id: 'employee-1',
		started_at: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
		ended_at: null,
		worked_minutes: null,
		photos: [],
		...overrides,
	}
}

describe('FieldDayFeature — jour déjà terminé', () => {
	/**
	 * The bug this fixes: `dayEndedAt` used to live only in React state set by
	 * the end-day mutation's answer, so a reload — which never re-runs that
	 * mutation — fell back to the clock-in form even though the day was over.
	 * `/field/current` is now the only source, so a fresh mount showing it
	 * already set must render the closed state immediately.
	 */
	it('shows the day as already over on a fresh load, straight from the server', async () => {
		renderFeature((api) =>
			api.mockGet(CURRENT_PATH, () => ({
				data: { running: null, day_ended_at: '2026-08-19T16:30:00Z' },
				pagination: null,
			})),
		)

		expect(await screen.findByText(/journée terminée à/i)).toBeDefined()
		expect(
			screen.queryByRole('button', { name: /terminer ma journée/i }),
		).toBeNull()
	})

	it('offers to end the day when the server says it has not been declared over', async () => {
		renderFeature(() => {})

		expect(
			await screen.findByRole('button', { name: /terminer ma journée/i }),
		).toBeDefined()
	})
})

describe('FieldDayFeature — messages d’erreur', () => {
	/**
	 * The backend keeps its conflict messages in English, and `ApiError` wraps
	 * them as `"Conflict: {message}"` on the wire. The screen must show the
	 * French mapping, never that raw string.
	 */
	it('translates a known conflict message instead of showing the raw English', async () => {
		renderFeature((api) => {
			api.mockGet(CURRENT_PATH, () => ({
				data: { running: entry(), day_ended_at: null },
				pagination: null,
			}))
			api.mockMutation('post', STOP_PATH, () => {
				throw new Error('Conflict: time entry is already closed')
			})
		})

		const stopButton = await screen.findByRole('button', {
			name: /clôturer ce projet/i,
		})
		await userEvent.click(stopButton)

		expect(
			await screen.findByText(/ce pointage est déjà clôturé/i),
		).toBeDefined()
		expect(screen.queryByText(/already closed/i)).toBeNull()
	})

	it('falls back to a generic French message for an unmapped error', async () => {
		renderFeature((api) => {
			api.mockGet(CURRENT_PATH, () => ({
				data: { running: entry(), day_ended_at: null },
				pagination: null,
			}))
			api.mockMutation('post', STOP_PATH, () => {
				throw new Error('Conflict: something entirely new')
			})
		})

		await userEvent.click(
			await screen.findByRole('button', { name: /clôturer ce projet/i }),
		)

		expect(
			await screen.findByText(/une erreur est survenue, réessayez/i),
		).toBeDefined()
	})
})

describe('FieldDayFeature — bascule entre projets', () => {
	/**
	 * Switching jobs stops the running entry, then starts the new one. If the
	 * start half fails after the stop half already went through, the worker is
	 * left clocked on to nothing, which the generic "start failed" message
	 * would not convey.
	 */
	it('warns distinctly when a job switch leaves nothing running', async () => {
		renderFeature((api) => {
			api.mockGet(TASKS_PATH, () => ({
				data: [task(), task({ id: 'task-2', title: 'Tonte du parc' })],
				pagination: null,
			}))
			api.mockGet(CURRENT_PATH, () => ({
				data: { running: entry(), day_ended_at: null },
				pagination: null,
			}))
			api.mockMutation('post', STOP_PATH, () => ({
				data: entry(),
				pagination: null,
			}))
			api.mockMutation('post', ENTRIES_PATH, () => {
				throw new Error('Conflict: employee is already clocked on to task 99')
			})
		})

		const switchButton = await screen.findByRole('button', {
			name: /basculer sur ce projet/i,
		})
		await userEvent.click(switchButton)

		expect(
			await screen.findByText(/vous êtes maintenant décroché de tout projet/i),
		).toBeDefined()
	})
})

describe('FieldDayFeature — échec de chargement', () => {
	/** An empty list and a failed fetch must not read the same way. */
	it('shows a retry state instead of the empty state when the task list fails to load', async () => {
		renderFeature((api) => {
			api.mockGet(TASKS_PATH, () => {
				throw new Error('network error')
			})
		})

		expect(
			await screen.findByText(/échec du chargement — réessayer/i),
		).toBeDefined()
		expect(screen.queryByText(/aucun projet ne vous est assigné/i)).toBeNull()
	})

	it('retrying the task list re-issues the request', async () => {
		let attempts = 0
		renderFeature((api) => {
			api.mockGet(TASKS_PATH, () => {
				attempts += 1
				if (attempts === 1) throw new Error('network error')
				return { data: [task()], pagination: null }
			})
		})

		await userEvent.click(
			await screen.findByRole('button', { name: /^réessayer$/i }),
		)

		await waitFor(() => expect(attempts).toBeGreaterThan(1))
		expect(await screen.findByText('Taille de haie')).toBeDefined()
	})
})

describe('FieldDayFeature — boucle de correction', () => {
	it('filing a report posts the assignment id, closes the form, and the new report appears once the server answered', async () => {
		let posted: unknown = null
		const filedReports: unknown[] = []
		renderFeature((api) => {
			api.mockGet(TASKS_PATH, () => ({ data: [task()], pagination: null }))
			api.mockGet(ASSIGNMENT_REPORTS_PATH, () => ({
				data: filedReports,
				pagination: null,
			}))
			api.mockMutation('post', REPORT_ASSIGNMENT_PATH, (params) => {
				posted = params
				const created = assignmentReport()
				filedReports.push(created)
				return { data: created, pagination: null }
			})
		})

		await userEvent.click(
			await screen.findByRole('button', { name: /signaler un écart/i }),
		)
		await userEvent.type(screen.getByLabelText(/durée réelle/i), '300')
		await userEvent.click(screen.getByRole('button', { name: /^déclarer$/i }))

		await waitFor(() => expect(posted).not.toBeNull())
		expect(posted).toMatchObject({
			path: {
				organization_id: 'org-1',
				task_assignment_id: 'assignment-1',
			},
			body: { reported_minutes: 300, comment: null },
		})
		// The form closes once the server actually answered — no optimistic
		// write, per the issue's own acceptance criterion.
		expect(screen.queryByLabelText(/durée réelle/i)).toBeNull()

		// The mutation's own `onSuccess` invalidates the reports list, which
		// refetches from the (now updated) fake backend above.
		expect(await screen.findByText(/en attente de validation/i)).toBeDefined()
	})

	it('amending a pending report pre-fills the form and PATCHes its own id', async () => {
		let patched: unknown = null
		renderFeature((api) => {
			api.mockGet(TASKS_PATH, () => ({ data: [task()], pagination: null }))
			api.mockGet(ASSIGNMENT_REPORTS_PATH, () => ({
				data: [assignmentReport({ reported_minutes: 300 })],
				pagination: null,
			}))
			api.mockMutation('patch', ASSIGNMENT_REPORT_PATH, (params) => {
				patched = params
				return {
					data: assignmentReport({ reported_minutes: 240 }),
					pagination: null,
				}
			})
		})

		const modifyButton = await screen.findByRole('button', {
			name: /^modifier$/i,
		})
		await userEvent.click(modifyButton)

		const minutesInput = screen.getByLabelText(
			/durée réelle/i,
		) as HTMLInputElement
		expect(minutesInput.value).toBe('300')

		await userEvent.clear(minutesInput)
		await userEvent.type(minutesInput, '240')
		await userEvent.click(screen.getByRole('button', { name: /enregistrer/i }))

		await waitFor(() => expect(patched).not.toBeNull())
		expect(patched).toMatchObject({
			path: { assignment_report_id: 'report-1' },
			body: { reported_minutes: 240, comment: null },
		})
	})

	it('withdrawing a pending report DELETEs it by its own id', async () => {
		let withdrawnPath: unknown = null
		renderFeature((api) => {
			api.mockGet(TASKS_PATH, () => ({ data: [task()], pagination: null }))
			api.mockGet(ASSIGNMENT_REPORTS_PATH, () => ({
				data: [assignmentReport()],
				pagination: null,
			}))
			api.mockMutation('delete', ASSIGNMENT_REPORT_PATH, (params) => {
				withdrawnPath = (params as { path?: unknown }).path
				return { data: undefined, pagination: null }
			})
		})

		await userEvent.click(
			await screen.findByRole('button', { name: /^retirer$/i }),
		)

		await waitFor(() =>
			expect(withdrawnPath).toEqual({ assignment_report_id: 'report-1' }),
		)
	})

	/** "Somebody already resolved this" — the race an amend can lose to a
	 * manager's decision landing first. */
	it('translates the already-resolved conflict when an amend loses the race', async () => {
		renderFeature((api) => {
			api.mockGet(TASKS_PATH, () => ({ data: [task()], pagination: null }))
			api.mockGet(ASSIGNMENT_REPORTS_PATH, () => ({
				data: [assignmentReport()],
				pagination: null,
			}))
			api.mockMutation('patch', ASSIGNMENT_REPORT_PATH, () => {
				throw new Error('Conflict: a resolved report can no longer be amended')
			})
		})

		await userEvent.click(
			await screen.findByRole('button', { name: /^modifier$/i }),
		)
		await userEvent.click(screen.getByRole('button', { name: /enregistrer/i }))

		expect(
			await screen.findByText(
				/déjà été traité par un responsable, vous ne pouvez plus le modifier/i,
			),
		).toBeDefined()
	})
})

describe('FieldDayFeature — pointeuse désactivée', () => {
	it('shows the tasks and no clock, and never polls /field/current', async () => {
		let currentRequests = 0
		renderFeature(
			(api) => {
				api.mockGet(TASKS_PATH, () => ({ data: [task()], pagination: null }))
				api.mockGet(CURRENT_PATH, () => {
					currentRequests += 1
					return {
						data: { running: null, day_ended_at: null },
						pagination: null,
					}
				})
			},
			{ field_clock_enabled: false },
		)

		expect(await screen.findByText('Taille de haie')).toBeDefined()
		expect(screen.queryByRole('button', { name: /^démarrer$/i })).toBeNull()
		expect(
			screen.queryByRole('button', { name: /terminer ma journée/i }),
		).toBeNull()
		// Nothing polls a running entry the screen would never show or act on.
		expect(currentRequests).toBe(0)
	})
})
