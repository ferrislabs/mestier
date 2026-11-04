import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
	useActiveOrganization,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { PERMISSION_CATALOG } from '#/lib/permission-catalog'
import { SupportTicketFeature } from '#/pages/support/feature/support-ticket-feature'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

vi.mock('#/hooks/use-active-organization', async (importOriginal) => {
	const actual =
		await importOriginal<typeof import('#/hooks/use-active-organization')>()
	return {
		...actual,
		useActiveOrganization: vi.fn(actual.useActiveOrganization),
	}
})

const TICKET_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}'
const ENTRIES_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/entries'
const STATUS_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/status'
const ASSIGNEE_PATH =
	'/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/assignee'
const ENTRY_PATH =
	'/api/v1/organizations/{organization_id}/support/entries/{entry_id}'
const ENTRY_SEND_PATH =
	'/api/v1/organizations/{organization_id}/support/entries/{entry_id}/send'
const ATTACHMENT_URL_PATH =
	'/api/v1/organizations/{organization_id}/support/attachments/{attachment_id}/url'
const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const ALL_PERMISSIONS = PERMISSION_CATALOG.map((entry) => entry.name)
const ATTACHMENT_URL = 'https://files.example/photo-fuite.jpg'
const ATTACHMENT_ERROR = 'Impossible d’ouvrir la pièce jointe.'

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
		number: 7,
		subject: 'Fuite sous évier',
		body: 'Bonjour, ma canalisation fuit.',
		status: 'new',
		assignee_member_id: null,
		assigned_at: null,
		custom_fields: { ville: 'Lyon' },
		badges: [{ label: 'Urgent', tone: 'error' }],
		external_id: null,
		created_at: '2026-11-01T07:00:00Z',
		updated_at: '2026-11-01T07:00:00Z',
		...overrides,
	}
}

function entry(
	overrides: Partial<Schemas.TicketEntryResponse> = {},
): Schemas.TicketEntryResponse {
	return {
		id: 'entry-1',
		ticket_id: 'ticket-1',
		kind: 'reply',
		body: 'Réponse',
		author_member_id: 'member-1',
		attachments: [],
		created_at: '2026-11-01T09:00:00Z',
		updated_at: '2026-11-01T09:00:00Z',
		...overrides,
	}
}

function serverEntries(): Schemas.TicketEntryResponse[] {
	return [
		entry({
			id: 'entry-1',
			kind: 'client_message',
			body: 'Bonjour, ma canalisation fuit.',
			author_member_id: null,
			created_at: '2026-11-01T08:00:00Z',
			attachments: [
				{
					id: 'attachment-1',
					file_name: 'photo-fuite.jpg',
					mime_type: 'image/jpeg',
					size_bytes: 2048,
					created_at: '2026-11-01T08:00:00Z',
				},
			],
		}),
		entry({
			id: 'entry-2',
			kind: 'reply',
			body: 'Nous passons demain matin.',
			author_member_id: 'member-1',
			created_at: '2026-11-01T09:00:00Z',
		}),
		entry({
			id: 'entry-3',
			kind: 'internal_note',
			body: 'Client difficile à joindre.',
			author_member_id: 'member-2',
			created_at: '2026-11-01T10:00:00Z',
		}),
		entry({
			id: 'entry-4',
			kind: 'draft',
			body: 'Brouillon de suivi.',
			author_member_id: 'member-1',
			created_at: '2026-11-01T11:00:00Z',
		}),
	]
}

