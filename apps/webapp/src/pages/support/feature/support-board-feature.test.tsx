import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import {
	fireEvent,
	render,
	screen,
	waitFor,
	within,
} from '@testing-library/react'
import { type ReactNode, useState } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
	useActiveOrganization,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { PERMISSION_CATALOG } from '#/lib/permission-catalog'
import { SupportBoardFeature } from '#/pages/support/feature/support-board-feature'
import { EMPTY_SUPPORT_FILTERS } from '#/pages/support/lib/filters'
import type { SupportFilters } from '#/pages/support/types'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

vi.mock('#/hooks/use-active-organization', async (importOriginal) => {
	const actual =
		await importOriginal<typeof import('#/hooks/use-active-organization')>()
	return {
		...actual,
		useActiveOrganization: vi.fn(actual.useActiveOrganization),
	}
})

const TICKETS_PATH = '/api/v1/organizations/{organization_id}/support/tickets'
const TICKET_STATUS_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/status'
const SETTINGS_PATH = '/api/v1/organizations/{organization_id}/support/settings'
const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const ALL_PERMISSIONS = PERMISSION_CATALOG.map((entry) => entry.name)

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	slug: 'atelier-bois',
	field_clock_enabled: false,
	vat_on_debits: false,
	mcp_enabled: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const MEMBERS = [
	{
		id: 'member-1',
		organization_id: 'org-1',
		display_name: 'Alix Martin',
		last_name: 'Martin',
		first_name: 'Alix',
		created_at: '2026-01-01T00:00:00Z',
	},
	{
		id: 'member-2',
		organization_id: 'org-1',
		display_name: 'Jo Durand',
		last_name: 'Durand',
		first_name: 'Jo',
		created_at: '2026-01-01T00:00:00Z',
	},
]

function ticket(
	overrides: Partial<Schemas.TicketResponse> = {},
): Schemas.TicketResponse {
	return {
		id: 'ticket-1',
		number: 1,
		subject: 'Fuite sous évier',
		body: 'Bonjour',
		status: 'new',
		assignee_member_id: null,
		assigned_at: null,
		custom_fields: {},
		badges: [],
		external_id: null,
		created_at: '2026-10-01T07:00:00Z',
		updated_at: '2026-10-01T07:00:00Z',
		...overrides,
	}
}

function serverTickets(): Schemas.TicketResponse[] {
	return [
		ticket({
			id: 'ticket-1',
			number: 1,
			subject: 'Fuite sous évier',
			status: 'new',
			assignee_member_id: 'member-1',
			custom_fields: { ville: 'Lyon' },
		}),
		ticket({
			id: 'ticket-2',
			number: 2,
			subject: 'Devis salle de bain',
			status: 'new',
		}),
		ticket({
			id: 'ticket-3',
			number: 3,
			subject: 'Chauffe-eau en panne',
			status: 'in_progress',
			assignee_member_id: 'member-9',
		}),
		ticket({
			id: 'ticket-4',
			number: 4,
			subject: 'Robinet qui goutte',
			status: 'resolved',
		}),
	]
}

function page(tickets: Schemas.TicketResponse[], total?: number) {
	return {
		data: tickets,
		pagination: {
			current_page: 1,
			first_page: 1,
			is_empty: tickets.length === 0,
			per_page: 100,
			total: total ?? tickets.length,
		},
	}
}

type Handler = (params: unknown) => unknown

interface RecordedCall {
	method: string
	path: string
	params: unknown
}

