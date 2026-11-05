import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
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
import { resetDraftReviews } from '#/pages/support/feature/mocks/draft-reviews'
import { resetTicketPriorities } from '#/pages/support/feature/mocks/priorities'
import {
	ALL_PERMISSIONS,
	ASSIGNEE_PATH,
	ATTACHMENT_URL_PATH,
	BADGE_PATH,
	BADGES_PATH,
	callsTo,
	chooseOption,
	createQueryClient,
	deferred,
	ENTRIES_PATH,
	ENTRY_PATH,
	ENTRY_SEND_PATH,
	entry,
	type FakeApi,
	installFakeTanstackApi,
	installRadixPolyfills,
	MEMBERS,
	MEMBERS_PATH,
	Providers,
	STATUS_PATH,
	TICKET_PATH,
	ticket,
} from '#/pages/support/feature/support-test-kit'
import { SupportTicketFeature } from '#/pages/support/feature/support-ticket-feature'

vi.mock('#/hooks/use-active-organization', async (importOriginal) => {
	const actual =
		await importOriginal<typeof import('#/hooks/use-active-organization')>()
	return {
		...actual,
		useActiveOrganization: vi.fn(actual.useActiveOrganization),
	}
})

const ATTACHMENT_URL = 'https://files.example/photo-fuite.jpg'
const ATTACHMENT_ERROR = 'Impossible d’ouvrir la pièce jointe.'

beforeAll(installRadixPolyfills)
beforeEach(() => {
	resetDraftReviews()
	resetTicketPriorities()
})

function sampleTicket(): Schemas.TicketResponse {
	return ticket({
		id: 'ticket-1',
		number: 7,
		subject: 'Fuite sous évier',
		body: 'Bonjour, ma canalisation fuit.',
		custom_fields: { ville: 'Lyon', requester_name: 'Claire Fontaine' },
		badges: [{ label: 'Urgent', tone: 'error' }],
	})
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
	api.mockMutation('post', BADGES_PATH, (params) => {
		const { body } = params as { body: { label: string } }
		server.ticket = {
			...server.ticket,
			badges: [...server.ticket.badges, { label: body.label, tone: 'neutral' }],
		}
		return { data: server.ticket }
	})
	api.mockMutation('delete', BADGE_PATH, (params) => {
		const { path } = params as { path: { label: string } }
		server.ticket = {
			...server.ticket,
			badges: server.ticket.badges.filter(
				(badge) => badge.label !== path.label,
			),
		}
		return { data: server.ticket }
	})
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
	const server: ServerState = {
		ticket: sampleTicket(),
		entries: serverEntries(),
	}
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

	render(
		<Providers queryClient={createQueryClient(permissions)}>
			<SupportTicketFeature ticketId="ticket-1" onBack={onBack} />
		</Providers>,
	)

	if (ready) {
		await screen.findByRole('article', {
			name: 'Envoyé au client de Alix Martin',
		})
	}

	return { ...api, server, user: userEvent.setup() }
}

function replyField(): HTMLTextAreaElement {
	return screen.getByRole('textbox', {
		name: 'Réponse au client',
	}) as HTMLTextAreaElement
}

function composer(): HTMLElement {
	return screen.getByRole('region', { name: 'Écrire' })
}

