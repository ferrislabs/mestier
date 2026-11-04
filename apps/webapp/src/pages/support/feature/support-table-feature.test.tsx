import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import {
	fireEvent,
	render,
	screen,
	waitFor,
	within,
} from '@testing-library/react'
import { type ReactNode, useState } from 'react'
import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { SupportTableFeature } from '#/pages/support/feature/support-table-feature'
import { EMPTY_SUPPORT_FILTERS } from '#/pages/support/lib/filters'
import type { SupportFilters } from '#/pages/support/types'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

const TICKETS_PATH = '/api/v1/organizations/{organization_id}/support/tickets'
const SETTINGS_PATH = '/api/v1/organizations/{organization_id}/support/settings'
const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'

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
			number: 12,
			subject: 'Fuite sous évier',
			status: 'in_progress',
			assignee_member_id: 'member-1',
			custom_fields: { ville: 'Lyon', etage: 3 },
			badges: [{ label: 'Urgent', tone: 'error' }],
		}),
		ticket({
			id: 'ticket-2',
			number: 11,
			subject: 'Devis salle de bain',
			status: 'new',
			custom_fields: { ville: 'Nantes' },
		}),
		ticket({
			id: 'ticket-3',
			number: 10,
			subject: 'Chauffe-eau en panne',
			status: 'waiting_customer',
			assignee_member_id: 'member-9',
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
		mutation() {
			throw new Error('the table never writes')
		},
	}

	Object.assign(window, { tanstackApi: fakeApi })

	return {
		calls,
		mockGet(path: string, handler: Handler) {
			getHandlers.set(path, handler)
		},
	}
}

type FakeApi = ReturnType<typeof installFakeTanstackApi>

interface RenderOptions {
	permissions?: string[]
	filters?: SupportFilters
	displayFields?: string[]
	configure?: (api: FakeApi) => void
	ready?: RegExp | null | false
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
		<SupportTableFeature
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

async function renderTable({
	permissions = [],
	filters = EMPTY_SUPPORT_FILTERS,
	displayFields = ['ville', 'etage'],
	configure,
	ready = /Fuite sous évier/,
}: RenderOptions = {}) {
	const api = installFakeTanstackApi()
	api.mockGet(TICKETS_PATH, () => page(serverTickets()))
	api.mockGet(SETTINGS_PATH, () => ({
		data: { display_fields: displayFields },
	}))
	api.mockGet(MEMBERS_PATH, () => ({ data: MEMBERS, pagination: null }))
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
		await screen.findByText(ready)
	} else if (ready === null) {
		await screen.findByText('Aucun ticket ne correspond aux filtres appliqués.')
	}

	return { ...api, filterChanges }
}

function rowCells(subject: string): string[] {
	const row = screen.getByRole('row', { name: new RegExp(subject) })
	return within(row)
		.getAllByRole('cell')
		.map((cell) => cell.textContent ?? '')
}

function ticketQueries(api: FakeApi) {
	return api.calls
		.filter((call) => call.method === 'get' && call.path === TICKETS_PATH)
		.map((call) => (call.params as { query?: unknown }).query)
}

describe('SupportTableFeature — colonnes', () => {
	it('orders the headers: the four fixed columns, the display fields in settings order, the badges', async () => {
		await renderTable({ displayFields: ['ville', 'etage'] })

		expect(
			screen.getAllByRole('columnheader').map((header) => header.textContent),
		).toEqual([
			'N°',
			'Sujet',
			'Statut',
			'Assigné à',
			'ville',
			'etage',
			'Badges',
		])
	})

	it('follows the settings when they name other fields', async () => {
		await renderTable({ displayFields: ['ville'] })

		expect(
			screen.getAllByRole('columnheader').map((header) => header.textContent),
		).toEqual(['N°', 'Sujet', 'Statut', 'Assigné à', 'ville', 'Badges'])
	})
})

