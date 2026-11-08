import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useState } from 'react'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { TooltipProvider } from '#/components/ui/tooltip'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { resetRhStore } from '#/pages/hr/feature/mocks/rh-store'
import type {
	AbsenceRecord,
	AbsenceTypeRecord,
} from '#/pages/hr/lib/rh/absence-wire'
import { renderWithRouter } from '#/test/render-with-router'
import {
	ABSENCE_TYPES_PATH,
	ABSENCES_PATH,
	absenceRecord,
	typeRecord,
} from '#/test/rh-absence-fixtures'
import {
	DEFAULT_ACCOUNT_RECORDS,
	LEAVE_ACCOUNTS_PATH,
	LEAVE_ADJUSTMENTS_PATH,
} from '#/test/rh-leave-fixtures'
import { AbsencesFeature } from './absences-feature'

const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const PROFILES_PATH = '/api/v1/organizations/{organization_id}/member-profiles'
const CONTRACTS_PATH =
	'/api/v1/organizations/{organization_id}/member-contracts'
const DOCUMENTS_PATH =
	'/api/v1/organizations/{organization_id}/member-documents'
const PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'
const ARCHIVE_PATH = `${ABSENCE_TYPES_PATH}/{absence_type_id}/archive`
const RESTORE_PATH = `${ABSENCE_TYPES_PATH}/{absence_type_id}/restore`
const TYPE_PATH = `${ABSENCE_TYPES_PATH}/{absence_type_id}`

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
}

function member(id: string, firstName: string, lastName: string) {
	return {
		id,
		organization_id: 'org-1',
		last_name: lastName,
		first_name: firstName,
		display_name: `${lastName} ${firstName}`,
		account: null,
		joined_at: null,
		created_at: '2026-01-01T00:00:00Z',
	}
}

const MEMBERS = [member('u-1', 'Alix', 'Nova'), member('u-2', 'Bao', 'Tran')]

const AUTO = typeRecord('t-auto', 'Intempéries', {
	count_mode: 'CALENDAR',
	needs_approval: false,
	kind: 'UNAVAILABLE',
})
const REQUEST = typeRecord('t-leave', 'Congés payés', {
	count_mode: 'CALENDAR',
})
const PROOF = typeRecord('t-proof', 'Arrêt maladie', {
	count_mode: 'CALENDAR',
	needs_approval: false,
	needs_proof: true,
	kind: 'SICK',
})

interface Write {
	method: string
	path: string
	params: { path?: Record<string, string>; body?: Record<string, unknown> }
}

let types: AbsenceTypeRecord[]
let absences: AbsenceRecord[]
let writes: Write[]
let uploads: number
let refuse: { status: number; data: Record<string, unknown> } | null
let listsForbidden: boolean
let permissions: string[]