function draftArticle(): HTMLElement {
	return screen.getByRole('article', { name: 'Brouillon de Alix Martin' })
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
			'Client de Claire Fontaine',
			'Envoyé au client de Alix Martin',
			'Note interne de Jo Durand',
			'Brouillon de Alix Martin',
		])
		articles.forEach((article, index) => {
			expect(article.textContent).toContain(expectedBodies[index])
		})
	})

	it('titles the page with the subject and the number of the ticket', async () => {
		await renderTicket()

		expect(screen.getByRole('heading', { level: 1 }).textContent).toBe(
			'Fuite sous évier',
		)
		expect(
			within(
				screen.getByRole('navigation', { name: 'Emplacement du ticket' }),
			).getByText('#7'),
		).toBeDefined()
	})

	it('shows the headline of the ticket: status, requester, source and badges', async () => {
		await renderTicket()

		const headline = screen.getByRole('heading', {
			level: 1,
		}).nextElementSibling
		expect(headline?.textContent).toContain('Backlog')
		expect(headline?.textContent).toContain(
			'Claire Fontaine via saisie manuelle',
		)
		expect(headline?.textContent).toContain('Urgent')
	})

	it('shows the custom fields and the origin in the side panel', async () => {
		await renderTicket()

		const panel = screen.getByRole('complementary', {
			name: 'Propriétés du ticket',
		})
		expect(within(panel).getByText('ville')).toBeDefined()
		expect(within(panel).getByText('Lyon')).toBeDefined()
		expect(within(panel).getByText('Saisie manuelle')).toBeDefined()
	})

	it('goes back to the list from the breadcrumb', async () => {
		const onBack = vi.fn()
		const { user } = await renderTicket({ onBack })

		await user.click(screen.getByRole('button', { name: 'Tickets' }))

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
			await screen.findByRole('article', { name: 'Note interne de Jo Durand' }),
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
				name: 'Fuite sous évier',
			}),
		).toBeDefined()
		expect(screen.queryByText('Le ticket est indisponible.')).toBeNull()
	})

	it('says the ticket is unknown when the API returns nothing', async () => {
		const onBack = vi.fn()
		const { user } = await renderTicket({
			ready: false,
			onBack,
			configure: (api) => api.mockGet(TICKET_PATH, () => ({ data: undefined })),
		})

		expect(await screen.findByText('Ticket introuvable')).toBeDefined()
		await user.click(screen.getByRole('button', { name: 'Voir les tickets' }))

		expect(onBack).toHaveBeenCalledTimes(1)
	})
})