describe('SupportTableFeature — lignes', () => {
	it('lists one row per ticket, in the order the API served them', async () => {
		await renderTable()

		const subjects = screen
			.getAllByRole('row')
			.slice(1)
			.map((row) => within(row).getAllByRole('cell')[1].textContent)
		expect(subjects).toEqual([
			'Fuite sous évier',
			'Devis salle de bain',
			'Chauffe-eau en panne',
		])
	})

	it('fills every cell of a complete ticket', async () => {
		await renderTable()

		expect(rowCells('Fuite sous évier')).toEqual([
			'#12',
			'Fuite sous évier',
			'En cours',
			'Alix Martin',
			'Lyon',
			'3',
			'Urgent',
		])
	})

	it('renders an empty cell for a field the ticket lacks', async () => {
		await renderTable()

		expect(rowCells('Devis salle de bain')).toEqual([
			'#11',
			'Devis salle de bain',
			'Nouveau',
			'Non assigné',
			'Nantes',
			'',
			'',
		])
	})

	it('resolves the assignee from the members and flags a stranger', async () => {
		await renderTable()

		expect(rowCells('Fuite sous évier')[3]).toBe('Alix Martin')
		expect(rowCells('Chauffe-eau en panne')[3]).toBe('Membre inconnu')
		expect(rowCells('Chauffe-eau en panne')[2]).toBe('En attente du client')
	})

	it('needs no permission to read', async () => {
		await renderTable({ permissions: [] })

		expect(screen.queryAllByRole('button', { name: /^Déplacer / })).toEqual([])
		expect(screen.getAllByRole('row')).toHaveLength(4)
	})
})

describe('SupportTableFeature — états', () => {
	it('says so when the listing was cut short', async () => {
		await renderTable({
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
		await renderTable()

		expect(screen.queryByText(/tickets affichés sur/)).toBeNull()
	})

	it('explains an empty table that a filter emptied', async () => {
		await renderTable({
			filters: { ...EMPTY_SUPPORT_FILTERS, badge: 'Urgent' },
			ready: null,
			configure: (api) => api.mockGet(TICKETS_PATH, () => page([])),
		})

		expect(
			screen.getByText('Aucun ticket ne correspond aux filtres appliqués.'),
		).toBeDefined()
	})

	it('shows the error of a failed listing and retries on demand', async () => {
		let reads = 0
		await renderTable({
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

		fireEvent.click(screen.getByRole('button', { name: 'Réessayer' }))

		expect(await screen.findByText('Fuite sous évier')).toBeDefined()
		expect(screen.queryByText('Le support est indisponible.')).toBeNull()
	})
})

describe('SupportTableFeature — filtres', () => {
	it('sends the filters of the URL to the API, the custom field pair only when complete', async () => {
		const complete = await renderTable({
			filters: {
				...EMPTY_SUPPORT_FILTERS,
				assignee: 'member-1',
				custom_field_key: 'ville',
				custom_field_value: 'Lyon',
			},
		})

		expect(ticketQueries(complete)).toEqual([
			{
				per_page: 100,
				assignee: 'member-1',
				custom_field_key: 'ville',
				custom_field_value: 'Lyon',
			},
		])
	})

	it('never sends a lone custom field key', async () => {
		const api = await renderTable({
			filters: { ...EMPTY_SUPPORT_FILTERS, custom_field_key: 'ville' },
		})

		expect(ticketQueries(api)).toEqual([{ per_page: 100 }])
	})

	it('writes a picked assignee back and asks the API again', async () => {
		const api = await renderTable()
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

	it('keeps a field key from a shared link selectable when it is no longer displayed', async () => {
		await renderTable({
			filters: { ...EMPTY_SUPPORT_FILTERS, custom_field_key: 'ancien_champ' },
		})

		expect(
			within(screen.getByLabelText('Champ')).getByRole('option', {
				name: 'ancien_champ',
			}),
		).toBeDefined()
	})

	it('clears every filter at once', async () => {
		const api = await renderTable({
			filters: { ...EMPTY_SUPPORT_FILTERS, badge: 'Urgent' },
		})

		fireEvent.click(screen.getByRole('button', { name: 'Effacer les filtres' }))

		expect(api.filterChanges.at(-1)).toStrictEqual(EMPTY_SUPPORT_FILTERS)
	})
})