function installFakeApi() {
	const fakeApi = {
		get(path: string, params: unknown) {
			const p = (params ?? {}) as { path?: unknown; query?: unknown }
			const queryKey = [{ _id: path, path: p.path, query: p.query }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						if (path === MEMBERS_PATH)
							return { data: MEMBERS, pagination: null }
						if (
							path === PROFILES_PATH ||
							path === CONTRACTS_PATH ||
							path === DOCUMENTS_PATH
						)
							return { data: [], pagination: null }
						if (path === ABSENCE_TYPES_PATH || path === ABSENCES_PATH) {
							if (listsForbidden)
								throw Object.assign(new Error('Forbidden'), { status: 403 })
							return {
								data: structuredClone(
									path === ABSENCES_PATH ? absences : types,
								),
								pagination: null,
							}
						}
						if (path === '/api/v1/files/url')
							return { data: { url: 'https://files.test/proof' } }
						if (path === PERMISSIONS_PATH)
							return { data: { permissions }, pagination: null }
						if (path === LEAVE_ACCOUNTS_PATH)
							return { data: DEFAULT_ACCOUNT_RECORDS, pagination: null }
						if (path === LEAVE_ADJUSTMENTS_PATH)
							return { data: [], pagination: null }
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation(method: string, path: string) {
			const mutationKey = [{ method, path }]
			return {
				mutationKey,
				mutationOptions: {
					mutationKey,
					mutationFn: async (params: Write['params']) => {
						writes.push({ method, path, params })
						if (refuse)
							throw Object.assign(new Error('refused'), {
								status: refuse.status,
								data: refuse.data,
							})
						return applyWrite(method, path, params)
					},
				},
			}
		},
	}
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake of the file upload client
	;(window as any).api = {
		post: async (path: string) => {
			if (path !== '/api/v1/files') throw new Error(`unmocked POST ${path}`)
			uploads += 1
			return { data: { key: 'uploads/proof-file' } }
		},
	}
}

function applyWrite(method: string, path: string, params: Write['params']) {
	const body = (params.body ?? {}) as Record<string, unknown>
	const id = params.path?.absence_id ?? params.path?.absence_type_id ?? ''
	if (method === 'post' && path === ABSENCES_PATH) {
		const type = types.find((candidate) => candidate.id === body.type_id)
		absences = [
			...absences,
			absenceRecord('a-new', String(body.member_id), String(body.type_id), {
				status: type?.needs_approval ? 'PENDING' : 'APPROVED',
				start_date: String(body.start_date),
				end_date: String(body.end_date),
			}),
		]
	}
	if (path.endsWith('/approve'))
		absences = absences.map((row) =>
			row.id === id ? { ...row, status: 'APPROVED' } : row,
		)
	if (path.endsWith('/reject'))
		absences = absences.map((row) =>
			row.id === id
				? {
						...row,
						status: 'REJECTED',
						decided_at: '2026-11-08T10:00:00Z',
						decision_comment: String(body.comment),
					}
				: row,
		)
	if (path.endsWith('/cancel'))
		absences = absences.map((row) =>
			row.id === id ? { ...row, status: 'CANCELLED' } : row,
		)
	if (method === 'post' && path === ABSENCE_TYPES_PATH)
		types = [
			...types,
			{
				...(body as unknown as AbsenceTypeRecord),
				id: 't-created',
				organization_id: 'org-1',
				archived_at: null,
				created_at: '2026-11-08T10:00:00Z',
			},
		]
	if (method === 'put' && path === TYPE_PATH)
		types = types.map((row) =>
			row.id === id ? { ...row, ...(body as object) } : row,
		)
	if (path === ARCHIVE_PATH)
		types = types.map((row) =>
			row.id === id ? { ...row, archived_at: '2026-11-08T10:00:00Z' } : row,
		)
	if (path === RESTORE_PATH)
		types = types.map((row) =>
			row.id === id ? { ...row, archived_at: null } : row,
		)
	return { data: {} }
}

function Harness({
	tab,
	creating,
	initial,
}: {
	tab: 'demandes' | 'motifs'
	creating: boolean
	initial?: { memberId?: string; startDate?: string }
}) {
	const [open, setOpen] = useState(creating)
	return (
		<AbsencesFeature
			tab={tab}
			onTabChange={() => undefined}
			historyStatus="all"
			historyMemberId="all"
			onHistoryFiltersChange={() => undefined}
			creating={open}
			initial={initial}
			onCreatingChange={setOpen}
		/>
	)
}

function renderAbsences(
	props: {
		tab?: 'demandes' | 'motifs'
		creating?: boolean
		initial?: { memberId?: string; startDate?: string }
	} = {},
) {
	installFakeApi()
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	return renderWithRouter(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION as never]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION as never}>
					<TooltipProvider>
						<Harness
							tab={props.tab ?? 'demandes'}
							creating={props.creating ?? false}
							initial={props.initial}
						/>
					</TooltipProvider>
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
	)
}

const FUTURE = '2099-03-02'

function pendingRequest(id = 'a-1', typeId = REQUEST.id) {
	return absenceRecord(id, 'u-1', typeId, {
		start_date: FUTURE,
		end_date: '2099-03-03',
		note: 'Vacances',
	})
}