interface ServerState {
	ticket: Schemas.TicketResponse
	entries: Schemas.TicketEntryResponse[]
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

function installServerMutations(api: FakeApi, server: ServerState) {
	api.mockMutation('post', ENTRIES_PATH, (params) => {
		const { body } = params as {
			body: { kind: 'reply' | 'draft' | 'note'; body: string }
		}
		const created = entry({
			id: `entry-${server.entries.length + 1}`,
			kind: body.kind === 'note' ? 'internal_note' : body.kind,
			body: body.body,
			created_at: '2026-11-02T08:00:00Z',
		})
		server.entries.push(created)
		return { data: created }
	})
	api.mockMutation('patch', ENTRY_PATH, (params) => {
		const { path, body } = params as {
			path: { entry_id: string }
			body: { body: string }
		}
		server.entries = server.entries.map((candidate) =>
			candidate.id === path.entry_id
				? { ...candidate, body: body.body }
				: candidate,
		)
		return { data: server.entries.find((item) => item.id === path.entry_id) }
	})
	api.mockMutation('post', ENTRY_SEND_PATH, (params) => {
		const { path } = params as { path: { entry_id: string } }
		server.entries = server.entries.map((candidate) =>
			candidate.id === path.entry_id
				? { ...candidate, kind: 'reply' }
				: candidate,
		)
		return { data: server.entries.find((item) => item.id === path.entry_id) }
	})
	api.mockMutation('delete', ENTRY_PATH, (params) => {
		const { path } = params as { path: { entry_id: string } }
		server.entries = server.entries.filter(
			(candidate) => candidate.id !== path.entry_id,
		)
		return {}
	})
	api.mockMutation('patch', STATUS_PATH, (params) => {
		const { body } = params as { body: { status: Schemas.TicketStatus } }
		server.ticket = { ...server.ticket, status: body.status }
		return { data: server.ticket }
	})
	api.mockMutation('put', ASSIGNEE_PATH, (params) => {
		const { body } = params as { body: { member_id: string | null } }
		server.ticket = { ...server.ticket, assignee_member_id: body.member_id }
		return { data: server.ticket }
	})
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

interface RenderOptions {
	permissions?: string[]
	onBack?: () => void
	configure?: (api: FakeApi, server: ServerState) => void
	ready?: boolean
}

async function renderTicket({
	permissions = ALL_PERMISSIONS,
	onBack = () => {},
	configure,
	ready = true,
}: RenderOptions = {}) {
	const api = installFakeTanstackApi()
	const server: ServerState = { ticket: ticket(), entries: serverEntries() }
	api.mockGet(TICKET_PATH, () => ({ data: server.ticket }))
	api.mockGet(ENTRIES_PATH, () => ({
		data: [...server.entries].reverse(),
		pagination: null,
	}))
	api.mockGet(MEMBERS_PATH, () => ({ data: MEMBERS, pagination: null }))
	api.mockGet(ATTACHMENT_URL_PATH, () => ({
		data: { url: ATTACHMENT_URL, expires_at: '2026-11-01T12:00:00Z' },
	}))
	installServerMutations(api, server)
	configure?.(api, server)

	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCacheForOrganization(queryClient, ORGANIZATION.id, permissions)

	render(
		<Providers queryClient={queryClient}>
			<SupportTicketFeature ticketId="ticket-1" onBack={onBack} />
		</Providers>,
	)

	if (ready) {
		await screen.findByRole('article', { name: 'Réponse envoyée, Alix Martin' })
	}

	return { ...api, server, user: userEvent.setup() }
}

function callsTo(api: FakeApi, method: string, path: string) {
	return api.calls.filter(
		(call) => call.method === method && call.path === path,
	)
}

function messageField(): HTMLTextAreaElement {
	return screen.getByRole('textbox', { name: 'Message' }) as HTMLTextAreaElement
}

function draftArticle(): HTMLElement {
	return screen.getByRole('article', { name: 'Brouillon, Alix Martin' })
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

describe('SupportTicketFeature — fil de discussion', () => {
	it('lists the entries in creation order even when the API serves them reversed', async () => {
		await renderTicket()

		const articles = screen.getAllByRole('article')
		const expectedBodies = [
			'Bonjour, ma canalisation fuit.',
			'Nous passons demain matin.',
			'Client difficile à joindre.',
			'Brouillon de suivi.',
		]
		expect(
			articles.map((article) => article.getAttribute('aria-label')),
		).toEqual([
			'Message du client, Client',
			'Réponse envoyée, Alix Martin',
			'Note interne, Jo Durand',
			'Brouillon, Alix Martin',
		])
		articles.forEach((article, index) => {
			expect(article.textContent).toContain(expectedBodies[index])
		})
	})

	it('titles the page with the number and the subject of the ticket', async () => {
		await renderTicket()

		expect(screen.getByRole('heading', { level: 1 }).textContent).toBe(
			'#7 Fuite sous évier',
		)
		expect(screen.getByText('Atelier Bois & Co')).toBeDefined()
	})

	it('shows the details of the ticket in the side panel', async () => {
		await renderTicket()

		expect(screen.getByText('Urgent')).toBeDefined()
		expect(screen.getByText('ville')).toBeDefined()
		expect(screen.getByText('Lyon')).toBeDefined()
	})

	it('goes back to the list from the back button', async () => {
		const onBack = vi.fn()
		const { user } = await renderTicket({ onBack })

		await user.click(screen.getByRole('button', { name: 'Retour aux tickets' }))

		expect(onBack).toHaveBeenCalledTimes(1)
	})

	it('shows the error of a failed thread and reads it again on retry', async () => {
		let reads = 0
		const { user } = await renderTicket({
			ready: false,
			configure: (api, server) =>
				api.mockGet(ENTRIES_PATH, () => {
					reads += 1
					if (reads === 1) throw new Error('Le fil est indisponible.')
					return { data: [...server.entries].reverse(), pagination: null }
				}),
		})

		expect(await screen.findByText('Le fil est indisponible.')).toBeDefined()

		await user.click(screen.getByRole('button', { name: 'Réessayer' }))

		expect(
			await screen.findByRole('article', { name: 'Note interne, Jo Durand' }),
		).toBeDefined()
		expect(screen.queryByText('Le fil est indisponible.')).toBeNull()
	})

	it('shows the error of a failed ticket and reads it again on retry', async () => {
		let reads = 0
		const { user } = await renderTicket({
			ready: false,
			configure: (api, state) =>
				api.mockGet(TICKET_PATH, () => {
					reads += 1
					if (reads === 1) throw new Error('Le ticket est indisponible.')
					return { data: state.ticket }
				}),
		})

		expect(await screen.findByText('Le ticket est indisponible.')).toBeDefined()

		await user.click(screen.getByRole('button', { name: 'Réessayer' }))

		expect(
			await screen.findByRole('heading', {
				level: 1,
				name: '#7 Fuite sous évier',
			}),
		).toBeDefined()
		expect(screen.queryByText('Le ticket est indisponible.')).toBeNull()
	})
})

describe('SupportTicketFeature — composeur', () => {
	it('posts a reply from the default tab and clears the field', async () => {
		const { user, calls } = await renderTicket()

		await user.type(messageField(), 'Nous passons jeudi.')
		await user.click(screen.getByRole('button', { name: 'Envoyer la réponse' }))

		await waitFor(() =>
			expect(
				calls.filter(
					(call) => call.method === 'post' && call.path === ENTRIES_PATH,
				),
			).toHaveLength(1),
		)
		expect(
			calls.find((call) => call.method === 'post' && call.path === ENTRIES_PATH)
				?.params,
		).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { kind: 'reply', body: 'Nous passons jeudi.' },
		})
		await waitFor(() => expect(messageField().value).toBe(''))
		expect(await screen.findByText('Nous passons jeudi.')).toBeDefined()
	})

	it('posts a draft from the Brouillon tab and clears the field', async () => {
		const api = await renderTicket()
		const { user } = api

		await user.click(screen.getByRole('tab', { name: 'Brouillon' }))
		await user.type(messageField(), 'À relire avant envoi.')
		await user.click(
			screen.getByRole('button', { name: 'Enregistrer le brouillon' }),
		)

		await waitFor(() =>
			expect(callsTo(api, 'post', ENTRIES_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', ENTRIES_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { kind: 'draft', body: 'À relire avant envoi.' },
		})
		await waitFor(() => expect(messageField().value).toBe(''))
	})

	it('posts an internal note from the Note interne tab and clears the field', async () => {
		const api = await renderTicket()
		const { user } = api

		await user.click(screen.getByRole('tab', { name: 'Note interne' }))
		await user.type(messageField(), 'Rappeler le client lundi.')
		await user.click(screen.getByRole('button', { name: 'Ajouter la note' }))

		await waitFor(() =>
			expect(callsTo(api, 'post', ENTRIES_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', ENTRIES_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { kind: 'note', body: 'Rappeler le client lundi.' },
		})
		await waitFor(() => expect(messageField().value).toBe(''))
	})

	it('sends nothing for an empty or whitespace-only body', async () => {
		const api = await renderTicket()
		const { user } = api
		const submit = screen.getByRole('button', { name: 'Envoyer la réponse' })

		expect((submit as HTMLButtonElement).disabled).toBe(true)

		await user.type(messageField(), '   ')
		expect((submit as HTMLButtonElement).disabled).toBe(true)
		await user.click(submit)

		expect(callsTo(api, 'post', ENTRIES_PATH)).toHaveLength(0)
	})

	it('keeps the body and shows the error when the post fails', async () => {
		const api = await renderTicket({
			configure: (fakeApi) =>
				fakeApi.mockMutation('post', ENTRIES_PATH, () => {
					throw new Error('Le serveur a refusé le message.')
				}),
		})

		await api.user.type(messageField(), 'Un message')
		await api.user.click(
			screen.getByRole('button', { name: 'Envoyer la réponse' }),
		)

		expect(
			await screen.findByText('Le serveur a refusé le message.'),
		).toBeDefined()
		expect(messageField().value).toBe('Un message')
	})
})

describe('SupportTicketFeature — brouillons', () => {
	it('sends a draft to the customer', async () => {
		const api = await renderTicket()

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Envoyer' }),
		)

		await waitFor(() =>
			expect(callsTo(api, 'post', ENTRY_SEND_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', ENTRY_SEND_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', entry_id: 'entry-4' },
		})
		expect(
			await screen.findAllByRole('article', {
				name: 'Réponse envoyée, Alix Martin',
			}),
		).toHaveLength(2)
	})

	it('edits a draft with the new body and leaves edit mode', async () => {
		const api = await renderTicket()

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Modifier' }),
		)
		const editor = screen.getByRole('textbox', {
			name: 'Modifier le brouillon',
		})
		expect((editor as HTMLTextAreaElement).value).toBe('Brouillon de suivi.')
		await api.user.clear(editor)
		await api.user.type(editor, 'Brouillon corrigé.')
		await api.user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() =>
			expect(callsTo(api, 'patch', ENTRY_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'patch', ENTRY_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', entry_id: 'entry-4' },
			body: { body: 'Brouillon corrigé.' },
		})
		await waitFor(() =>
			expect(
				screen.queryByRole('textbox', { name: 'Modifier le brouillon' }),
			).toBeNull(),
		)
		expect(await screen.findByText('Brouillon corrigé.')).toBeDefined()
	})

	it('cancels an edit without sending anything', async () => {
		const api = await renderTicket()

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Modifier' }),
		)
		await api.user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(
			screen.queryByRole('textbox', { name: 'Modifier le brouillon' }),
		).toBeNull()
		expect(callsTo(api, 'patch', ENTRY_PATH)).toHaveLength(0)
	})

	it('sends nothing for a blank edit', async () => {
		const api = await renderTicket()

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Modifier' }),
		)
		const editor = screen.getByRole('textbox', {
			name: 'Modifier le brouillon',
		})
		await api.user.clear(editor)
		await api.user.type(editor, '   ')
		const save = screen.getByRole('button', { name: 'Enregistrer' })
		await api.user.click(save)

		expect((save as HTMLButtonElement).disabled).toBe(true)
		expect(callsTo(api, 'patch', ENTRY_PATH)).toHaveLength(0)
	})

