import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it } from 'vitest'
import { TooltipProvider } from '#/components/ui/tooltip'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type {
	LeaveAccountRecord,
	LeaveAdjustmentRecord,
} from '#/pages/hr/lib/rh/leave-wire'
import { RhSidebarFeature } from '#/pages/rh-nav/feature/rh-sidebar-feature'
import { renderWithRouter } from '#/test/render-with-router'
import {
	ABSENCE_TYPES_PATH,
	ABSENCES_PATH,
	DEFAULT_TYPE_RECORDS,
} from '#/test/rh-absence-fixtures'
import { MY_MEMBER_PATH, myMemberResponse } from '#/test/rh-current-member'
import { EXTRAS_PATH, SURCHARGES_PATH } from '#/test/rh-extra-fixtures'
import {
	accountRecord,
	adjustmentRecord,
	DEFAULT_ACCOUNT_RECORDS,
	LEAVE_ACCOUNTS_PATH,
	LEAVE_ADJUSTMENTS_PATH,
} from '#/test/rh-leave-fixtures'
import { RightsFeature } from './rights-feature'

const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const PROFILES_PATH = '/api/v1/organizations/{organization_id}/member-profiles'
const CONTRACTS_PATH =
	'/api/v1/organizations/{organization_id}/member-contracts'
const DOCUMENTS_PATH =
	'/api/v1/organizations/{organization_id}/member-documents'
const PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'
const ACCOUNT_PATH = `${LEAVE_ACCOUNTS_PATH}/{account_id}`
const ARCHIVE_PATH = `${ACCOUNT_PATH}/archive`
const RESTORE_PATH = `${ACCOUNT_PATH}/restore`
const ADJUSTMENT_PATH = `${LEAVE_ADJUSTMENTS_PATH}/{adjustment_id}`

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

interface Write {
	method: string
	path: string
	params: { path?: Record<string, string>; body?: Record<string, unknown> }
}

let accounts: LeaveAccountRecord[]
let adjustments: LeaveAdjustmentRecord[]
let writes: Write[]
let refuse: { status: number; data: Record<string, unknown> } | null
let adjustmentsForbidden: boolean
let permissions: string[]
let accountReads: number
let adjustmentReads: number

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
							path === DOCUMENTS_PATH ||
							path === ABSENCES_PATH
						)
							return { data: [], pagination: null }
						if (path === ABSENCE_TYPES_PATH)
							return { data: DEFAULT_TYPE_RECORDS, pagination: null }
						if (path === LEAVE_ACCOUNTS_PATH) {
							accountReads += 1
							return { data: structuredClone(accounts), pagination: null }
						}
						if (path === LEAVE_ADJUSTMENTS_PATH) {
							adjustmentReads += 1
							if (adjustmentsForbidden)
								throw Object.assign(new Error('Forbidden'), { status: 403 })
							return { data: structuredClone(adjustments), pagination: null }
						}
						if (path === MY_MEMBER_PATH) return myMemberResponse()
						if (path === PERMISSIONS_PATH)
							return { data: { permissions }, pagination: null }
						if (path === SURCHARGES_PATH || path === EXTRAS_PATH)
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
}

function applyWrite(method: string, path: string, params: Write['params']) {
	const body = (params.body ?? {}) as Record<string, unknown>
	const accountId = params.path?.account_id
	if (method === 'post' && path === LEAVE_ADJUSTMENTS_PATH)
		adjustments = [
			...adjustments,
			adjustmentRecord(
				'adj-new',
				String(body.member_id),
				String(body.account_id),
				{
					date: String(body.date),
					days: Number(body.days),
					kind: body.kind as LeaveAdjustmentRecord['kind'],
					reason: String(body.reason),
				},
			),
		]
	if (method === 'delete' && path === ADJUSTMENT_PATH)
		adjustments = adjustments.filter(
			(row) => row.id !== params.path?.adjustment_id,
		)
	if (method === 'put' && path === ACCOUNT_PATH)
		accounts = accounts.map((row) =>
			row.id === accountId ? ({ ...row, ...body } as LeaveAccountRecord) : row,
		)
	if (path === ARCHIVE_PATH)
		accounts = accounts.map((row) =>
			row.id === accountId
				? { ...row, archived_at: '2026-11-08T10:00:00Z' }
				: row,
		)
	if (path === RESTORE_PATH)
		accounts = accounts.map((row) =>
			row.id === accountId ? { ...row, archived_at: null } : row,
		)
	return { data: {} }
}