function installFakeTanstackApi() {
	const calls: RecordedCall[] = []
	const getHandlers = new Map<string, Handler>()
	const mutationHandlers = new Map<string, Handler>()

	const fakeApi = {
		get(path: string, params: unknown) {
			const request = (params ?? {}) as { path?: unknown; query?: unknown }
			const queryKey = [{ _id: path, path: request.path, query: request.query }]
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

	Object.assign(window, { tanstackApi: fakeApi })

	return {
		calls,
		mockGet(path: string, handler: Handler) {
			getHandlers.set(path, handler)
		},
		mockMutation(method: string, path: string, handler: Handler) {
			mutationHandlers.set(`${method}:${path}`, handler)
		},
	}
}

type FakeApi = ReturnType<typeof installFakeTanstackApi>

interface RenderOptions {
	permissions?: string[]
	filters?: SupportFilters
	displayFields?: string[]
	configure?: (api: FakeApi) => void
	ready?: RegExp | null
}

function Harness({
	initialFilters,
	onFiltersChange,
}: {
	initialFilters: SupportFilters
	onFiltersChange: (filters: SupportFilters) => void
}) {
	const [filters, setFilters] = useState(initialFilters)

	return (
		<SupportBoardFeature
			filters={filters}
			onFiltersChange={(next) => {
				onFiltersChange(next)
				setFilters(next)
			}}
		/>
	)
}

function Providers({
	queryClient,
	children,
}: {
	queryClient: QueryClient
	children: ReactNode
}) {
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

async function renderBoard({
	permissions = ALL_PERMISSIONS,
	filters = EMPTY_SUPPORT_FILTERS,
	displayFields = ['ville'],
	configure,
	ready = /Fuite sous évier/,
}: RenderOptions = {}) {
	const api = installFakeTanstackApi()
	let tickets = serverTickets()
	api.mockGet(TICKETS_PATH, () => page(tickets))
	api.mockGet(SETTINGS_PATH, () => ({
		data: { display_fields: displayFields },
	}))
	api.mockGet(MEMBERS_PATH, () => ({ data: MEMBERS, pagination: null }))
	api.mockMutation('patch', TICKET_STATUS_PATH, (params) => {
		const { path, body } = params as {
			path: { ticket_id: string }
			body: { status: Schemas.TicketResponse['status'] }
		}
		tickets = tickets.map((entry) =>
			entry.id === path.ticket_id ? { ...entry, status: body.status } : entry,
		)
		return { data: tickets.find((entry) => entry.id === path.ticket_id) }
	})
	configure?.(api)

	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCacheForOrganization(queryClient, ORGANIZATION.id, permissions)

	const filterChanges: SupportFilters[] = []
	render(
		<Providers queryClient={queryClient}>
			<Harness
				initialFilters={filters}
				onFiltersChange={(next) => filterChanges.push(next)}
			/>
		</Providers>,
	)

	if (ready) {
		await screen.findByRole('article', { name: ready })
	} else {
		await screen.findByText('Aucun ticket')
	}

	return { ...api, filterChanges }
}

function column(label: string): HTMLElement {
	return screen.getByRole('region', { name: `Colonne ${label}` })
}

function card(subject: string): HTMLElement {
	return screen.getByRole('article', { name: new RegExp(subject) })
}

function handle(subject: string): HTMLElement {
	return within(card(subject)).getByRole('button', {
		name: `Déplacer ${subject}`,
	})
}

function subjectsIn(label: string): string[] {
	return within(column(label))
		.queryAllByRole('article')
		.map((element) => element.getAttribute('aria-label')?.split(',')[0] ?? '')
}

function patchCalls(api: FakeApi) {
	return api.calls.filter(
		(call) => call.method === 'patch' && call.path === TICKET_STATUS_PATH,
	)
}

function ticketQueries(api: FakeApi) {
	return api.calls
		.filter((call) => call.method === 'get' && call.path === TICKETS_PATH)
		.map((call) => (call.params as { query?: unknown }).query)
}

function drag(from: HTMLElement, to: HTMLElement) {
	fireEvent.dragStart(from)
	fireEvent.dragOver(to)
	fireEvent.drop(to)
	fireEvent.dragEnd(from)
}

interface Deferred<T> {
	promise: Promise<T>
	resolve: (value: T) => void
	reject: (reason: unknown) => void
}

function deferred<T>(): Deferred<T> {
	let resolve: (value: T) => void = () => {}
	let reject: (reason: unknown) => void = () => {}
	const promise = new Promise<T>((resolvePromise, rejectPromise) => {
		resolve = resolvePromise
		reject = rejectPromise
	})
	return { promise, resolve, reject }
}

describe('SupportBoardFeature — rendu', () => {
	it('groups the tickets into the four columns, in order', async () => {
		await renderBoard()

		expect(
			screen
				.getAllByRole('region')
				.map((region) => region.getAttribute('aria-label')),
		).toEqual([
			'Colonne Nouveau',
			'Colonne En cours',
			'Colonne En attente du client',
			'Colonne Résolu',
		])
		expect(subjectsIn('Nouveau')).toEqual([
			'Fuite sous évier',
			'Devis salle de bain',
		])
		expect(subjectsIn('En cours')).toEqual(['Chauffe-eau en panne'])
		expect(subjectsIn('En attente du client')).toEqual([])
		expect(subjectsIn('Résolu')).toEqual(['Robinet qui goutte'])
		expect(
			within(column('En attente du client')).getByText('Aucun ticket'),
		).toBeDefined()
	})

	it('names the assignee from the members and flags a stranger', async () => {
		await renderBoard()

		expect(
			within(card('Fuite sous évier')).getByText('Alix Martin'),
		).toBeDefined()
		expect(
			within(card('Devis salle de bain')).getByText('Non assigné'),
		).toBeDefined()
		expect(
			within(card('Chauffe-eau en panne')).getByText('Membre inconnu'),
		).toBeDefined()
	})

	it('shows the display fields of the settings that the ticket has a value for', async () => {
		await renderBoard()

		const withValue = within(card('Fuite sous évier'))
		expect(withValue.getByText('ville')).toBeDefined()
		expect(withValue.getByText('Lyon')).toBeDefined()
		expect(within(card('Devis salle de bain')).queryByText('ville')).toBeNull()
	})

	it('says nothing is truncated when the listing is complete', async () => {
		await renderBoard()

		expect(screen.queryByText(/tickets affichés sur/)).toBeNull()
	})

	it('says so when the listing was cut short', async () => {
		await renderBoard({
			configure: (api) =>
				api.mockGet(TICKETS_PATH, () => page(serverTickets(), 240)),
		})

		expect(
			screen.getByText(
				'4 tickets affichés sur 240. Affinez les filtres pour voir les autres.',
			),
		).toBeDefined()
	})
})

describe('SupportBoardFeature — permission', () => {
	it('offers no move handle without MANAGE_SUPPORT', async () => {
		await renderBoard({ permissions: ['VIEW_SUPPORT'] })

		expect(screen.queryAllByRole('button', { name: /^Déplacer / })).toEqual([])
		expect(screen.getAllByRole('article')).toHaveLength(4)
	})

	it('offers one handle per ticket with MANAGE_SUPPORT', async () => {
		await renderBoard({ permissions: ['MANAGE_SUPPORT'] })

		expect(screen.getAllByRole('button', { name: /^Déplacer / })).toHaveLength(
			4,
		)
	})
})

describe('SupportBoardFeature — clavier', () => {
	it('moves a ticket one column over at once and sends status alone', async () => {
		const patch = deferred<unknown>()
		const api = await renderBoard({
			configure: (fakeApi) =>
				fakeApi.mockMutation('patch', TICKET_STATUS_PATH, () => patch.promise),
		})
		const moved = handle('Fuite sous évier')
		moved.focus()

		fireEvent.keyDown(moved, { key: 'ArrowRight' })

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		expect(patchCalls(api)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { status: 'in_progress' },
		})
		await waitFor(() =>
			expect(subjectsIn('En cours')).toEqual([
				'Fuite sous évier',
				'Chauffe-eau en panne',
			]),
		)
		expect(subjectsIn('Nouveau')).toEqual(['Devis salle de bain'])
		expect(
			screen.getByText('Fuite sous évier déplacé vers En cours'),
		).toBeDefined()

		patch.resolve({ data: ticket({ id: 'ticket-1', status: 'in_progress' }) })
	})

	it('moves a ticket one column back', async () => {
		const api = await renderBoard()
		const moved = handle('Chauffe-eau en panne')

		fireEvent.keyDown(moved, { key: 'ArrowLeft' })

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		expect(patchCalls(api)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-3' },
			body: { status: 'new' },
		})
	})

	it('sends nothing past the first or the last column', async () => {
		const api = await renderBoard()

		fireEvent.keyDown(handle('Fuite sous évier'), { key: 'ArrowLeft' })
		fireEvent.keyDown(handle('Robinet qui goutte'), { key: 'ArrowRight' })

		await waitFor(() => expect(subjectsIn('Nouveau')).toHaveLength(2))
		expect(patchCalls(api)).toHaveLength(0)
	})
})

describe('SupportBoardFeature — glisser-déposer', () => {
	it('sends one PATCH for a drop on another column', async () => {
		const api = await renderBoard()

		drag(card('Devis salle de bain'), column('Résolu'))

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		expect(patchCalls(api)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-2' },
			body: { status: 'resolved' },
		})
		await waitFor(() =>
			expect(subjectsIn('Résolu')).toContain('Devis salle de bain'),
		)
	})

	it('sends nothing for a drop on the ticket’s own column', async () => {
		const api = await renderBoard()

		drag(card('Fuite sous évier'), column('Nouveau'))

		await waitFor(() => expect(subjectsIn('Nouveau')).toHaveLength(2))
		expect(patchCalls(api)).toHaveLength(0)
	})
})

