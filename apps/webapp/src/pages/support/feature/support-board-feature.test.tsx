import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import {
	fireEvent,
	render,
	screen,
	waitFor,
	within,
} from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useState } from 'react'
import {
	afterEach,
	beforeAll,
	beforeEach,
	describe,
	expect,
	it,
	vi,
} from 'vitest'
import type { Schemas } from '#/api/api.client'
import { useActiveOrganization } from '#/hooks/use-active-organization'
import { resetTicketPriorities } from '#/pages/support/feature/mocks/priorities'
import { SupportBoardFeature } from '#/pages/support/feature/support-board-feature'
import {
	ALL_PERMISSIONS,
	callsTo,
	chooseOption,
	createQueryClient,
	deferred,
	ENTRIES_PATH,
	type FakeApi,
	installFakeTanstackApi,
	installRadixPolyfills,
	MEMBERS,
	MEMBERS_PATH,
	Providers,
	page,
	SETTINGS_PATH,
	STATUS_PATH,
	TICKET_PATH,
	TICKETS_PATH,
	ticket,
} from '#/pages/support/feature/support-test-kit'
import { EMPTY_SUPPORT_FILTERS } from '#/pages/support/lib/filters'
import type { SupportFilters, SupportView } from '#/pages/support/types'

vi.mock('#/hooks/use-active-organization', async (importOriginal) => {
	const actual =
		await importOriginal<typeof import('#/hooks/use-active-organization')>()
	return {
		...actual,
		useActiveOrganization: vi.fn(actual.useActiveOrganization),
	}
})

beforeAll(installRadixPolyfills)
beforeEach(resetTicketPriorities)