describe('absence types and requests bound to the API', () => {
	beforeEach(() => {
		resetRhStore()
		types = [AUTO, REQUEST, PROOF]
		absences = []
		writes = []
		uploads = 0
		refuse = null
		listsForbidden = false
		permissions = ['MANAGE_REFERENCE', 'MANAGE_PLANNING']
	})

	afterEach(() => {
		resetRhStore()
	})

	it('spec_rh_3_9 lists the API requests with their type and decides nothing from the prototype', async () => {
		absences = [pendingRequest()]
		await renderAbsences()

		const pending = await screen.findByRole('region', { name: /À valider/ })
		expect(within(pending).getByText('Alix Nova')).toBeDefined()
		expect(within(pending).getAllByText('Congés payés')).not.toHaveLength(0)
		expect(within(pending).getByText('Vacances')).toBeDefined()
		const history = screen.getByRole('region', { name: 'Historique' })
		expect(within(history).getAllByText('Alix Nova')).toHaveLength(1)
		expect(screen.queryByText('Julien Moreau')).toBeNull()
	})

	it('spec_rh_3_9 sends a request in the new shape and a type needing approval gives PENDING', async () => {
		types = [REQUEST, AUTO, PROOF]
		await renderAbsences({
			creating: true,
			initial: { memberId: 'u-2', startDate: FUTURE },
		})
		const user = userEvent.setup()
		const dialog = await screen.findByRole('dialog')
		expect(
			within(dialog).queryByRole('button', { name: 'Enregistrer l’absence' }),
		).toBeNull()
		await user.click(
			within(dialog).getByRole('button', { name: 'Envoyer la demande' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.path).toBe(ABSENCES_PATH)
		const body = writes[0]?.params.body
		expect(body).toMatchObject({
			member_id: 'u-2',
			type_id: REQUEST.id,
			start_date: FUTURE,
			start_part: 'MORNING',
			end_date: FUTURE,
			end_part: 'AFTERNOON',
		})
		expect(body).not.toHaveProperty('kind')
		expect(body).not.toHaveProperty('starts_at')
		await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
		expect(await screen.findAllByText('À valider')).not.toHaveLength(0)
	})

	it('spec_rh_3_9 a type without approval is saved APPROVED at once', async () => {
		types = [AUTO, REQUEST, PROOF]
		await renderAbsences({
			creating: true,
			initial: { memberId: 'u-2', startDate: FUTURE },
		})
		const user = userEvent.setup()
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer l’absence' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.params.body).toMatchObject({ type_id: AUTO.id })
		await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
		const history = screen.getByRole('region', { name: 'Historique' })
		expect(await within(history).findByText('Validée')).toBeDefined()
	})

	it('spec_rh_3_9 uploads the proof through /files and sends its key', async () => {
		types = [PROOF, AUTO, REQUEST]
		await renderAbsences({
			creating: true,
			initial: { memberId: 'u-2', startDate: FUTURE },
		})
		const user = userEvent.setup()
		const dialog = await screen.findByRole('dialog')
		await user.upload(
			within(dialog).getByLabelText(/Justificatif/),
			new File(['pdf'], 'arret.pdf', { type: 'application/pdf' }),
		)
		expect(within(dialog).getByText('arret.pdf')).toBeDefined()
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer l’absence' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(uploads).toBe(1)
		expect(writes[0]?.params.body).toMatchObject({
			type_id: PROOF.id,
			proof_file_key: 'uploads/proof-file',
		})
	})

	it('spec_rh_3_9 shows the proof refusal under the form and keeps the dialog open', async () => {
		types = [PROOF, AUTO, REQUEST]
		refuse = {
			status: 422,
			data: {
				message:
					'proof_file_key: this absence type needs a proof before approval',
				details: { field: 'proof_file_key' },
			},
		}
		await renderAbsences({
			creating: true,
			initial: { memberId: 'u-2', startDate: FUTURE },
		})
		const user = userEvent.setup()
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer l’absence' }),
		)

		expect(
			await within(dialog).findByText(
				'Ce motif exige un justificatif avant la validation.',
			),
		).toBeDefined()
		expect(screen.getByRole('dialog')).toBeDefined()
	})

	it('spec_rh_3_9 approves a pending request through the API', async () => {
		absences = [pendingRequest()]
		await renderAbsences()
		const user = userEvent.setup()

		await user.click(
			await screen.findByRole('button', {
				name: 'Valider la demande de Alix Nova',
			}),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.method).toBe('post')
		expect(writes[0]?.params.path).toEqual({
			organization_id: 'org-1',
			absence_id: 'a-1',
		})
		await waitFor(() =>
			expect(screen.queryByRole('region', { name: /À valider/ })).toBeNull(),
		)
		expect(
			await screen.findByText('Demande de Alix Nova validée.'),
		).toBeDefined()
	})

	it('spec_rh_3_9 shows an approval refused for a missing proof', async () => {
		absences = [pendingRequest('a-1', PROOF.id)]
		refuse = {
			status: 422,
			data: {
				message:
					'proof_file_key: this absence type needs a proof before approval',
				details: { field: 'proof_file_key' },
			},
		}
		await renderAbsences()
		const user = userEvent.setup()

		await user.click(
			await screen.findByRole('button', {
				name: 'Valider la demande de Alix Nova',
			}),
		)

		expect(
			await screen.findByText(
				'Ce motif exige un justificatif avant la validation.',
			),
		).toBeDefined()
		expect(screen.getByRole('region', { name: /À valider/ })).toBeDefined()
	})

	it('spec_rh_3_9 shows an approval of an already decided request', async () => {
		absences = [pendingRequest()]
		refuse = {
			status: 422,
			data: {
				message: 'status: the absence was already decided',
				details: { field: 'status' },
			},
		}
		await renderAbsences()
		const user = userEvent.setup()

		await user.click(
			await screen.findByRole('button', {
				name: 'Valider la demande de Alix Nova',
			}),
		)

		expect(
			await screen.findByText('Cette demande a déjà été traitée.'),
		).toBeDefined()
	})

	it('spec_rh_3_9 rejects with a comment and refuses a blank one before calling the API', async () => {
		absences = [pendingRequest()]
		await renderAbsences()
		const user = userEvent.setup()

		await user.click(
			await screen.findByRole('button', {
				name: 'Refuser la demande de Alix Nova',
			}),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Refuser la demande' }),
		)
		expect(
			await within(dialog).findByText(
				'Un refus doit être motivé par un commentaire.',
			),
		).toBeDefined()
		expect(writes).toHaveLength(0)

		await user.type(within(dialog).getByLabelText('Commentaire'), 'Trop chargé')
		await user.click(
			within(dialog).getByRole('button', { name: 'Refuser la demande' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.params.body).toEqual({ comment: 'Trop chargé' })
		await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
		const history = screen.getByRole('region', { name: 'Historique' })
		expect(await within(history).findByText('Trop chargé')).toBeDefined()
	})

	it('spec_rh_3_9 keeps the reject dialog open on a refusal from the API', async () => {
		absences = [pendingRequest()]
		refuse = {
			status: 422,
			data: {
				message: 'comment: a rejection needs a comment',
				details: { field: 'comment' },
			},
		}
		await renderAbsences()
		const user = userEvent.setup()

		await user.click(
			await screen.findByRole('button', {
				name: 'Refuser la demande de Alix Nova',
			}),
		)
		const dialog = await screen.findByRole('dialog')
		await user.type(within(dialog).getByLabelText('Commentaire'), 'Non')
		await user.click(
			within(dialog).getByRole('button', { name: 'Refuser la demande' }),
		)

		expect(
			await within(dialog).findByText(
				'Un refus doit être motivé par un commentaire.',
			),
		).toBeDefined()
		expect(screen.getByRole('dialog')).toBeDefined()
	})

	it('spec_rh_3_9 cancels an upcoming request from the history menu', async () => {
		absences = [pendingRequest()]
		await renderAbsences()
		const user = userEvent.setup()

		const history = await screen.findByRole('region', { name: 'Historique' })
		await user.click(
			await within(history).findByRole('button', {
				name: /Actions pour la demande de Alix Nova/,
			}),
		)
		await user.click(
			await screen.findByRole('menuitem', { name: 'Annuler la demande' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.path).toContain('/cancel')
		expect(
			await screen.findByText('Demande de Alix Nova annulée.'),
		).toBeDefined()
		expect(await within(history).findByText('Annulée')).toBeDefined()
	})

	it('spec_rh_3_9 creates a type with the wire enums', async () => {
		await renderAbsences({ tab: 'motifs' })
		const user = userEvent.setup()

		await user.click(
			await screen.findByRole('button', { name: 'Nouveau motif' }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.type(within(dialog).getByLabelText('Nom'), 'Déménagement')
		await user.click(
			within(dialog).getByRole('button', { name: 'Créer le motif' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.method).toBe('post')
		expect(writes[0]?.params.body).toEqual({
			name: 'Déménagement',
			hue: 25,
			kind: 'LEAVE',
			count_mode: 'WORKING',
			account_id: null,
			paid: true,
			needs_approval: true,
			needs_proof: false,
		})
		await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
		expect(await screen.findByText('Déménagement')).toBeDefined()
	})

	it('spec_rh_3_9 edits a type through PUT and keeps its kind', async () => {
		await renderAbsences({ tab: 'motifs' })
		const user = userEvent.setup()

		await user.click(
			await screen.findByRole('button', {
				name: 'Modifier le motif Intempéries',
			}),
		)
		const dialog = await screen.findByRole('dialog')
		const name = within(dialog).getByLabelText('Nom')
		await user.clear(name)
		await user.type(name, 'Intempéries graves')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.method).toBe('put')
		expect(writes[0]?.params.path).toEqual({
			organization_id: 'org-1',
			absence_type_id: AUTO.id,
		})
		expect(writes[0]?.params.body).toMatchObject({
			name: 'Intempéries graves',
			kind: 'UNAVAILABLE',
			count_mode: 'CALENDAR',
			needs_approval: false,
		})
		expect(await screen.findByText('Intempéries graves')).toBeDefined()
	})

	it('spec_rh_3_9 archives then restores a type', async () => {
		await renderAbsences({ tab: 'motifs' })
		const user = userEvent.setup()

		await user.click(
			await screen.findByRole('button', {
				name: 'Archiver le motif Congés payés',
			}),
		)
		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.path).toBe(ARCHIVE_PATH)
		await user.click(await screen.findByText('Motifs archivés'))
		await user.click(
			await screen.findByRole('button', {
				name: 'Réactiver le motif Congés payés',
			}),
		)
		await waitFor(() => expect(writes).toHaveLength(2))
		expect(writes[1]?.path).toBe(RESTORE_PATH)
	})

	it('spec_rh_3_9 shows a name refusal under the field', async () => {
		refuse = {
			status: 422,
			data: { message: 'name: must not be empty', details: { field: 'name' } },
		}
		await renderAbsences({ tab: 'motifs' })
		const user = userEvent.setup()

		await user.click(
			await screen.findByRole('button', { name: 'Nouveau motif' }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.type(within(dialog).getByLabelText('Nom'), 'X')
		await user.click(
			within(dialog).getByRole('button', { name: 'Créer le motif' }),
		)

		expect(
			await within(dialog).findByText('Le nom ne peut pas être vide.'),
		).toBeDefined()
	})

	it('spec_rh_3_9 hides every write control without the permissions', async () => {
		permissions = []
		absences = [pendingRequest()]
		await renderAbsences()

		expect(
			await screen.findByRole('region', { name: 'Historique' }),
		).toBeDefined()
		expect(
			screen.queryByRole('button', { name: 'Poser une absence' }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: /Valider la demande/ }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: /Actions pour la demande/ }),
		).toBeNull()
	})

	it('spec_rh_3_9 hides the type controls without MANAGE_REFERENCE', async () => {
		permissions = ['MANAGE_PLANNING']
		await renderAbsences({ tab: 'motifs' })

		expect(await screen.findByText('Intempéries')).toBeDefined()
		expect(screen.queryByRole('button', { name: 'Nouveau motif' })).toBeNull()
		expect(
			screen.queryByRole('button', { name: /Modifier le motif/ }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: /Archiver le motif/ }),
		).toBeNull()
	})

	it('spec_rh_3_9 opens the screen empty when the lists answer 403', async () => {
		listsForbidden = true
		await renderAbsences()

		expect(
			await screen.findByRole('region', { name: 'Historique' }),
		).toBeDefined()
		expect(
			screen.getByText('Aucune demande d’absence pour le moment.'),
		).toBeDefined()
		expect(screen.queryByText(/Forbidden/)).toBeNull()
	})
})
