import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
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
import { SupportListFeature } from '#/pages/support/feature/support-list-feature'
import {
	ALL_PERMISSIONS,
	callsTo,
	createQueryClient,
	ENTRIES_PATH,
	type FakeApi,
	installFakeTanstackApi,
	installRadixPolyfills,
	MEMBERS,
	MEMBERS_PATH,
	Providers,
	page,
	SETTINGS_PATH,
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
			body: 'Bonjour,\n ma canalisation fuit.',
			status: 'new',
			assignee_member_id: 'member-1',
			custom_fields: { ville: 'Lyon', requester_name: 'Claire Fontaine' },
			badges: [{ label: 'Urgent', tone: 'error' }],
			updated_at: '2026-10-04T07:00:00Z',
		}),
		ticket({
			id: 'ticket-2',
			number: 2,
			subject: 'Devis salle de bain',
			status: 'in_progress',
			updated_at: '2026-10-05T07:00:00Z',
		}),
		ticket({
			id: 'ticket-3',
			number: 3,
			subject: 'Robinet qui goutte',
			status: 'resolved',
		}),
	]
}

interface RenderOptions {
	permissions?: string[]
	filters?: SupportFilters
	configure?: (api: FakeApi) => void
	ready?: boolean
}

function Harness({
	initialFilters,
	onFiltersChange,
	onOpenTicket,
	onViewChange,
	onOpenFullPage,
}: {
	initialFilters: SupportFilters
	onFiltersChange: (filters: SupportFilters) => void
	onOpenTicket: (ticketId: string | null) => void
	onOpenFullPage: (ticketId: string) => void
	onViewChange: (view: SupportView) => void
}) {
	const [filters, setFilters] = useState(initialFilters)

	return (
		<SupportListFeature
			filters={filters}
			onFiltersChange={(next) => {
				onFiltersChange(next)
				setFilters(next)
			}}
			onOpenTicket={(ticketId) => {
				onOpenTicket(ticketId)
				setFilters((current) => ({ ...current, ticket: ticketId ?? undefined }))
			}}
			onOpenFullPage={onOpenFullPage}
			onViewChange={onViewChange}
		/>
	)
}

async function renderList({
	permissions = ALL_PERMISSIONS,
	filters = EMPTY_SUPPORT_FILTERS,
	configure,
	ready = true,
}: RenderOptions = {}) {
	const api = installFakeTanstackApi()
	const tickets = serverTickets()
	api.mockGet(TICKETS_PATH, () => page(tickets))
	api.mockGet(SETTINGS_PATH, () => ({ data: { display_fields: ['ville'] } }))
	api.mockGet(MEMBERS_PATH, () => ({ data: MEMBERS, pagination: null }))
	api.mockGet(TICKET_PATH, (params) => {
		const { path } = params as { path: { ticket_id: string } }
		return { data: tickets.find((entry) => entry.id === path.ticket_id) }
	})
	api.mockGet(ENTRIES_PATH, () => ({ data: [], pagination: null }))
	configure?.(api)

	const filterChanges: SupportFilters[] = []
	const openedTickets: (string | null)[] = []
	const views: SupportView[] = []
	render(
		<Providers queryClient={createQueryClient(permissions)}>
			<Harness
				initialFilters={filters}
				onFiltersChange={(next) => filterChanges.push(next)}
				onOpenTicket={(ticketId) => openedTickets.push(ticketId)}
				onOpenFullPage={() => {}}
				onViewChange={(view) => views.push(view)}
			/>
		</Providers>,
	)

	if (ready) await screen.findByRole('button', { name: /Devis salle de bain/ })

	return {
		...api,
		filterChanges,
		openedTickets,
		views,
		user: userEvent.setup(),
	}
}

function rows(): string[] {
	return within(screen.getByRole('region', { name: 'Liste des tickets' }))
		.queryAllByRole('button')
		.filter((button) => button.className.includes('border-b'))
		.map((button) => button.textContent ?? '')
}