	it('stays in edit mode and shows the error when the edit fails', async () => {
		const api = await renderTicket({
			configure: (fakeApi) =>
				fakeApi.mockMutation('patch', ENTRY_PATH, () => {
					throw new Error('Le brouillon est verrouillé.')
				}),
		})

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Modifier' }),
		)
		await api.user.type(
			screen.getByRole('textbox', { name: 'Modifier le brouillon' }),
			' bis',
		)
		await api.user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(
			await screen.findByText('Le brouillon est verrouillé.'),
		).toBeDefined()
		expect(
			screen.getByRole('textbox', { name: 'Modifier le brouillon' }),
		).toBeDefined()
	})

	it('deletes a draft', async () => {
		const api = await renderTicket()

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Supprimer' }),
		)

		await waitFor(() =>
			expect(callsTo(api, 'delete', ENTRY_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'delete', ENTRY_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', entry_id: 'entry-4' },
		})
		await waitFor(() =>
			expect(screen.queryByText('Brouillon de suivi.')).toBeNull(),
		)
	})

	it.each([
		['sending', 'post', ENTRY_SEND_PATH, 'Envoyer'],
		['deleting', 'delete', ENTRY_PATH, 'Supprimer'],
	])('locks the draft while %s it', async (_label, method, path, action) => {
		const pending = deferred<unknown>()
		const api = await renderTicket({
			configure: (fakeApi) =>
				fakeApi.mockMutation(method, path, () => pending.promise),
		})

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: action }),
		)

		await waitFor(() => expect(callsTo(api, method, path)).toHaveLength(1))
		for (const name of ['Modifier', 'Envoyer', 'Supprimer']) {
			expect(
				(
					within(draftArticle()).getByRole('button', {
						name,
					}) as HTMLButtonElement
				).disabled,
			).toBe(true)
		}

		pending.resolve({})
	})

	it('locks the editor while saving', async () => {
		const pending = deferred<unknown>()
		const api = await renderTicket({
			configure: (fakeApi) =>
				fakeApi.mockMutation('patch', ENTRY_PATH, () => pending.promise),
		})

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Modifier' }),
		)
		await api.user.type(
			screen.getByRole('textbox', { name: 'Modifier le brouillon' }),
			' bis',
		)
		await api.user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() =>
			expect(callsTo(api, 'patch', ENTRY_PATH)).toHaveLength(1),
		)
		expect(
			(screen.getByRole('button', { name: 'Enregistrer' }) as HTMLButtonElement)
				.disabled,
		).toBe(true)
		expect(
			(screen.getByRole('button', { name: 'Annuler' }) as HTMLButtonElement)
				.disabled,
		).toBe(true)

		pending.resolve({})
	})
})