describe('SupportBoardFeature — rollback', () => {
	it('puts a rejected ticket back in its column and shows the error', async () => {
		const patch = deferred<unknown>()
		const refetch = deferred<unknown>()
		let reads = 0

		await renderBoard({
			configure: (api) => {
				api.mockGet(TICKETS_PATH, () => {
					reads += 1
					return reads === 1 ? page(serverTickets()) : refetch.promise
				})
				api.mockMutation('patch', TICKET_STATUS_PATH, () => patch.promise)
			},
		})

		fireEvent.keyDown(handle('Fuite sous évier'), { key: 'ArrowRight' })
		await waitFor(() =>
			expect(subjectsIn('En cours')).toEqual([
				'Fuite sous évier',
				'Chauffe-eau en panne',
			]),
		)

		patch.reject(new Error('Le serveur a refusé le déplacement.'))

		await waitFor(() =>
			expect(subjectsIn('Nouveau')).toEqual([
				'Fuite sous évier',
				'Devis salle de bain',
			]),
		)
		expect(subjectsIn('En cours')).toEqual(['Chauffe-eau en panne'])

		refetch.resolve(page(serverTickets()))
		expect(
			await screen.findByText('Le serveur a refusé le déplacement.'),
		).toBeDefined()
	})
})

describe('SupportBoardFeature — filtres', () => {
	it('sends the assignee and the badge to the API', async () => {
		const api = await renderBoard({
			filters: {
				...EMPTY_SUPPORT_FILTERS,
				assignee: 'member-1',
				badge: 'Urgent',
			},
		})

		expect(ticketQueries(api)).toEqual([
			{ per_page: 100, assignee: 'member-1', badge: 'Urgent' },
		])
	})

	it('sends the custom field pair when both halves are set', async () => {
		const api = await renderBoard({
			filters: {
				...EMPTY_SUPPORT_FILTERS,
				custom_field_key: 'ville',
				custom_field_value: 'Lyon',
			},
		})

		expect(ticketQueries(api)).toEqual([
			{ per_page: 100, custom_field_key: 'ville', custom_field_value: 'Lyon' },
		])
	})

	it('never sends half of the custom field pair', async () => {
		const api = await renderBoard({
			filters: { ...EMPTY_SUPPORT_FILTERS, custom_field_key: 'ville' },
		})

		expect(ticketQueries(api)).toEqual([{ per_page: 100 }])
	})

	it('keeps a field key from a shared link selectable when it is no longer displayed', async () => {
		await renderBoard({
			filters: { ...EMPTY_SUPPORT_FILTERS, custom_field_key: 'ancien_champ' },
		})

		const select = screen.getByLabelText('Champ')
		expect(
			within(select).getByRole('option', { name: 'ancien_champ' }),
		).toBeDefined()
		expect(within(select).getByRole('option', { name: 'ville' })).toBeDefined()
	})

	it('writes a picked assignee back and asks the API again', async () => {
		const api = await renderBoard()
		await screen.findByRole('option', { name: 'Alix Martin' })

		fireEvent.change(screen.getByLabelText('Assigné à'), {
			target: { value: 'member-1' },
		})

		expect(api.filterChanges.at(-1)).toEqual({
			...EMPTY_SUPPORT_FILTERS,
			assignee: 'member-1',
		})
		await waitFor(() =>
			expect(ticketQueries(api).at(-1)).toEqual({
				per_page: 100,
				assignee: 'member-1',
			}),
		)
	})

	it('offers the badges carried by the listed tickets', async () => {
		await renderBoard({
			configure: (api) =>
				api.mockGet(TICKETS_PATH, () =>
					page([
						ticket({ badges: [{ label: 'Urgent', tone: 'error' }] }),
						ticket({
							id: 'ticket-2',
							subject: 'Autre demande',
							badges: [{ label: 'Garantie', tone: 'success' }],
						}),
					]),
				),
		})

		const select = screen.getByLabelText('Badge')
		expect(
			within(select)
				.getAllByRole('option')
				.map((option) => option.textContent),
		).toEqual(['Tous les badges', 'Garantie', 'Urgent'])
	})

	it('clears every filter at once', async () => {
		const api = await renderBoard({
			filters: { ...EMPTY_SUPPORT_FILTERS, badge: 'Urgent' },
		})

		fireEvent.click(screen.getByRole('button', { name: 'Effacer les filtres' }))

		expect(api.filterChanges.at(-1)).toStrictEqual(EMPTY_SUPPORT_FILTERS)
	})

	it('explains an empty board that a filter emptied', async () => {
		await renderBoard({
			filters: { ...EMPTY_SUPPORT_FILTERS, badge: 'Urgent' },
			ready: null,
			configure: (api) => api.mockGet(TICKETS_PATH, () => page([])),
		})

		expect(
			screen.getByText('Aucun ticket ne correspond aux filtres appliqués.'),
		).toBeDefined()
	})
})

describe('SupportBoardFeature — organisation', () => {
	afterEach(() => {
		vi.mocked(useActiveOrganization).mockReset()
	})

	it('shows the unavailable block without an active organization', () => {
		const api = installFakeTanstackApi()
		vi.mocked(useActiveOrganization).mockReturnValue({
			organizations: [],
			activeOrganization: null,
			activeOrganizationId: '',
		} as never)

		render(
			<QueryClientProvider client={new QueryClient()}>
				<SupportBoardFeature
					filters={EMPTY_SUPPORT_FILTERS}
					onFiltersChange={() => {}}
				/>
			</QueryClientProvider>,
		)

		expect(screen.getByText('Organisation indisponible')).toBeDefined()
		expect(
			screen.getByText('Le support nécessite une organisation active.'),
		).toBeDefined()
		expect(api.calls).toEqual([])
	})
})