function ticketQueries(api: FakeApi) {
	return callsTo(api, 'get', TICKETS_PATH).map(
		(call) => (call.params as { query?: unknown }).query,
	)
}

describe('SupportListFeature — lignes', () => {
	it('hides the closed tickets by default and counts the visible ones', async () => {
		await renderList()

		expect(rows()).toHaveLength(2)
		expect(screen.queryByText(/Robinet qui goutte/)).toBeNull()
		expect(screen.getByRole('heading', { level: 1 }).textContent).toBe(
			'Boîte de réception',
		)
		expect(screen.getByTitle(/Tickets affichés/).textContent).toBe('2')
	})

	it('shows the closed tickets once the status filter says so', async () => {
		await renderList({ filters: { ...EMPTY_SUPPORT_FILTERS, status: 'all' } })

		expect(rows()).toHaveLength(3)
	})

	it('shows requester, number, subject, preview, status, badge and display field in a row', async () => {
		await renderList()

		const row = rows().find((text) => text.includes('Fuite sous évier'))
		expect(row).toContain('Claire Fontaine')
		expect(row).toContain('#1')
		expect(row).toContain('Bonjour, ma canalisation fuit.')
		expect(row).toContain('Backlog')
		expect(row).toContain('Urgent')
		expect(row).toContain('Lyon')
	})

	it('orders the rows by activity within the same priority', async () => {
		await renderList()

		expect(rows()[0]).toContain('Devis salle de bain')
		expect(rows()[1]).toContain('Fuite sous évier')
	})

	it('needs no permission to read', async () => {
		await renderList({ permissions: ['VIEW_SUPPORT'] })

		expect(rows()).toHaveLength(2)
	})

	it('invites to pick a ticket while none is open', async () => {
		await renderList()

		expect(screen.getByText('Sélectionnez un ticket')).toBeDefined()
	})
})

describe('SupportListFeature — ouverture', () => {
	it('opens the ticket whose row is clicked and marks the row', async () => {
		const { openedTickets, user } = await renderList()

		await user.click(
			screen.getByRole('button', { name: /Devis salle de bain/ }),
		)

		expect(openedTickets).toEqual(['ticket-2'])
		const pane = await screen.findByRole('region', { name: 'Détail du ticket' })
		expect(
			await within(pane).findByRole('heading', { name: /Devis salle de bain/ }),
		).toBeDefined()
		expect(
			screen
				.getAllByRole('button', { name: /Devis salle de bain/ })[0]
				.getAttribute('aria-current'),
		).toBe('true')
	})

	it('closes the pane', async () => {
		const { openedTickets, user } = await renderList({
			filters: { ...EMPTY_SUPPORT_FILTERS, ticket: 'ticket-2' },
		})

		await user.click(await screen.findByLabelText('Fermer le ticket'))

		expect(openedTickets).toEqual([null])
		expect(await screen.findByText('Sélectionnez un ticket')).toBeDefined()
	})

	it('goes back to the board and drops the open ticket', async () => {
		const { openedTickets, views, user } = await renderList()

		await user.click(screen.getByRole('radio', { name: 'Kanban' }))

		expect(views).toEqual(['board'])
		expect(openedTickets).toEqual([null])
	})
})

