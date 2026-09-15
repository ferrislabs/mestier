import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import {
	fireEvent,
	render,
	screen,
	waitFor,
	within,
} from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { PERMISSION_CATALOG } from '#/lib/permission-catalog'
import { BoardFeature } from '#/pages/planification/feature/board-feature'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

// jsdom has neither ResizeObserver nor pointer capture, which the Radix
// primitives behind the date chip probe defensively — the same stubs as
// `planning/feature/task-sheet-feature.test.tsx`.
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
const PLANNING_PATH = '/api/v1/organizations/{organization_id}/planning'
const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'
const ALL_PERMISSIONS = PERMISSION_CATALOG.map((entry) => entry.name)

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	slug: 'atelier-bois',
	field_clock_enabled: false,
	vat_on_debits: false,
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

interface TaskOverrides {
	id?: string
	title?: string
	status?: string
	starts_at?: string | null
	ends_at?: string | null
	all_day?: boolean
}

function task(overrides: TaskOverrides = {}) {
	return {
		id: 'task-1',
		organization_id: 'org-1',
		title: 'Projet toiture',
		description: null,
		status: 'BACKLOG' as const,
		all_day: false,
		blocks_availability: true,
		parent_task_id: null,
		child_count: 0,
		customer_id: null,
		customer_context_id: null,
		quote_id: null,
		starts_at: null,
		ends_at: null,
		member_ids: ['member-1'],
		labels: [],
		assignments: [],
		equipment: [],
		expenses_cents: 0,
		expenses_label: null,
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}

/** Backlog: a, b, c. En cours: x. */
function boardTasks() {
	return [
		task({ id: 'a', title: 'Commander le bois', status: 'BACKLOG' }),
		task({
			id: 'b',
			title: 'Poser les tuiles',
			status: 'BACKLOG',
			starts_at: '2026-09-09T22:00:00.000Z',
			ends_at: '2026-09-10T22:00:00.000Z',
			all_day: true,
		}),
		task({ id: 'c', title: 'Nettoyer le chantier', status: 'BACKLOG' }),
		task({ id: 'x', title: 'Monter la charpente', status: 'IN_PROGRESS' }),
	]
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

function tasksHandler(tasks: ReturnType<typeof task>[]) {
	return () => ({
		data: tasks,
		pagination: {
			current_page: 1,
			first_page: 1,
			is_empty: tasks.length === 0,
			last_page: 1,
			next_page: null,
			per_page: 200,
			prev_page: null,
			total: tasks.length,
		},
	})
}

type ServerTask = ReturnType<typeof task>

/**
 * What the API does with a board `PATCH`, written out here rather than
 * reusing `lib/board.ts` so the screen is checked against an independent
 * placement, not against its own helper. Deliberately literal: a body with
 * no `status` and no neighbour moves nothing, which is the rule the "a date
 * never changes the column" criterion rests on.
 */
function applyOnServer(
	tasks: ServerTask[],
	taskId: string,
	body: Record<string, unknown>,
): ServerTask[] {
	const current = tasks.find((entry) => entry.id === taskId)
	if (!current) return tasks

	const patched = { ...current }
	if (body.status) patched.status = body.status as ServerTask['status']
	if ('starts_at' in body) patched.starts_at = body.starts_at as string | null
	if ('ends_at' in body) patched.ends_at = body.ends_at as string | null

	const moves =
		Boolean(body.status) ||
		Boolean(body.preceding_task_id) ||
		Boolean(body.following_task_id)
	if (!moves) {
		return tasks.map((entry) => (entry.id === taskId ? patched : entry))
	}

	const others = tasks.filter((entry) => entry.id !== taskId)
	let index = others.length
	if (body.preceding_task_id) {
		index = others.findIndex((e) => e.id === body.preceding_task_id) + 1
	} else if (body.following_task_id) {
		index = others.findIndex((e) => e.id === body.following_task_id)
	} else {
		for (let at = others.length - 1; at >= 0; at -= 1) {
			if (others[at].status === patched.status) {
				index = at + 1
				break
			}
		}
	}

	const next = [...others]
	next.splice(index, 0, patched)
	return next
}

interface RenderOptions {
	permissions?: string[]
}

type BoardApi = ReturnType<typeof installFakeTanstackApi> & {
	seedTasks: (tasks: ServerTask[]) => void
}

async function renderBoard(
	configure: (api: BoardApi) => void,
	{ permissions = ALL_PERMISSIONS }: RenderOptions = {},
) {
	const api = installFakeTanstackApi()
	api.mockGet(MY_PERMISSIONS_PATH, () => ({
		data: { permissions },
		pagination: null,
	}))
	api.mockGet(PLANNING_PATH, () => ({
		data: {
			timezone: 'Europe/Paris',
			resources: [RESOURCE_MEMBER],
			entries: [],
			work_time: [],
		},
		pagination: null,
	}))
	let serverTasks = boardTasks()
	api.mockGet(TASKS_PATH, () => tasksHandler(serverTasks)())
	api.mockMutation('patch', TASK_PATH, (params) => {
		const { path, body } = params as {
			path: { task_id: string }
			body: Record<string, unknown>
		}
		serverTasks = applyOnServer(serverTasks, path.task_id, body)
		return {
			data: {
				detached: false,
				task: serverTasks.find((entry) => entry.id === path.task_id),
			},
			pagination: null,
		}
	})
	configure({
		...api,
		seedTasks: (tasks: ServerTask[]) => {
			serverTasks = tasks
		},
	})

	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCacheForOrganization(queryClient, ORGANIZATION.id, permissions)

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
			<BoardFeature />
		</Providers>,
	)

	await screen.findByRole('article', { name: /Commander le bois/ })

	return api
}

function column(label: string): HTMLElement {
	return screen.getByRole('region', { name: `Colonne ${label}` })
}

function card(title: string): HTMLElement {
	return screen.getByRole('article', { name: new RegExp(title) })
}

/** The card's keyboard affordance: a real button, so the arrow keys hang off something interactive. */
function handle(title: string): HTMLElement {
	return within(card(title)).getByRole('button', { name: `Déplacer ${title}` })
}

function cardTitlesIn(label: string): string[] {
	return within(column(label))
		.queryAllByRole('article')
		.map((element) => element.getAttribute('aria-label')?.split(',')[0] ?? '')
}

function patchCalls(api: ReturnType<typeof installFakeTanstackApi>) {
	return api.calls.filter(
		(call) => call.method === 'patch' && call.path === TASK_PATH,
	)
}

function patchBody(call: { params: unknown }) {
	return (call.params as { body: Record<string, unknown> }).body
}

function drag(from: HTMLElement, to: HTMLElement) {
	fireEvent.dragStart(from)
	fireEvent.dragOver(to)
	fireEvent.drop(to)
	fireEvent.dragEnd(from)
}

describe('BoardFeature — rendu', () => {
	it('groups the cards into the five columns, in order', async () => {
		await renderBoard(() => {})

		expect(
			screen
				.getAllByRole('region')
				.map((region) => region.getAttribute('aria-label')),
		).toEqual([
			'Colonne Backlog',
			'Colonne À planifier',
			'Colonne En cours',
			'Colonne Terminé',
			'Colonne Annulé',
		])
		expect(cardTitlesIn('Backlog')).toEqual([
			'Commander le bois',
			'Poser les tuiles',
			'Nettoyer le chantier',
		])
		expect(cardTitlesIn('En cours')).toEqual(['Monter la charpente'])
	})

	it('spells out an empty column rather than leaving a bare frame', async () => {
		await renderBoard(() => {})

		expect(within(column('Terminé')).getByText('Aucune carte')).toBeDefined()
		expect(within(column('Backlog')).queryByText('Aucune carte')).toBeNull()
	})
})

describe('BoardFeature — tableau vide', () => {
	it('shows an empty state and no columns at all', async () => {
		const api = installFakeTanstackApi()
		api.mockGet(MY_PERMISSIONS_PATH, () => ({
			data: { permissions: ALL_PERMISSIONS },
			pagination: null,
		}))
		api.mockGet(PLANNING_PATH, () => ({
			data: {
				timezone: 'Europe/Paris',
				resources: [],
				entries: [],
				work_time: [],
			},
			pagination: null,
		}))
		api.mockGet(TASKS_PATH, tasksHandler([]))

		const queryClient = new QueryClient({
			defaultOptions: { queries: { retry: false } },
		})
		seedPermissionsCacheForOrganization(queryClient, ORGANIZATION.id)

		render(
			<QueryClientProvider client={queryClient}>
				<OrganizationListProvider organizations={[ORGANIZATION]}>
					<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
						<BoardFeature />
					</ActiveOrganizationProvider>
				</OrganizationListProvider>
			</QueryClientProvider>,
		)

		expect(await screen.findByText('Aucune tâche')).toBeDefined()
		expect(screen.queryByRole('region', { name: 'Colonne Backlog' })).toBeNull()
	})
})

describe('BoardFeature — glisser-déposer', () => {
	/**
	 * #466's first invariant. Status, dates and rank are three independent
	 * axes: a card crossing a column carries its column and, when the drop
	 * chose a slot, its neighbours — never a date.
	 */
	it('a cross-column drop onto the column sends one PATCH carrying status, and leaves the dates alone', async () => {
		const api = await renderBoard(() => {})

		drag(card('Poser les tuiles'), column('En cours'))

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		const call = patchCalls(api)[0]
		expect((call.params as { path: { task_id: string } }).path.task_id).toBe(
			'b',
		)
		expect(patchBody(call)).toEqual({ status: 'IN_PROGRESS' })
		expect(patchBody(call).starts_at).toBeUndefined()
		expect(patchBody(call).ends_at).toBeUndefined()

		// And the window the card displays is the one it had before the drag.
		await waitFor(() =>
			expect(cardTitlesIn('En cours')).toContain('Poser les tuiles'),
		)
		expect(
			within(card('Poser les tuiles')).getByRole('button', {
				name: /^Dates de Poser les tuiles/,
			}).textContent,
		).toContain('10/09/2026')
	})

	it('a cross-column drop onto a card sends status and the target column’s neighbours, together', async () => {
		const api = await renderBoard((apis) => {
			apis.seedTasks([
				...boardTasks(),
				task({ id: 'y', title: 'Poser la gouttière', status: 'IN_PROGRESS' }),
			])
		})

		drag(card('Commander le bois'), card('Poser la gouttière'))

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		expect(patchBody(patchCalls(api)[0])).toEqual({
			status: 'IN_PROGRESS',
			preceding_task_id: 'x',
			following_task_id: 'y',
		})
	})

	it('a reorder inside a column sends the neighbour ids only', async () => {
		const api = await renderBoard(() => {})

		drag(card('Nettoyer le chantier'), card('Poser les tuiles'))

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		const body = patchBody(patchCalls(api)[0])
		expect(body).toEqual({ preceding_task_id: 'a', following_task_id: 'b' })
		expect(body.status).toBeUndefined()
		expect(body.starts_at).toBeUndefined()
	})

	it('sends nothing for a drop that changes nothing', async () => {
		const api = await renderBoard(() => {})

		drag(card('Commander le bois'), column('Backlog'))

		await waitFor(() => expect(cardTitlesIn('Backlog')).toHaveLength(3))
		expect(patchCalls(api)).toHaveLength(0)
	})
})

describe('BoardFeature — rollback', () => {
	it('puts a rejected card back in its column and its position', async () => {
		let rejectPatch: ((error: Error) => void) | null = null
		let served = 0

		await renderBoard((apis) => {
			apis.mockGet(TASKS_PATH, () => {
				served += 1
				// Only the first read succeeds: a refetch that re-served the
				// server's order would hide whether the rollback happened at all.
				if (served > 1) throw new Error('indisponible')
				return tasksHandler(boardTasks())()
			})
			apis.mockMutation(
				'patch',
				TASK_PATH,
				() =>
					new Promise((_resolve, reject) => {
						rejectPatch = reject
					}),
			)
		})

		drag(card('Nettoyer le chantier'), card('Monter la charpente'))

		// The optimistic move lands first: the card is in En cours while the
		// request is still in flight.
		await waitFor(() =>
			expect(cardTitlesIn('En cours')).toEqual([
				'Nettoyer le chantier',
				'Monter la charpente',
			]),
		)
		expect(cardTitlesIn('Backlog')).toEqual([
			'Commander le bois',
			'Poser les tuiles',
		])

		rejectPatch?.(new Error('Le serveur a refusé le déplacement.'))

		await waitFor(() =>
			expect(cardTitlesIn('Backlog')).toEqual([
				'Commander le bois',
				'Poser les tuiles',
				'Nettoyer le chantier',
			]),
		)
		expect(cardTitlesIn('En cours')).toEqual(['Monter la charpente'])
	})
})

describe('BoardFeature — clavier', () => {
	it('moves a card one column to the right, appended, with status alone', async () => {
		const api = await renderBoard(() => {})
		const moved = handle('Commander le bois')
		moved.focus()

		fireEvent.keyDown(moved, { key: 'ArrowRight' })

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		expect(patchBody(patchCalls(api)[0])).toEqual({ status: 'PLANNED' })
		await waitFor(() =>
			expect(cardTitlesIn('À planifier')).toEqual(['Commander le bois']),
		)
	})

	it('moves a card one column to the left', async () => {
		const api = await renderBoard(() => {})
		const moved = handle('Monter la charpente')
		moved.focus()

		fireEvent.keyDown(moved, { key: 'ArrowLeft' })

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		expect(patchBody(patchCalls(api)[0])).toEqual({ status: 'PLANNED' })
	})

	it('reorders inside the column with the neighbour ids only', async () => {
		const api = await renderBoard(() => {})
		const moved = handle('Commander le bois')
		moved.focus()

		fireEvent.keyDown(moved, { key: 'ArrowDown' })

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		expect(patchBody(patchCalls(api)[0])).toEqual({
			preceding_task_id: 'b',
			following_task_id: 'c',
		})
		await waitFor(() =>
			expect(cardTitlesIn('Backlog')).toEqual([
				'Poser les tuiles',
				'Commander le bois',
				'Nettoyer le chantier',
			]),
		)
	})

	it('keeps focus on the card after it has been re-parented into another column', async () => {
		await renderBoard(() => {})
		const moved = handle('Commander le bois')
		moved.focus()

		fireEvent.keyDown(moved, { key: 'ArrowRight' })

		await waitFor(() =>
			expect(cardTitlesIn('À planifier')).toEqual(['Commander le bois']),
		)
		expect(document.activeElement).toBe(handle('Commander le bois'))
	})

	it('announces the move for screen readers', async () => {
		await renderBoard(() => {})
		const moved = handle('Commander le bois')
		moved.focus()

		fireEvent.keyDown(moved, { key: 'ArrowRight' })

		await waitFor(() =>
			expect(screen.getByRole('status').textContent).toBe(
				'Commander le bois déplacée vers À planifier',
			),
		)
	})

	it('sends nothing at the edges of the board', async () => {
		const api = await renderBoard(() => {})
		const first = handle('Commander le bois')
		first.focus()

		fireEvent.keyDown(first, { key: 'ArrowLeft' })
		fireEvent.keyDown(first, { key: 'ArrowUp' })

		await waitFor(() => expect(cardTitlesIn('Backlog')).toHaveLength(3))
		expect(patchCalls(api)).toHaveLength(0)
	})
})

describe('BoardFeature — permission', () => {
	it('neither drags nor moves a card without MANAGE_PLANNING', async () => {
		const api = await renderBoard(() => {}, {
			permissions: ALL_PERMISSIONS.filter(
				(permission) => permission !== 'MANAGE_PLANNING',
			),
		})
		const moved = card('Commander le bois')

		expect(moved.getAttribute('draggable')).toBe('false')
		expect(
			within(moved).queryByRole('button', {
				name: 'Déplacer Commander le bois',
			}),
		).toBeNull()
		fireEvent.keyDown(moved, { key: 'ArrowRight' })

		await waitFor(() => expect(cardTitlesIn('Backlog')).toHaveLength(3))
		expect(patchCalls(api)).toHaveLength(0)
	})
})

describe('BoardFeature — planifier depuis la carte', () => {
	beforeEach(() => {
		vi.useFakeTimers({ shouldAdvanceTime: true })
		vi.setSystemTime(new Date('2026-09-15T09:00:00Z'))
	})

	afterEach(() => {
		vi.useRealTimers()
	})

	/**
	 * #466's second invariant, and the one a single "save" mutation carrying
	 * both fields breaks: scheduling a Backlog card writes the window and
	 * nothing else, so the card stays exactly where it was.
	 */
	it('a date set from the card sends the window only, and the card stays in Backlog', async () => {
		const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime })
		const api = await renderBoard(() => {})

		await user.click(
			within(card('Commander le bois')).getByRole('button', {
				name: /^Dates de Commander le bois/,
			}),
		)

		// react-day-picker tags the grid cell with the ISO day and nests the
		// clickable button inside it; the button carries a locale-formatted
		// `data-day` of its own, which is not something to match on.
		const day = document.querySelector<HTMLElement>(
			'[data-day="2026-09-17"] button',
		)
		expect(day).not.toBeNull()
		await user.click(day as HTMLElement)

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		const body = patchBody(patchCalls(api)[0])
		expect(Object.keys(body).sort()).toEqual(['ends_at', 'starts_at'])
		expect(body.starts_at).toBe('2026-09-16T22:00:00.000Z')
		expect(body.ends_at).toBe('2026-09-17T22:00:00.000Z')

		expect(cardTitlesIn('Backlog')).toEqual([
			'Commander le bois',
			'Poser les tuiles',
			'Nettoyer le chantier',
		])
	})
})
