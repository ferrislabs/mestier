import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { TooltipProvider } from '#/components/ui/tooltip'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { resetRhStore } from '#/pages/hr/feature/mocks/rh-store'
import type {
	ContractRecord,
	DocumentRecord,
} from '#/pages/hr/lib/rh/member-wire'
import { renderWithRouter } from '#/test/render-with-router'
import {
	ABSENCE_TYPES_PATH,
	ABSENCES_PATH,
	DEFAULT_TYPE_RECORDS,
} from '#/test/rh-absence-fixtures'
import {
	DEFAULT_ACCOUNT_RECORDS,
	LEAVE_ACCOUNTS_PATH,
	LEAVE_ADJUSTMENTS_PATH,
} from '#/test/rh-leave-fixtures'
import { MemberDetailFeature } from './member-detail-feature'

const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const PROFILES_PATH = '/api/v1/organizations/{organization_id}/member-profiles'
const CONTRACTS_PATH =
	'/api/v1/organizations/{organization_id}/member-contracts'
const DOCUMENTS_PATH =
	'/api/v1/organizations/{organization_id}/member-documents'
const FILE_URL_PATH = '/api/v1/files/url'
const PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

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

const DAY = {
	start: '08:00',
	end: '17:00',
	lunch: { start: '12:00', end: '13:00' },
}

function contract(
	id: string,
	memberId: string,
	patch: Partial<ContractRecord> = {},
): ContractRecord {
	return {
		id,
		organization_id: 'org-1',
		member_id: memberId,
		type: 'CDI',
		start_date: '2026-01-01',
		end_date: null,
		trial_end_date: null,
		schedule: { kind: 'weekly', week: [DAY, DAY, DAY, DAY, DAY, null, null] },
		...patch,
	}
}

function document(id: string, memberId: string, name: string): DocumentRecord {
	return {
		id,
		organization_id: 'org-1',
		member_id: memberId,
		name,
		kind: 'IDENTITY',
		file_key: `uploads/${id}`,
		added_at: '2026-03-04T10:00:00Z',
	}
}

interface Write {
	method: string
	path: string
	params: { path?: Record<string, string>; body?: Record<string, unknown> }
}

let contracts: ContractRecord[]
let documents: DocumentRecord[]
let writes: Write[]
let uploads: File[]
let refuse: { status: number; data: Record<string, unknown> } | null
let listsForbidden: boolean
let permissions: string[]

function forbidden() {
	return Object.assign(new Error('Forbidden'), { status: 403 })
}

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
						if (path === ABSENCE_TYPES_PATH)
							return { data: DEFAULT_TYPE_RECORDS, pagination: null }
						if (path === ABSENCES_PATH || path === PROFILES_PATH)
							return { data: [], pagination: null }
						if (path === CONTRACTS_PATH || path === DOCUMENTS_PATH) {
							if (listsForbidden) throw forbidden()
							return {
								data: structuredClone(
									path === CONTRACTS_PATH ? contracts : documents,
								),
								pagination: null,
							}
						}
						if (path === FILE_URL_PATH) {
							const key = (p.query as { key: string }).key
							return { data: { url: `https://files.test/${key}` } }
						}
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
						const memberId = params.path?.member_id ?? ''
						if (path.endsWith('/contracts'))
							contracts = [
								...contracts,
								{
									...(params.body as unknown as ContractRecord),
									id: 'c-new',
									organization_id: 'org-1',
									member_id: memberId,
								},
							]
						if (path.endsWith('/contracts/{contract_id}')) {
							const id = params.path?.contract_id
							contracts =
								method === 'delete'
									? contracts.filter((row) => row.id !== id)
									: contracts.map((row) =>
											row.id === id
												? { ...row, ...(params.body as object) }
												: row,
										)
						}
						if (path.endsWith('/documents'))
							documents = [
								...documents,
								{
									...(params.body as unknown as DocumentRecord),
									id: 'd-new',
									organization_id: 'org-1',
									member_id: memberId,
									added_at: '2026-11-08T10:00:00Z',
								},
							]
						if (path.endsWith('/documents/{document_id}'))
							documents = documents.filter(
								(row) => row.id !== params.path?.document_id,
							)
						return { data: {} }
					},
				},
			}
		},
	}
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake of the file upload client
	;(window as any).api = {
		post: async (path: string, init: { body: ArrayBuffer }) => {
			if (path !== '/api/v1/files') throw new Error(`unmocked POST ${path}`)
			uploads.push(new File([init.body], 'uploaded'))
			return { data: { key: 'uploads/new-file' } }
		},
	}
}