describe('SupportTicketFeature — panneau de détails', () => {
	it('moves the ticket to the chosen status', async () => {
		const api = await renderTicket()

		await api.user.selectOptions(screen.getByLabelText('Statut'), 'in_progress')

		await waitFor(() =>
			expect(callsTo(api, 'patch', STATUS_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'patch', STATUS_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { status: 'in_progress' },
		})
		await waitFor(() =>
			expect((screen.getByLabelText('Statut') as HTMLSelectElement).value).toBe(
				'in_progress',
			),
		)
	})

	it('offers the four statuses in column order', async () => {
		await renderTicket()

		expect(
			within(screen.getByLabelText('Statut'))
				.getAllByRole('option')
				.map((option) => option.textContent),
		).toEqual(['Nouveau', 'En cours', 'En attente du client', 'Résolu'])
	})

	it('assigns the ticket to the chosen member', async () => {
		const api = await renderTicket()
		await screen.findByRole('option', { name: 'Jo Durand' })

		await api.user.selectOptions(screen.getByLabelText('Assigné à'), 'member-2')

		await waitFor(() =>
			expect(callsTo(api, 'put', ASSIGNEE_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'put', ASSIGNEE_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { member_id: 'member-2' },
		})
	})

	it('unassigns the ticket with a null member for Non assigné', async () => {
		const api = await renderTicket({
			configure: (_fakeApi, server) => {
				server.ticket = ticket({ assignee_member_id: 'member-1' })
			},
		})
		await screen.findByRole('option', { name: 'Alix Martin' })

		await api.user.selectOptions(
			screen.getByLabelText('Assigné à'),
			'Non assigné',
		)

		await waitFor(() =>
			expect(callsTo(api, 'put', ASSIGNEE_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'put', ASSIGNEE_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { member_id: null },
		})
	})

	it('shows the error of a refused status change', async () => {
		const api = await renderTicket({
			configure: (fakeApi) =>
				fakeApi.mockMutation('patch', STATUS_PATH, () => {
					throw new Error('Transition refusée.')
				}),
		})

		await api.user.selectOptions(screen.getByLabelText('Statut'), 'resolved')

		expect(await screen.findByText('Transition refusée.')).toBeDefined()
	})

	it('shows the error of the most recent failure of the two controls', async () => {
		const api = await renderTicket({
			configure: (fakeApi) => {
				fakeApi.mockMutation('patch', STATUS_PATH, () => {
					throw new Error('Transition refusée.')
				})
				fakeApi.mockMutation('put', ASSIGNEE_PATH, () => {
					throw new Error('Assignation refusée.')
				})
			},
		})
		await screen.findByRole('option', { name: 'Jo Durand' })

		await api.user.selectOptions(screen.getByLabelText('Statut'), 'resolved')
		expect(await screen.findByText('Transition refusée.')).toBeDefined()

		await api.user.selectOptions(screen.getByLabelText('Assigné à'), 'member-2')

		expect(await screen.findByText('Assignation refusée.')).toBeDefined()
		expect(screen.queryByText('Transition refusée.')).toBeNull()
	})

	it('locks both controls while an update is in flight', async () => {
		const pending = deferred<unknown>()
		const api = await renderTicket({
			configure: (fakeApi) =>
				fakeApi.mockMutation('patch', STATUS_PATH, () => pending.promise),
		})

		await api.user.selectOptions(screen.getByLabelText('Statut'), 'resolved')

		await waitFor(() =>
			expect(
				(screen.getByLabelText('Statut') as HTMLSelectElement).disabled,
			).toBe(true),
		)
		expect(
			(screen.getByLabelText('Assigné à') as HTMLSelectElement).disabled,
		).toBe(true)

		pending.resolve({})
	})
})

describe('SupportTicketFeature — sans MANAGE_SUPPORT', () => {
	it('reads the thread but offers no way to write', async () => {
		await renderTicket({ permissions: ['VIEW_SUPPORT'] })

		expect(screen.getAllByRole('article')).toHaveLength(4)
		expect(screen.queryByRole('tablist')).toBeNull()
		expect(screen.queryByRole('textbox')).toBeNull()
		expect(screen.queryByRole('combobox')).toBeNull()
		for (const name of ['Modifier', 'Envoyer', 'Supprimer']) {
			expect(screen.queryByRole('button', { name })).toBeNull()
		}
	})

	it('shows the status and the assignee as plain text', async () => {
		await renderTicket({
			permissions: ['VIEW_SUPPORT'],
			configure: (_fakeApi, server) => {
				server.ticket = ticket({ assignee_member_id: 'member-2' })
			},
		})

		expect(screen.getAllByText('Nouveau').length).toBeGreaterThan(0)
		expect(screen.getAllByText('Jo Durand').length).toBeGreaterThan(0)
	})
})

describe('SupportTicketFeature — pièces jointes', () => {
	let openSpy: ReturnType<typeof stubWindowOpen>

	function stubWindowOpen() {
		return vi.spyOn(window, 'open').mockReturnValue(null)
	}

	beforeEach(() => {
		openSpy = stubWindowOpen()
	})

	afterEach(() => {
		openSpy.mockRestore()
	})

	it('fetches the url only when the attachment is clicked, then opens it', async () => {
		const api = await renderTicket()

		expect(callsTo(api, 'get', ATTACHMENT_URL_PATH)).toHaveLength(0)

		await api.user.click(
			screen.getByRole('button', { name: 'photo-fuite.jpg (2 Ko)' }),
		)

		await waitFor(() =>
			expect(openSpy).toHaveBeenCalledWith(
				ATTACHMENT_URL,
				'_blank',
				'noopener,noreferrer',
			),
		)
		expect(callsTo(api, 'get', ATTACHMENT_URL_PATH)).toHaveLength(1)
		expect(callsTo(api, 'get', ATTACHMENT_URL_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', attachment_id: 'attachment-1' },
		})
		expect(screen.queryByText(ATTACHMENT_ERROR)).toBeNull()
	})

	it('says so when the url cannot be fetched, and clears it on the next success', async () => {
		let failing = true
		const api = await renderTicket({
			configure: (fakeApi) =>
				fakeApi.mockGet(ATTACHMENT_URL_PATH, () => {
					if (failing) throw new Error('indisponible')
					return {
						data: { url: ATTACHMENT_URL, expires_at: '2026-11-01T12:00:00Z' },
					}
				}),
		})
		const attachment = screen.getByRole('button', {
			name: 'photo-fuite.jpg (2 Ko)',
		})

		await api.user.click(attachment)

		expect(await screen.findByText(ATTACHMENT_ERROR)).toBeDefined()
		expect(openSpy).not.toHaveBeenCalled()

		failing = false
		await api.user.click(attachment)

		await waitFor(() => expect(openSpy).toHaveBeenCalledTimes(1))
		await waitFor(() => expect(screen.queryByText(ATTACHMENT_ERROR)).toBeNull())
	})

	it('reads the thread again from the retry shown beside the attachment error', async () => {
		let reads = 0
		const api = await renderTicket({
			configure: (fakeApi, server) => {
				fakeApi.mockGet(ATTACHMENT_URL_PATH, () => {
					throw new Error('indisponible')
				})
				fakeApi.mockGet(ENTRIES_PATH, () => {
					reads += 1
					return { data: [...server.entries].reverse(), pagination: null }
				})
			},
		})

		await api.user.click(
			screen.getByRole('button', { name: 'photo-fuite.jpg (2 Ko)' }),
		)
		expect(await screen.findByText(ATTACHMENT_ERROR)).toBeDefined()

		const readsBefore = reads
		await api.user.click(screen.getByRole('button', { name: 'Réessayer' }))

		await waitFor(() => expect(reads).toBeGreaterThan(readsBefore))
	})
})

describe('SupportTicketFeature — organisation', () => {
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
				<SupportTicketFeature ticketId="ticket-1" onBack={() => {}} />
			</QueryClientProvider>,
		)

		expect(screen.getByText('Organisation indisponible')).toBeDefined()
		expect(
			screen.getByText('Le support nécessite une organisation active.'),
		).toBeDefined()
		expect(api.calls).toEqual([])
	})
})