function serverTickets(): Schemas.TicketResponse[] {
	return [
		ticket({
			id: 'ticket-1',
			number: 1,
			subject: 'Fuite sous évier',
			status: 'new',
			assignee_member_id: 'member-1',
			custom_fields: { ville: 'Lyon' },
			updated_at: '2026-10-04T07:00:00Z',
		}),
		ticket({
			id: 'ticket-2',
			number: 2,
			subject: 'Devis salle de bain',
			status: 'new',
			updated_at: '2026-10-03T07:00:00Z',
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

interface RenderOptions {
	permissions?: string[]
	filters?: SupportFilters
	displayFields?: string[]
	configure?: (api: FakeApi) => void
	ready?: boolean
}

function Harness({
	initialFilters,
	onFiltersChange,
	onOpenTicket,
	onOpenFullPage,
	onViewChange,
}: {
	initialFilters: SupportFilters
	onFiltersChange: (filters: SupportFilters) => void
	onOpenTicket: (ticketId: string | null) => void
	onOpenFullPage: (ticketId: string) => void
	onViewChange: (view: SupportView) => void
}) {
	const [filters, setFilters] = useState(initialFilters)

	return (
		<SupportBoardFeature
			filters={filters}
			onFiltersChange={(next) => {
				onFiltersChange(next)
				setFilters(next)
			}}
			onOpenTicket={(ticketId) => {
				onOpenTicket(ticketId)
				setFilters((current) => ({
					...current,
					ticket: ticketId ?? undefined,
				}))
			}}
			onOpenFullPage={onOpenFullPage}
			onViewChange={onViewChange}
		/>
	)
}

async function renderBoard({
	permissions = ALL_PERMISSIONS,
	filters = EMPTY_SUPPORT_FILTERS,
	displayFields = ['ville'],
	configure,
	ready = true,
}: RenderOptions = {}) {
	const api = installFakeTanstackApi()
	let tickets = serverTickets()
	api.mockGet(TICKETS_PATH, () => page(tickets))
	api.mockGet(SETTINGS_PATH, () => ({
		data: { display_fields: displayFields },
	}))
	api.mockGet(MEMBERS_PATH, () => ({ data: MEMBERS, pagination: null }))
	api.mockGet(TICKET_PATH, (params) => {
		const { path } = params as { path: { ticket_id: string } }
		return { data: tickets.find((entry) => entry.id === path.ticket_id) }
	})
	api.mockGet(ENTRIES_PATH, () => ({ data: [], pagination: null }))
	api.mockMutation('patch', STATUS_PATH, (params) => {
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

	const queryClient = createQueryClient(permissions)
	const filterChanges: SupportFilters[] = []
	const openedTickets: (string | null)[] = []
	const fullPages: string[] = []
	const views: SupportView[] = []
	render(
		<Providers queryClient={queryClient}>
			<Harness
				initialFilters={filters}
				onFiltersChange={(next) => filterChanges.push(next)}
				onOpenTicket={(ticketId) => openedTickets.push(ticketId)}
				onOpenFullPage={(ticketId) => fullPages.push(ticketId)}
				onViewChange={(view) => views.push(view)}
			/>
		</Providers>,
	)

	if (ready) await screen.findByRole('button', { name: /Fuite sous évier/ })

	return {
		...api,
		filterChanges,
		openedTickets,
		fullPages,
		views,
		user: userEvent.setup(),
	}
}

function column(label: string): HTMLElement {
	return screen.getByRole('region', { name: label })
}

function card(subject: string): HTMLElement {
	return within(column(columnOf(subject))).getByRole('button', {
		name: new RegExp(subject),
	})
}

const COLUMN_OF: Record<string, string> = {
	'Fuite sous évier': 'Backlog',
	'Devis salle de bain': 'Backlog',
	'Chauffe-eau en panne': 'Ouvert',
	'Robinet qui goutte': 'Fermé',
}

function columnOf(subject: string): string {
	return COLUMN_OF[subject] ?? 'Backlog'
}

function subjectsIn(label: string): string[] {
	return within(column(label))
		.queryAllByRole('button')
		.map((element) => element.querySelector('.line-clamp-2')?.textContent ?? '')
}

function patchCalls(api: FakeApi) {
	return callsTo(api, 'patch', STATUS_PATH)
}

function ticketQueries(api: FakeApi) {
	return callsTo(api, 'get', TICKETS_PATH).map(
		(call) => (call.params as { query?: unknown }).query,
	)
}

function drag(from: HTMLElement, to: HTMLElement) {
	const dataTransfer = { setData() {}, effectAllowed: '', dropEffect: '' }
	fireEvent.dragStart(from, { dataTransfer })
	fireEvent.dragOver(to, { dataTransfer })
	fireEvent.drop(to, { dataTransfer })
	fireEvent.dragEnd(from, { dataTransfer })
}

describe('SupportBoardFeature — rendu', () => {
	it('titles the page with the count of open tickets', async () => {
		await renderBoard()

		expect(
			screen.getByRole('heading', { level: 1, name: 'Boîte de réception' }),
		).toBeDefined()
		expect(
			screen.getByText(
				'3 tickets non fermés. Les demandes reçues par l’API arrivent dans le backlog.',
			),
		).toBeDefined()
	})

	it('groups the tickets into the four columns, in order', async () => {
		await renderBoard()

		expect(
			screen
				.getAllByRole('region')
				.map((region) => region.getAttribute('aria-label')),
		).toEqual(['Backlog', 'Ouvert', 'En attente client', 'Fermé'])
		expect(subjectsIn('Backlog')).toEqual([
			'Fuite sous évier',
			'Devis salle de bain',
		])
		expect(subjectsIn('Ouvert')).toEqual(['Chauffe-eau en panne'])
		expect(subjectsIn('En attente client')).toEqual([])
		expect(subjectsIn('Fermé')).toEqual(['Robinet qui goutte'])
		expect(
			within(column('En attente client')).getByText(
				'Nous attendons une réponse du client.',
			),
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
		expect(withValue.getByText('Lyon')).toBeDefined()
		expect(within(card('Devis salle de bain')).queryByText('Lyon')).toBeNull()
	})

	it('shows the priority of every card', async () => {
		await renderBoard()

		expect(within(card('Fuite sous évier')).getByText('Normale')).toBeDefined()
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

	it('shows the error of a failed listing and reads it again on retry', async () => {
		let reads = 0
		const { user } = await renderBoard({
			ready: false,
			configure: (api) =>
				api.mockGet(TICKETS_PATH, () => {
					reads += 1
					if (reads === 1) throw new Error('Le support est indisponible.')
					return page(serverTickets())
				}),
		})

		expect(
			await screen.findByText('Le support est indisponible.'),
		).toBeDefined()
		await user.click(screen.getByRole('button', { name: 'Réessayer' }))

		expect(
			await screen.findByRole('button', { name: /Fuite sous évier/ }),
		).toBeDefined()
	})
})

describe('SupportBoardFeature — ouverture', () => {
	it('opens the ticket whose card is clicked', async () => {
		const { openedTickets, user } = await renderBoard()

		await user.click(card('Devis salle de bain'))

		expect(openedTickets).toEqual(['ticket-2'])
	})

	it('opens a ticket without MANAGE_SUPPORT too', async () => {
		const { openedTickets, user } = await renderBoard({
			permissions: ['VIEW_SUPPORT'],
		})

		await user.click(card('Chauffe-eau en panne'))

		expect(openedTickets).toEqual(['ticket-3'])
	})

	it('shows the ticket in a dialog once it is open, and closes it', async () => {
		const { openedTickets, user } = await renderBoard({
			filters: { ...EMPTY_SUPPORT_FILTERS, ticket: 'ticket-1' },
			ready: false,
		})

		const dialog = await screen.findByRole('dialog')
		expect(
			await within(dialog).findByRole('heading', { name: /Fuite sous évier/ }),
		).toBeDefined()

		await user.click(within(dialog).getByLabelText('Fermer le ticket'))

		expect(openedTickets).toEqual([null])
		await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
	})

	it('opens the full page of the ticket from the dialog', async () => {
		const { fullPages, user } = await renderBoard({
			filters: { ...EMPTY_SUPPORT_FILTERS, ticket: 'ticket-1' },
			ready: false,
		})

		await user.click(await screen.findByLabelText('Ouvrir en pleine page'))

		expect(fullPages).toEqual(['ticket-1'])
	})
})

describe('SupportBoardFeature — permission', () => {
	it('makes no card draggable without MANAGE_SUPPORT', async () => {
		await renderBoard({ permissions: ['VIEW_SUPPORT'] })

		for (const subject of Object.keys(COLUMN_OF)) {
			expect(card(subject).getAttribute('draggable')).toBeNull()
		}
	})

	it('makes every card draggable with MANAGE_SUPPORT', async () => {
		await renderBoard({ permissions: ['MANAGE_SUPPORT'] })

		for (const subject of Object.keys(COLUMN_OF)) {
			expect(card(subject).getAttribute('draggable')).toBe('true')
		}
	})

	it('ignores the arrow keys without MANAGE_SUPPORT', async () => {
		const api = await renderBoard({ permissions: ['VIEW_SUPPORT'] })

		fireEvent.keyDown(card('Fuite sous évier'), { key: 'ArrowRight' })

		await waitFor(() => expect(subjectsIn('Backlog')).toHaveLength(2))
		expect(patchCalls(api)).toHaveLength(0)
	})
})

describe('SupportBoardFeature — clavier', () => {
	it('moves a ticket one column over at once and sends status alone', async () => {
		const patch = deferred<unknown>()
		const api = await renderBoard({
			configure: (fakeApi) =>
				fakeApi.mockMutation('patch', STATUS_PATH, () => patch.promise),
		})
		const moved = card('Fuite sous évier')
		moved.focus()

		fireEvent.keyDown(moved, { key: 'ArrowRight' })

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		expect(patchCalls(api)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { status: 'in_progress' },
		})
		await waitFor(() =>
			expect(subjectsIn('Ouvert')).toEqual([
				'Fuite sous évier',
				'Chauffe-eau en panne',
			]),
		)
		expect(subjectsIn('Backlog')).toEqual(['Devis salle de bain'])
		expect(
			screen.getByText('Fuite sous évier déplacé vers Ouvert'),
		).toBeDefined()

		patch.resolve({ data: ticket({ id: 'ticket-1', status: 'in_progress' }) })
	})

	it('moves a ticket one column back', async () => {
		const api = await renderBoard()

		fireEvent.keyDown(card('Chauffe-eau en panne'), { key: 'ArrowLeft' })

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		expect(patchCalls(api)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-3' },
			body: { status: 'new' },
		})
	})

	it('sends nothing past the first or the last column', async () => {
		const api = await renderBoard()

		fireEvent.keyDown(card('Fuite sous évier'), { key: 'ArrowLeft' })
		fireEvent.keyDown(card('Robinet qui goutte'), { key: 'ArrowRight' })

		await waitFor(() => expect(subjectsIn('Backlog')).toHaveLength(2))
		expect(patchCalls(api)).toHaveLength(0)
	})
})

describe('SupportBoardFeature — glisser-déposer', () => {
	it('sends one PATCH for a drop on another column', async () => {
		const api = await renderBoard()

		drag(card('Devis salle de bain'), column('Fermé'))

		await waitFor(() => expect(patchCalls(api)).toHaveLength(1))
		expect(patchCalls(api)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-2' },
			body: { status: 'resolved' },
		})
		await waitFor(() =>
			expect(subjectsIn('Fermé')).toContain('Devis salle de bain'),
		)
	})

	it('sends nothing for a drop on the ticket’s own column', async () => {
		const api = await renderBoard()

		drag(card('Fuite sous évier'), column('Backlog'))

		await waitFor(() => expect(subjectsIn('Backlog')).toHaveLength(2))
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
				api.mockMutation('patch', STATUS_PATH, () => patch.promise)
			},
		})

		fireEvent.keyDown(card('Fuite sous évier'), { key: 'ArrowRight' })
		await waitFor(() =>
			expect(subjectsIn('Ouvert')).toEqual([
				'Fuite sous évier',
				'Chauffe-eau en panne',
			]),
		)

		patch.reject(new Error('Le serveur a refusé le déplacement.'))

		await waitFor(() =>
			expect(subjectsIn('Backlog')).toEqual([
				'Fuite sous évier',
				'Devis salle de bain',
			]),
		)
		expect(subjectsIn('Ouvert')).toEqual(['Chauffe-eau en panne'])

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

	it('keeps the unassigned filter on the client side', async () => {
		const api = await renderBoard({
			filters: { ...EMPTY_SUPPORT_FILTERS, assignee: 'none' },
			ready: false,
		})

		await screen.findByRole('button', { name: /Devis salle de bain/ })
		expect(ticketQueries(api)).toEqual([{ per_page: 100 }])
		expect(subjectsIn('Backlog')).toEqual(['Devis salle de bain'])
		expect(
			screen.getByRole('heading', { level: 1, name: 'Non assignés' }),
		).toBeDefined()
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

	it('counts the active filters on the popover button', async () => {
		await renderBoard({
			filters: { ...EMPTY_SUPPORT_FILTERS, badge: 'Urgent', priority: 'high' },
			ready: false,
		})

		expect(
			await screen.findByRole('img', { name: '2 filtres actifs' }),
		).toBeDefined()
	})

	it('writes a picked assignee back and asks the API again', async () => {
		const api = await renderBoard()
		await api.user.click(screen.getByRole('button', { name: /Filtres/ }))
		await screen.findByLabelText('Assigné à')

		await chooseOption(
			api.user,
			screen.getByLabelText('Assigné à'),
			/Alix Martin/,
		)

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
		const api = await renderBoard({
			configure: (fakeApi) =>
				fakeApi.mockGet(TICKETS_PATH, () =>
					page([
						ticket({
							subject: 'Fuite sous évier',
							badges: [{ label: 'Urgent', tone: 'error' }],
						}),
						ticket({
							id: 'ticket-2',
							subject: 'Autre demande',
							badges: [{ label: 'Garantie', tone: 'success' }],
						}),
					]),
				),
		})
		await api.user.click(screen.getByRole('button', { name: /Filtres/ }))
		await api.user.click(await screen.findByLabelText('Badge'))

		expect(
			(await screen.findAllByRole('option')).map(
				(option) => option.textContent,
			),
		).toEqual(['Tous les badges', 'Garantie', 'Urgent'])
	})

	it('keeps a field key from a shared link selectable when it is no longer displayed', async () => {
		const api = await renderBoard({
			filters: { ...EMPTY_SUPPORT_FILTERS, custom_field_key: 'ancien_champ' },
		})
		await api.user.click(screen.getByRole('button', { name: /Filtres/ }))
		await api.user.click(await screen.findByLabelText('Champ'))

		const names = (await screen.findAllByRole('option')).map(
			(option) => option.textContent,
		)
		expect(names).toContain('ancien_champ')
		expect(names).toContain('ville')
	})

	it('resets every filter at once', async () => {
		const api = await renderBoard({
			filters: {
				...EMPTY_SUPPORT_FILTERS,
				badge: 'Urgent',
				priority: 'high',
			},
			ready: false,
		})
		await api.user.click(await screen.findByRole('button', { name: /Filtres/ }))

		await api.user.click(
			await screen.findByRole('button', { name: /Réinitialiser/ }),
		)

		expect(api.filterChanges.at(-1)).toMatchObject({
			badge: undefined,
			priority: undefined,
			assignee: undefined,
		})
	})

	it('explains an empty board that a filter emptied', async () => {
		await renderBoard({
			filters: { ...EMPTY_SUPPORT_FILTERS, badge: 'Urgent' },
			ready: false,
			configure: (api) => api.mockGet(TICKETS_PATH, () => page([])),
		})

		expect(
			await screen.findByText('Aucun ticket ne correspond à ces filtres.'),
		).toBeDefined()
	})

	it('narrows the board with the search field', async () => {
		const { user } = await renderBoard()

		await user.type(screen.getByLabelText('Rechercher un ticket'), 'chauffe')

		await waitFor(() => expect(subjectsIn('Backlog')).toEqual([]))
		expect(subjectsIn('Ouvert')).toEqual(['Chauffe-eau en panne'])
	})

	it('filters by priority on the client side', async () => {
		const api = await renderBoard({
			filters: { ...EMPTY_SUPPORT_FILTERS, priority: 'urgent' },
			ready: false,
		})

		expect(
			await screen.findByText('Aucun ticket ne correspond à ces filtres.'),
		).toBeDefined()
		expect(ticketQueries(api)).toEqual([{ per_page: 100 }])
		expect(subjectsIn('Backlog')).toEqual([])
	})
})

describe('SupportBoardFeature — barre d’outils', () => {
	it('switches to the list view', async () => {
		const { user, views } = await renderBoard()

		await user.click(screen.getByRole('radio', { name: 'Liste' }))

		expect(views).toEqual(['list'])
	})

	it('lists the custom fields to display and saves a changed choice', async () => {
		const { user, calls, mockMutation } = await renderBoard({
			configure: (api) =>
				api.mockMutation('put', SETTINGS_PATH, () => ({
					data: { display_fields: [] },
				})),
		})
		void mockMutation

		await user.click(screen.getByRole('button', { name: 'Champs affichés' }))
		await user.click(
			await screen.findByRole('menuitemcheckbox', { name: 'ville' }),
		)

		await waitFor(() =>
			expect(
				calls.filter(
					(call) => call.method === 'put' && call.path === SETTINGS_PATH,
				),
			).toHaveLength(1),
		)
		expect(
			calls.find((call) => call.method === 'put' && call.path === SETTINGS_PATH)
				?.params,
		).toEqual({
			path: { organization_id: 'org-1' },
			body: { display_fields: [] },
		})
	})

	it('does not let a reader change the displayed fields', async () => {
		const { user } = await renderBoard({ permissions: ['VIEW_SUPPORT'] })

		await user.click(screen.getByRole('button', { name: 'Champs affichés' }))

		expect(
			(
				await screen.findByRole('menuitemcheckbox', { name: 'ville' })
			).getAttribute('aria-disabled'),
		).toBe('true')
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
					onOpenTicket={() => {}}
					onOpenFullPage={() => {}}
					onViewChange={() => {}}
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