describe('SupportListFeature — états', () => {
	it('says so when the listing was cut short', async () => {
		await renderList({
			configure: (api) =>
				api.mockGet(TICKETS_PATH, () => page(serverTickets(), 240)),
		})

		expect(
			screen.getByText(
				'3 tickets affichés sur 240. Affinez les filtres pour voir les autres.',
			),
		).toBeDefined()
	})

	it('stays quiet when the listing is complete', async () => {
		await renderList()

		expect(screen.queryByText(/tickets affichés sur/)).toBeNull()
	})

	it('explains an empty list that a filter emptied', async () => {
		await renderList({
			filters: { ...EMPTY_SUPPORT_FILTERS, badge: 'Urgent' },
			ready: false,
			configure: (api) => api.mockGet(TICKETS_PATH, () => page([])),
		})

		expect(await screen.findByText('Aucun ticket')).toBeDefined()
		expect(
			screen.getByText('Aucun ticket ne correspond à ces filtres.'),
		).toBeDefined()
	})

	it('says there is no ticket yet without any filter', async () => {
		await renderList({
			ready: false,
			configure: (api) => api.mockGet(TICKETS_PATH, () => page([])),
		})

		expect(
			await screen.findByText('Aucun ticket pour le moment.'),
		).toBeDefined()
	})

	it('shows the error of a failed listing and retries on demand', async () => {
		let reads = 0
		const { user } = await renderList({
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
			await screen.findByRole('button', { name: /Devis salle de bain/ }),
		).toBeDefined()
	})
})

describe('SupportListFeature — filtres', () => {
	it('sends the filters of the URL to the API, the custom field pair only when complete', async () => {
		const api = await renderList({
			filters: {
				...EMPTY_SUPPORT_FILTERS,
				assignee: 'member-1',
				badge: 'Urgent',
				custom_field_key: 'ville',
				custom_field_value: 'Lyon',
			},
			ready: false,
		})

		await waitFor(() => expect(ticketQueries(api)).toHaveLength(1))
		expect(ticketQueries(api)[0]).toEqual({
			per_page: 100,
			assignee: 'member-1',
			badge: 'Urgent',
			custom_field_key: 'ville',
			custom_field_value: 'Lyon',
		})
	})

	it('never sends a lone custom field key', async () => {
		const api = await renderList({
			filters: { ...EMPTY_SUPPORT_FILTERS, custom_field_key: 'ville' },
		})

		expect(ticketQueries(api)).toEqual([{ per_page: 100 }])
	})

	it('keeps the status choice on the client side', async () => {
		const api = await renderList({
			filters: { ...EMPTY_SUPPORT_FILTERS, status: 'resolved' },
			ready: false,
		})

		await screen.findByRole('button', { name: /Robinet qui goutte/ })
		expect(ticketQueries(api)).toEqual([{ per_page: 100 }])
		expect(rows()).toHaveLength(1)
	})

	it('writes a picked status back', async () => {
		const api = await renderList()
		await api.user.click(screen.getByRole('button', { name: /Filtres/ }))
		await api.user.click(await screen.findByLabelText('Statut'))

		await api.user.click(await screen.findByRole('option', { name: 'Fermé' }))

		expect(api.filterChanges.at(-1)).toEqual({
			...EMPTY_SUPPORT_FILTERS,
			status: 'resolved',
		})
	})

	it('narrows the list with the search field', async () => {
		const { user } = await renderList()

		await user.type(screen.getByLabelText('Rechercher un ticket'), 'claire')

		await waitFor(() => expect(rows()).toHaveLength(1))
		expect(rows()[0]).toContain('Fuite sous évier')
	})

	it('clears every filter at once', async () => {
		const api = await renderList({
			filters: { ...EMPTY_SUPPORT_FILTERS, badge: 'Urgent', status: 'all' },
		})
		await api.user.click(screen.getByRole('button', { name: /Filtres/ }))

		await api.user.click(
			await screen.findByRole('button', { name: /Réinitialiser/ }),
		)

		expect(api.filterChanges.at(-1)).toMatchObject({
			badge: undefined,
			status: undefined,
		})
	})
})

describe('SupportListFeature — organisation', () => {
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
				<SupportListFeature
					filters={EMPTY_SUPPORT_FILTERS}
					onFiltersChange={() => {}}
					onOpenTicket={() => {}}
					onOpenFullPage={() => {}}
					onViewChange={() => {}}
				/>
			</QueryClientProvider>,
		)

		expect(screen.getByText('Organisation indisponible')).toBeDefined()
		expect(api.calls).toEqual([])
	})
})