describe('SupportTicketFeature — composeur', () => {
	it('asks for a confirmation before sending a reply, then moves the ticket on', async () => {
		const { user, calls } = await renderTicket()

		await user.type(replyField(), 'Nous passons jeudi.')
		await user.click(
			within(composer()).getByRole('button', { name: 'Envoyer au client' }),
		)

		expect(
			calls.filter(
				(call) => call.method === 'post' && call.path === ENTRIES_PATH,
			),
		).toHaveLength(0)

		await user.click(
			within(composer()).getByRole('button', { name: 'Confirmer l’envoi' }),
		)

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
		await waitFor(() => expect(replyField().value).toBe(''))
		expect(await screen.findByText('Nous passons jeudi.')).toBeDefined()
		await waitFor(() =>
			expect(
				calls.filter(
					(call) => call.method === 'patch' && call.path === STATUS_PATH,
				),
			).toHaveLength(1),
		)
		expect(
			calls.find((call) => call.method === 'patch' && call.path === STATUS_PATH)
				?.params,
		).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { status: 'waiting_customer' },
		})
	})

	it('sends the status picked for after the reply', async () => {
		const api = await renderTicket()

		await api.user.type(replyField(), 'C’est réglé.')
		await chooseOption(
			api.user,
			screen.getByLabelText('Statut après l’envoi'),
			'Fermé',
		)
		await api.user.click(
			within(composer()).getByRole('button', { name: 'Envoyer au client' }),
		)
		await api.user.click(
			within(composer()).getByRole('button', { name: 'Confirmer l’envoi' }),
		)

		await waitFor(() =>
			expect(callsTo(api, 'patch', STATUS_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'patch', STATUS_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { status: 'resolved' },
		})
	})

	it('leaves the status alone when it already is the one picked', async () => {
		const api = await renderTicket({
			configure: (_fakeApi, server) => {
				server.ticket = { ...server.ticket, status: 'waiting_customer' }
			},
		})

		await api.user.type(replyField(), 'Un message.')
		await api.user.click(
			within(composer()).getByRole('button', { name: 'Envoyer au client' }),
		)
		await api.user.click(
			within(composer()).getByRole('button', { name: 'Confirmer l’envoi' }),
		)

		await waitFor(() =>
			expect(callsTo(api, 'post', ENTRIES_PATH)).toHaveLength(1),
		)
		await waitFor(() => expect(replyField().value).toBe(''))
		expect(callsTo(api, 'patch', STATUS_PATH)).toHaveLength(0)
	})

	it('saves a draft from the reply tab and clears the field', async () => {
		const api = await renderTicket()

		await api.user.type(replyField(), 'À relire avant envoi.')
		await api.user.click(
			screen.getByRole('button', { name: 'Enregistrer en brouillon' }),
		)

		await waitFor(() =>
			expect(callsTo(api, 'post', ENTRIES_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', ENTRIES_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { kind: 'draft', body: 'À relire avant envoi.' },
		})
		await waitFor(() => expect(replyField().value).toBe(''))
		expect(callsTo(api, 'patch', STATUS_PATH)).toHaveLength(0)
	})

	it('posts an internal note from the Note interne mode and clears the field', async () => {
		const api = await renderTicket()

		await api.user.click(screen.getByRole('radio', { name: 'Note interne' }))
		await api.user.type(
			screen.getByRole('textbox', { name: 'Note interne' }),
			'Rappeler le client lundi.',
		)
		await api.user.click(
			screen.getByRole('button', { name: 'Ajouter la note' }),
		)

		await waitFor(() =>
			expect(callsTo(api, 'post', ENTRIES_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', ENTRIES_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { kind: 'note', body: 'Rappeler le client lundi.' },
		})
		await waitFor(() => expect(replyField().value).toBe(''))
	})

	it('sends nothing for an empty or whitespace-only body', async () => {
		const api = await renderTicket()
		const send = within(composer()).getByRole('button', {
			name: 'Envoyer au client',
		})

		expect((send as HTMLButtonElement).disabled).toBe(true)

		await api.user.type(replyField(), '   ')
		expect((send as HTMLButtonElement).disabled).toBe(true)
		await api.user.click(send)

		expect(callsTo(api, 'post', ENTRIES_PATH)).toHaveLength(0)
	})

	it('disarms the confirmation when the text changes', async () => {
		const api = await renderTicket()

		await api.user.type(replyField(), 'Un')
		await api.user.click(
			within(composer()).getByRole('button', { name: 'Envoyer au client' }),
		)
		expect(
			within(composer()).getByRole('button', { name: 'Confirmer l’envoi' }),
		).toBeDefined()

		await api.user.type(replyField(), ' mot')

		expect(
			within(composer()).getByRole('button', { name: 'Envoyer au client' }),
		).toBeDefined()
		expect(callsTo(api, 'post', ENTRIES_PATH)).toHaveLength(0)
	})

	it('keeps the body and shows the error when the post fails', async () => {
		const api = await renderTicket({
			configure: (fakeApi) =>
				fakeApi.mockMutation('post', ENTRIES_PATH, () => {
					throw new Error('Le serveur a refusé le message.')
				}),
		})

		await api.user.type(replyField(), 'Un message')
		await api.user.click(
			within(composer()).getByRole('button', { name: 'Envoyer au client' }),
		)
		await api.user.click(
			within(composer()).getByRole('button', { name: 'Confirmer l’envoi' }),
		)

		expect(
			await screen.findByText('Le serveur a refusé le message.'),
		).toBeDefined()
		expect(replyField().value).toBe('Un message')
	})
})

describe('SupportTicketFeature — brouillons', () => {
	it('sends a draft to the customer after a confirmation', async () => {
		const api = await renderTicket()

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Envoyer au client' }),
		)
		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Confirmer l’envoi' }),
		)

		await waitFor(() =>
			expect(callsTo(api, 'post', ENTRY_SEND_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', ENTRY_SEND_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', entry_id: 'entry-4' },
		})
		expect(
			await screen.findAllByRole('article', {
				name: 'Envoyé au client de Alix Martin',
			}),
		).toHaveLength(2)
	})

	it('loads a draft in the composer, then updates it', async () => {
		const api = await renderTicket()

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Modifier' }),
		)
		expect(replyField().value).toBe('Brouillon de suivi.')
		expect(screen.getByText(/Modification d’un brouillon/)).toBeDefined()
		await api.user.clear(replyField())
		await api.user.type(replyField(), 'Brouillon corrigé.')
		await api.user.click(
			screen.getByRole('button', { name: 'Mettre à jour le brouillon' }),
		)

		await waitFor(() =>
			expect(callsTo(api, 'patch', ENTRY_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'patch', ENTRY_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', entry_id: 'entry-4' },
			body: { body: 'Brouillon corrigé.' },
		})
		await waitFor(() =>
			expect(screen.queryByText(/Modification d’un brouillon/)).toBeNull(),
		)
		expect(await screen.findByText('Brouillon corrigé.')).toBeDefined()
	})

	it('updates a draft and sends it from the composer', async () => {
		const api = await renderTicket()

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Modifier' }),
		)
		await api.user.type(replyField(), ' bis')
		await api.user.click(
			within(composer()).getByRole('button', { name: 'Envoyer au client' }),
		)
		await api.user.click(
			within(composer()).getByRole('button', { name: 'Confirmer l’envoi' }),
		)

		await waitFor(() =>
			expect(callsTo(api, 'post', ENTRY_SEND_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'patch', ENTRY_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', entry_id: 'entry-4' },
			body: { body: 'Brouillon de suivi. bis' },
		})
		expect(callsTo(api, 'post', ENTRIES_PATH)).toHaveLength(0)
	})

	it('cancels an edit without sending anything', async () => {
		const api = await renderTicket()

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Modifier' }),
		)
		await api.user.click(screen.getByRole('button', { name: 'Annuler' }))

		expect(screen.queryByText(/Modification d’un brouillon/)).toBeNull()
		expect(replyField().value).toBe('')
		expect(callsTo(api, 'patch', ENTRY_PATH)).toHaveLength(0)
	})

	it('sends nothing for a blank edit', async () => {
		const api = await renderTicket()

		await api.user.click(
			within(draftArticle()).getByRole('button', { name: 'Modifier' }),
		)
		await api.user.clear(replyField())
		await api.user.type(replyField(), '   ')
		const save = screen.getByRole('button', {
			name: 'Mettre à jour le brouillon',
		})
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
		await api.user.type(replyField(), ' bis')
		await api.user.click(
			screen.getByRole('button', { name: 'Mettre à jour le brouillon' }),
		)

		expect(
			await screen.findByText('Le brouillon est verrouillé.'),
		).toBeDefined()
		expect(screen.getByText(/Modification d’un brouillon/)).toBeDefined()
	})

	it('deletes a draft', async () => {
		const api = await renderTicket()

		await api.user.click(
			within(draftArticle()).getByRole('button', {
				name: 'Supprimer le brouillon',
			}),
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

	it('locks the draft while deleting it', async () => {
		const pending = deferred<unknown>()
		const api = await renderTicket({
			configure: (fakeApi) =>
				fakeApi.mockMutation('delete', ENTRY_PATH, () => pending.promise),
		})

		await api.user.click(
			within(draftArticle()).getByRole('button', {
				name: 'Supprimer le brouillon',
			}),
		)

		await waitFor(() =>
			expect(callsTo(api, 'delete', ENTRY_PATH)).toHaveLength(1),
		)
		for (const name of ['Modifier', 'Supprimer le brouillon']) {
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
})

describe('SupportTicketFeature — avis sur un brouillon (maquette)', () => {
	it('approves a draft and withdraws the approval', async () => {
		const { user } = await renderTicket()

		expect(
			within(draftArticle()).getByText('Pas encore d’approbation'),
		).toBeDefined()
		await user.click(
			within(draftArticle()).getByRole('button', { name: 'Approuver' }),
		)

		expect(
			await within(draftArticle()).findByText('Approuvé par Alix Martin'),
		).toBeDefined()
		await user.click(
			within(draftArticle()).getByRole('button', { name: 'Approuvé' }),
		)

		expect(
			await within(draftArticle()).findByText('Pas encore d’approbation'),
		).toBeDefined()
	})

	it('publishes an opinion on a draft', async () => {
		const { user } = await renderTicket()

		await user.click(
			within(draftArticle()).getByRole('button', { name: 'Donner un avis' }),
		)
		await user.type(
			within(draftArticle()).getByLabelText('Avis sur le brouillon'),
			'Plus court ?',
		)
		await user.click(
			within(draftArticle()).getByRole('button', { name: 'Publier' }),
		)

		expect(
			await within(draftArticle()).findByText(': Plus court ?'),
		).toBeDefined()
	})
})

describe('SupportTicketFeature — panneau de propriétés', () => {
	function statusTrigger() {
		return screen.getByRole('combobox', { name: 'Statut du ticket' })
	}

	it('moves the ticket to the chosen status', async () => {
		const api = await renderTicket()

		await chooseOption(api.user, statusTrigger(), 'Ouvert')

		await waitFor(() =>
			expect(callsTo(api, 'patch', STATUS_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'patch', STATUS_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { status: 'in_progress' },
		})
		await waitFor(() => expect(statusTrigger().textContent).toContain('Ouvert'))
	})

	it('offers the four statuses in column order', async () => {
		const api = await renderTicket()

		await api.user.click(statusTrigger())

		expect(
			(await screen.findAllByRole('option')).map(
				(option) => option.textContent,
			),
		).toEqual(['Backlog', 'Ouvert', 'En attente client', 'Fermé'])
	})

	it('changes the priority', async () => {
		const api = await renderTicket()
		const trigger = screen.getByRole('combobox', { name: 'Priorité du ticket' })
		expect(trigger.textContent).toContain('Normale')

		await chooseOption(api.user, trigger, 'Urgente')

		await waitFor(() => expect(trigger.textContent).toContain('Urgente'))
	})

	it('assigns the ticket to the chosen member', async () => {
		const api = await renderTicket()

		await api.user.click(screen.getByLabelText('Modifier les assignés'))
		await api.user.click(
			await screen.findByRole('option', { name: /Jo Durand/ }),
		)

		await waitFor(() =>
			expect(callsTo(api, 'put', ASSIGNEE_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'put', ASSIGNEE_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { member_id: 'member-2' },
		})
	})

	it('unassigns the ticket with a null member when its assignee is picked again', async () => {
		const api = await renderTicket({
			configure: (_fakeApi, server) => {
				server.ticket = { ...server.ticket, assignee_member_id: 'member-1' }
			},
		})

		await api.user.click(screen.getByLabelText('Modifier les assignés'))
		await api.user.click(
			await screen.findByRole('option', { name: /Alix Martin/ }),
		)

		await waitFor(() =>
			expect(callsTo(api, 'put', ASSIGNEE_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'put', ASSIGNEE_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { member_id: null },
		})
	})

	it('adds a badge created from the typed label', async () => {
		const api = await renderTicket()

		await api.user.click(screen.getByRole('button', { name: 'Ajouter' }))
		await api.user.type(
			await screen.findByPlaceholderText('Chercher ou créer un badge…'),
			'Garantie',
		)
		await api.user.click(await screen.findByText('Créer « Garantie »'))

		await waitFor(() =>
			expect(callsTo(api, 'post', BADGES_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'post', BADGES_PATH)[0].params).toEqual({
			path: { organization_id: 'org-1', ticket_id: 'ticket-1' },
			body: { label: 'Garantie' },
		})
	})

	it('removes a badge', async () => {
		const api = await renderTicket()

		await api.user.click(screen.getByLabelText('Retirer le badge Urgent'))

		await waitFor(() =>
			expect(callsTo(api, 'delete', BADGE_PATH)).toHaveLength(1),
		)
		expect(callsTo(api, 'delete', BADGE_PATH)[0].params).toEqual({
			path: {
				organization_id: 'org-1',
				ticket_id: 'ticket-1',
				label: 'Urgent',
			},
		})
	})

	it('shows the error of a refused status change', async () => {
		const api = await renderTicket({
			configure: (fakeApi) =>
				fakeApi.mockMutation('patch', STATUS_PATH, () => {
					throw new Error('Transition refusée.')
				}),
		})

		await chooseOption(api.user, statusTrigger(), 'Fermé')

		expect(await screen.findByText('Transition refusée.')).toBeDefined()
	})

	it('shows the error of the most recent failure of the controls', async () => {
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

		await chooseOption(api.user, statusTrigger(), 'Fermé')
		expect(await screen.findByText('Transition refusée.')).toBeDefined()

		await api.user.click(screen.getByLabelText('Modifier les assignés'))
		await api.user.click(
			await screen.findByRole('option', { name: /Jo Durand/ }),
		)

		expect(await screen.findByText('Assignation refusée.')).toBeDefined()
		expect(screen.queryByText('Transition refusée.')).toBeNull()
	})
})

describe('SupportTicketFeature — sans MANAGE_SUPPORT', () => {
	it('reads the thread but offers no way to write', async () => {
		await renderTicket({ permissions: ['VIEW_SUPPORT'] })

		expect(screen.getAllByRole('article')).toHaveLength(4)
		expect(screen.queryByRole('textbox')).toBeNull()
		expect(screen.queryByRole('radio')).toBeNull()
		for (const name of [
			'Modifier',
			'Envoyer au client',
			'Supprimer le brouillon',
		]) {
			expect(screen.queryByRole('button', { name })).toBeNull()
		}
	})

	it('locks the property controls', async () => {
		await renderTicket({ permissions: ['VIEW_SUPPORT'] })

		for (const trigger of screen.getAllByRole('combobox')) {
			expect((trigger as HTMLButtonElement).disabled).toBe(true)
		}
		expect(
			(screen.getByLabelText('Modifier les assignés') as HTMLButtonElement)
				.disabled,
		).toBe(true)
		expect(
			(screen.getByLabelText('Retirer le badge Urgent') as HTMLButtonElement)
				.disabled,
		).toBe(true)
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