function providers(children: React.ReactNode) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	return (
		<QueryClientProvider client={queryClient}>
			<OrganizationListProvider organizations={[ORGANIZATION as never]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION as never}>
					<TooltipProvider>{children}</TooltipProvider>
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>
	)
}

async function renderRights(accountId: string | null = null) {
	installFakeApi()
	return renderWithRouter(
		providers(
			<RightsFeature accountId={accountId} onAccountChange={() => undefined} />,
		),
	)
}

async function openAlix() {
	const user = userEvent.setup()
	await user.click(
		await screen.findByRole('button', { name: 'Détail de Alix Nova' }),
	)
	return { user, dialog: await screen.findByRole('dialog') }
}

describe('leave accounts and adjustments bound to the API', () => {
	beforeEach(() => {
		accounts = structuredClone(DEFAULT_ACCOUNT_RECORDS)
		adjustments = []
		writes = []
		refuse = null
		adjustmentsForbidden = false
		permissions = ['MANAGE_REFERENCE', 'MANAGE_PLANNING']
		accountReads = 0
		adjustmentReads = 0
	})

	it('spec_rh_4_5 reads the accounts and the adjustments from the API', async () => {
		adjustments = [
			adjustmentRecord('adj-1', 'u-1', 'acc-cp', {
				days: 2,
				kind: 'CARRY_OVER',
				date: new Date().toISOString().slice(0, 10),
				reason: 'Report API',
			}),
		]
		await renderRights()

		expect(
			await screen.findByRole('tab', { name: 'Congés payés' }),
		).toBeDefined()
		expect(screen.getByRole('tab', { name: 'RTT' })).toBeDefined()
		expect(screen.queryByText('Karim Haddad')).toBeNull()
		expect(accountReads).toBeGreaterThan(0)
		expect(adjustmentReads).toBeGreaterThan(0)
		const { dialog } = await openAlix()
		expect(await within(dialog).findByText('Report API')).toBeDefined()
	})

	it('spec_rh_4_5 resolves the account asked for by its real id', async () => {
		await renderRights('acc-rtt')

		expect(
			await screen.findByRole('region', { name: 'Règle du compte RTT' }),
		).toBeDefined()
	})

	it('spec_rh_4_5 creates an adjustment through the API and shows it after the read', async () => {
		await renderRights()
		const { user, dialog } = await openAlix()
		await user.click(
			within(dialog).getByRole('button', {
				name: 'Ajouter une régularisation',
			}),
		)
		await user.type(within(dialog).getByLabelText('Jours'), '-1')
		await user.type(within(dialog).getByLabelText(/Motif/), 'Jour récupéré')
		await user.click(
			within(dialog).getByRole('button', {
				name: 'Enregistrer la régularisation',
			}),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.path).toBe(LEAVE_ADJUSTMENTS_PATH)
		expect(writes[0]?.params.body).toMatchObject({
			member_id: 'u-1',
			account_id: 'acc-cp',
			days: -1,
			kind: 'ADJUSTMENT',
			reason: 'Jour récupéré',
		})
		expect(await within(dialog).findByText('Jour récupéré')).toBeDefined()
		expect(
			within(dialog).queryByRole('form', {
				name: 'Ajouter une régularisation',
			}),
		).toBeNull()
	})

	it('spec_rh_4_5 says why an adjustment is refused and keeps the form open', async () => {
		refuse = {
			status: 422,
			data: {
				message: 'account_id: the leave account is archived',
				details: { field: 'account_id' },
			},
		}
		await renderRights()
		const { user, dialog } = await openAlix()
		await user.click(
			within(dialog).getByRole('button', {
				name: 'Ajouter une régularisation',
			}),
		)
		await user.type(within(dialog).getByLabelText('Jours'), '2')
		await user.type(within(dialog).getByLabelText(/Motif/), 'Geste')
		await user.click(
			within(dialog).getByRole('button', {
				name: 'Enregistrer la régularisation',
			}),
		)

		expect(
			await within(dialog).findByText('Ce compte est archivé ou introuvable.'),
		).toBeDefined()
		expect(
			within(dialog).getByRole('form', { name: 'Ajouter une régularisation' }),
		).toBeDefined()
	})

	it('spec_rh_4_5 removes an adjustment through the API', async () => {
		adjustments = [
			adjustmentRecord('adj-1', 'u-1', 'acc-cp', {
				date: new Date().toISOString().slice(0, 10),
				reason: 'À retirer',
			}),
		]
		await renderRights()
		const { user, dialog } = await openAlix()
		expect(await within(dialog).findByText('À retirer')).toBeDefined()
		const remove = within(dialog).getByRole('button', {
			name: /Supprimer la régularisation/,
		})
		await user.click(remove)
		await user.click(
			within(dialog).getByRole('button', { name: 'Confirmer la suppression' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]).toMatchObject({
			method: 'delete',
			path: ADJUSTMENT_PATH,
			params: {
				path: { organization_id: 'org-1', adjustment_id: 'adj-1' },
			},
		})
		await waitFor(() =>
			expect(within(dialog).queryByText('À retirer')).toBeNull(),
		)
	})

	it('spec_rh_4_5 edits an account rule through the API', async () => {
		await renderRights()
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', { name: 'Paramétrer le compte' }),
		)
		const dialog = await screen.findByRole('dialog')
		const days = within(dialog).getByLabelText('Jours par mois')
		await user.clear(days)
		await user.type(days, '2.5')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]).toMatchObject({
			method: 'put',
			path: ACCOUNT_PATH,
			params: {
				path: { organization_id: 'org-1', account_id: 'acc-cp' },
				body: {
					name: 'Congés payés',
					acquisition: { kind: 'MONTHLY', per_month: 2.5 },
					period_start_month: 6,
					carry_over_max: 5,
				},
			},
		})
		await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
	})

	it('spec_rh_4_5 shows a refused account rule inside the dialog', async () => {
		refuse = {
			status: 422,
			data: { message: 'acquisition', details: { field: 'acquisition' } },
		}
		await renderRights()
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', { name: 'Paramétrer le compte' }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		expect(
			await within(dialog).findByText(/L’acquisition doit être positive/),
		).toBeDefined()
		expect(screen.getByRole('dialog')).toBeDefined()
	})

	it('spec_rh_4_5 archives an account through the API and labels its tab', async () => {
		await renderRights()
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', { name: 'Paramétrer le compte' }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Archiver le compte' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]).toMatchObject({
			method: 'post',
			path: ARCHIVE_PATH,
			params: { path: { organization_id: 'org-1', account_id: 'acc-cp' } },
		})
		expect(
			await screen.findByRole('tab', { name: 'Congés payés (archivé)' }),
		).toBeDefined()
	})

	it('spec_rh_4_5 falls back to no movement and read-only controls on a 403', async () => {
		adjustments = [
			adjustmentRecord('adj-1', 'u-1', 'acc-cp', {
				date: new Date().toISOString().slice(0, 10),
				reason: 'Jamais lu',
			}),
		]
		adjustmentsForbidden = true
		permissions = []
		await renderRights()

		expect(
			await screen.findByRole('tab', { name: 'Congés payés' }),
		).toBeDefined()
		expect(screen.queryByRole('alert')).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Paramétrer le compte' }),
		).toBeNull()
		const { dialog } = await openAlix()
		expect(within(dialog).queryByText('Jamais lu')).toBeNull()
		expect(
			within(dialog).queryByRole('button', {
				name: 'Ajouter une régularisation',
			}),
		).toBeNull()
		expect(writes).toHaveLength(0)
	})

	it('spec_rh_4_5 lists the API accounts in the sidebar, archived ones left out', async () => {
		accounts = [
			...DEFAULT_ACCOUNT_RECORDS,
			accountRecord('acc-old', 'Ancienneté', {
				archived_at: '2026-01-01T00:00:00Z',
			}),
		]
		installFakeApi()
		await renderWithRouter(providers(<RhSidebarFeature />))

		const links = await screen.findAllByRole('link', {
			name: /Congés payés|RTT/,
		})
		expect(links.length).toBeGreaterThanOrEqual(2)
		expect(screen.queryByText('Ancienneté')).toBeNull()
	})
})