function renderDetail(memberId: string, tab: string) {
	installFakeApi()
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	return renderWithRouter(
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION as never]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION as never}>
					<TooltipProvider>
						<MemberDetailFeature
							memberId={memberId}
							tab={tab as never}
							month={null}
							onChange={() => undefined}
						/>
					</TooltipProvider>
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>,
	)
}

describe('member contracts and documents bound to the API', () => {
	beforeEach(() => {
		resetRhStore()
		contracts = []
		documents = []
		writes = []
		uploads = []
		refuse = null
		listsForbidden = false
		permissions = ['MANAGE_MEMBERS']
	})

	afterEach(() => {
		resetRhStore()
	})

	it('spec_rh_2_6 shows the contracts read from the API and none for a member without one', async () => {
		contracts = [
			contract('c-1', 'u-2', {
				type: 'CDD',
				start_date: '2026-02-01',
				end_date: '2027-01-31',
			}),
		]
		await renderDetail('u-2', 'contrats')

		expect(
			await screen.findByRole('heading', { name: 'Bao Tran' }),
		).toBeDefined()
		expect(
			await screen.findByRole('button', { name: /Modifier le contrat CDD/ }),
		).toBeDefined()
	})

	it('spec_rh_2_6 never shows a prototype contract for a member without one', async () => {
		await renderDetail('u-1', 'contrats')

		expect(
			await screen.findByRole('heading', { name: 'Alix Nova' }),
		).toBeDefined()
		expect(screen.getAllByText('Aucun contrat').length).toBeGreaterThan(0)
		expect(
			screen.queryByRole('button', { name: /Modifier le contrat/ }),
		).toBeNull()
	})

	it('spec_rh_2_6 writes a new contract through the API with the wire enums and schedule', async () => {
		const user = userEvent.setup()
		await renderDetail('u-1', 'contrats')
		await screen.findByRole('heading', { name: 'Alix Nova' })

		await user.click(
			await screen.findByRole('button', { name: /Ajouter un contrat/ }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Ajouter le contrat' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.method).toBe('post')
		expect(writes[0]?.params.path).toEqual({
			organization_id: 'org-1',
			member_id: 'u-1',
		})
		expect(writes[0]?.params.body).toMatchObject({
			type: 'CDI',
			end_date: null,
			trial_end_date: null,
			schedule: { kind: 'weekly' },
		})
		await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
		expect(
			await screen.findByRole('button', { name: /Modifier le contrat CDI/ }),
		).toBeDefined()
	})

	it('spec_rh_2_6 shows an overlap refusal next to the dates and keeps the dialog open', async () => {
		const user = userEvent.setup()
		refuse = {
			status: 422,
			data: {
				code: 'E_MEMBER_CONTRACT_OVERLAP',
				message: 'overlap',
				details: { field: 'start_date' },
			},
		}
		await renderDetail('u-1', 'contrats')
		await screen.findByRole('heading', { name: 'Alix Nova' })

		await user.click(
			await screen.findByRole('button', { name: /Ajouter un contrat/ }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Ajouter le contrat' }),
		)

		expect(
			await within(dialog).findByText('Ce contrat chevauche un autre contrat.'),
		).toBeDefined()
		expect(screen.queryByRole('dialog')).not.toBeNull()
	})

	it('spec_rh_2_6 names the day of a refused schedule in the dialog error', async () => {
		const user = userEvent.setup()
		refuse = {
			status: 422,
			data: {
				code: 'E_UNPROCESSABLE_ENTITY',
				message: 'the lunch break must sit inside the day',
				details: { field: 'schedule.week[2]' },
			},
		}
		await renderDetail('u-1', 'contrats')
		await screen.findByRole('heading', { name: 'Alix Nova' })

		await user.click(
			await screen.findByRole('button', { name: /Ajouter un contrat/ }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Ajouter le contrat' }),
		)

		expect(
			await within(dialog).findByText(
				'Rythme, mercredi : the lunch break must sit inside the day',
			),
		).toBeDefined()
	})

	it('spec_rh_2_6 deletes a contract through the API', async () => {
		const user = userEvent.setup()
		contracts = [
			contract('c-1', 'u-2', {
				type: 'CDD',
				start_date: '2026-01-01',
				end_date: '2026-06-30',
			}),
			contract('c-2', 'u-2', { start_date: '2026-07-01' }),
		]
		await renderDetail('u-2', 'contrats')
		await screen.findByRole('heading', { name: 'Bao Tran' })

		await user.click(
			await screen.findByRole('button', { name: /Supprimer le contrat CDD/ }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(within(dialog).getByRole('button', { name: 'Supprimer' }))

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.method).toBe('delete')
		expect(writes[0]?.params.path).toEqual({
			organization_id: 'org-1',
			member_id: 'u-2',
			contract_id: 'c-1',
		})
		await waitFor(() =>
			expect(
				screen.queryByRole('button', { name: /Supprimer le contrat CDD/ }),
			).toBeNull(),
		)
	})

	it('spec_rh_2_6 saves the rhythm as a full contract update', async () => {
		const user = userEvent.setup()
		contracts = [contract('c-1', 'u-2', { trial_end_date: '2026-03-01' })]
		await renderDetail('u-2', 'emploi-du-temps')
		await screen.findByRole('heading', { name: 'Bao Tran' })

		await user.click(
			await screen.findByRole('button', { name: /Modifier le rythme/ }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.method).toBe('put')
		expect(writes[0]?.params.path).toEqual({
			organization_id: 'org-1',
			member_id: 'u-2',
			contract_id: 'c-1',
		})
		expect(writes[0]?.params.body).toMatchObject({
			type: 'CDI',
			start_date: '2026-01-01',
			trial_end_date: '2026-03-01',
			schedule: { kind: 'weekly' },
		})
	})

	it('spec_rh_2_6 falls back to no contract and no write control when the lists are forbidden', async () => {
		listsForbidden = true
		permissions = ['VIEW_PLANNING']
		await renderDetail('u-2', 'contrats')

		expect(
			await screen.findByRole('heading', { name: 'Bao Tran' }),
		).toBeDefined()
		expect(screen.getAllByText('Aucun contrat').length).toBeGreaterThan(0)
		expect(
			screen.queryByRole('button', { name: /Ajouter un contrat/ }),
		).toBeNull()
	})

	it('spec_rh_2_6 hides the contract writes without MANAGE_MEMBERS', async () => {
		contracts = [contract('c-1', 'u-2')]
		permissions = ['VIEW_PLANNING']
		await renderDetail('u-2', 'contrats')

		expect(
			await screen.findByRole('heading', { name: 'Bao Tran' }),
		).toBeDefined()
		expect(
			screen.queryByRole('button', { name: /Ajouter un contrat/ }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: /Modifier le contrat/ }),
		).toBeNull()
		expect(
			screen.queryByRole('button', { name: /Supprimer le contrat/ }),
		).toBeNull()
	})

	it('spec_rh_2_6 lists the documents read from the API with a link to the file', async () => {
		documents = [document('d-1', 'u-2', 'CNI.pdf')]
		await renderDetail('u-2', 'documents')

		const link = await screen.findByRole('link', { name: 'CNI.pdf' })
		await waitFor(() =>
			expect(link.getAttribute('href')).toBe('https://files.test/uploads/d-1'),
		)
	})

	it('spec_rh_2_6 uploads the picked file then registers the document with the returned key', async () => {
		const user = userEvent.setup()
		await renderDetail('u-2', 'documents')
		await screen.findByRole('heading', { name: 'Bao Tran' })

		const input = await screen.findByLabelText('Choisir un fichier')
		await user.upload(
			input,
			new File(['pdf'], 'Diplome.pdf', { type: 'application/pdf' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(uploads).toHaveLength(1)
		expect(writes[0]?.method).toBe('post')
		expect(writes[0]?.params.path).toEqual({
			organization_id: 'org-1',
			member_id: 'u-2',
		})
		expect(writes[0]?.params.body).toEqual({
			name: 'Diplome.pdf',
			kind: 'OTHER',
			file_key: 'uploads/new-file',
		})
		expect(await screen.findByText('Diplome.pdf')).toBeDefined()
	})

	it('spec_rh_2_6 deletes a document through the API', async () => {
		const user = userEvent.setup()
		documents = [document('d-1', 'u-2', 'CNI.pdf')]
		await renderDetail('u-2', 'documents')

		await user.click(
			await screen.findByRole('button', { name: 'Supprimer CNI.pdf' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.method).toBe('delete')
		expect(writes[0]?.params.path).toEqual({
			organization_id: 'org-1',
			member_id: 'u-2',
			document_id: 'd-1',
		})
		await waitFor(() => expect(screen.queryByText('CNI.pdf')).toBeNull())
	})

	it('spec_rh_2_6 hides the document writes when the lists are forbidden', async () => {
		listsForbidden = true
		permissions = ['VIEW_PLANNING']
		await renderDetail('u-2', 'documents')

		expect(
			await screen.findByRole('heading', { name: 'Bao Tran' }),
		).toBeDefined()
		expect(screen.queryByLabelText('Choisir un fichier')).toBeNull()
		expect(screen.getByText('Aucun document pour l’instant.')).toBeDefined()
	})
})
