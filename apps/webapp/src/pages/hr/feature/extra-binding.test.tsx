import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it } from 'vitest'
import { TooltipProvider } from '#/components/ui/tooltip'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type {
	ExtraPeriodRecord,
	SurchargeRecord,
} from '#/pages/hr/lib/rh/extra-wire'
import type { ContractRecord } from '#/pages/hr/lib/rh/member-wire'
import { RhSidebarFeature } from '#/pages/rh-nav/feature/rh-sidebar-feature'
import { renderWithRouter } from '#/test/render-with-router'
import {
	ABSENCE_TYPES_PATH,
	ABSENCES_PATH,
	DEFAULT_TYPE_RECORDS,
} from '#/test/rh-absence-fixtures'
import {
	DEFAULT_SURCHARGE_RECORDS,
	EXTRAS_PATH,
	extraRecord,
	SURCHARGES_PATH,
	surchargeRecord,
} from '#/test/rh-extra-fixtures'
import {
	DEFAULT_ACCOUNT_RECORDS,
	LEAVE_ACCOUNTS_PATH,
	LEAVE_ADJUSTMENTS_PATH,
} from '#/test/rh-leave-fixtures'
import { ExtraFeature, type ExtraFeatureProps } from './extra-feature'

const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const PROFILES_PATH = '/api/v1/organizations/{organization_id}/member-profiles'
const CONTRACTS_PATH =
	'/api/v1/organizations/{organization_id}/member-contracts'
const DOCUMENTS_PATH =
	'/api/v1/organizations/{organization_id}/member-documents'
const PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'
const SURCHARGE_PATH = `${SURCHARGES_PATH}/{surcharge_id}`
const SURCHARGE_ARCHIVE_PATH = `${SURCHARGE_PATH}/archive`
const SURCHARGE_RESTORE_PATH = `${SURCHARGE_PATH}/restore`
const APPROVE_PATH = `${EXTRAS_PATH}/{period_id}/approve`
const REJECT_PATH = `${EXTRAS_PATH}/{period_id}/reject`
const CANCEL_PATH = `${EXTRAS_PATH}/{period_id}/cancel`

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

function contract(id: string, memberId: string): ContractRecord {
	return {
		id,
		organization_id: 'org-1',
		member_id: memberId,
		type: 'CDI',
		start_date: '2026-01-01',
		end_date: null,
		trial_end_date: null,
		schedule: { kind: 'weekly', week: [DAY, DAY, DAY, DAY, DAY, null, null] },
	}
}

const CONTRACTS = [contract('c-1', 'u-1'), contract('c-2', 'u-2')]

interface Write {
	method: string
	path: string
	params: { path?: Record<string, string>; body?: Record<string, unknown> }
}

let surcharges: SurchargeRecord[]
let extras: ExtraPeriodRecord[]
let writes: Write[]
let refuse: { status: number; data: Record<string, unknown> } | null
let extrasForbidden: boolean
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
						if (path === CONTRACTS_PATH)
							return { data: CONTRACTS, pagination: null }
						if (
							path === PROFILES_PATH ||
							path === DOCUMENTS_PATH ||
							path === ABSENCES_PATH ||
							path === LEAVE_ADJUSTMENTS_PATH
						)
							return { data: [], pagination: null }
						if (path === ABSENCE_TYPES_PATH)
							return { data: DEFAULT_TYPE_RECORDS, pagination: null }
						if (path === LEAVE_ACCOUNTS_PATH)
							return { data: DEFAULT_ACCOUNT_RECORDS, pagination: null }
						if (path === SURCHARGES_PATH)
							return { data: structuredClone(surcharges), pagination: null }
						if (path === EXTRAS_PATH) {
							if (extrasForbidden)
								throw Object.assign(new Error('Forbidden'), { status: 403 })
							return { data: structuredClone(extras), pagination: null }
						}
						if (path === PERMISSIONS_PATH)
							return { data: { permissions }, pagination: null }
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

function decide(
	periodId: string | undefined,
	patch: Partial<ExtraPeriodRecord>,
) {
	extras = extras.map((row) =>
		row.id === periodId ? { ...row, ...patch } : row,
	)
}

function applyWrite(method: string, path: string, params: Write['params']) {
	const body = (params.body ?? {}) as Record<string, unknown>
	const periodId = params.path?.period_id
	const surchargeId = params.path?.surcharge_id
	if (method === 'post' && path === EXTRAS_PATH) {
		const fromSurcharge = surcharges.find((row) => row.id === body.surcharge_id)
		extras = [
			extraRecord('e-new', String(body.member_id), {
				date: String(body.date),
				start: String(body.start),
				end: String(body.end),
				reason: String(body.reason),
				surcharge_id: fromSurcharge?.id ?? null,
				rate_bp: fromSurcharge?.rate_bp ?? Number(body.rate_bp ?? 0),
				status: body.approve ? 'APPROVED' : 'PENDING',
			}),
			...extras,
		]
	}
	if (path === APPROVE_PATH) decide(periodId, { status: 'APPROVED' })
	if (path === REJECT_PATH)
		decide(periodId, {
			status: 'REJECTED',
			decision_comment: String(body.comment),
		})
	if (path === CANCEL_PATH) decide(periodId, { status: 'CANCELLED' })
	if (method === 'post' && path === SURCHARGES_PATH)
		surcharges = [
			...surcharges,
			surchargeRecord('s-new', String(body.name), {
				rate_bp: Number(body.rate_bp),
				kind: body.kind as SurchargeRecord['kind'],
			}),
		]
	if (method === 'put' && path === SURCHARGE_PATH)
		surcharges = surcharges.map((row) =>
			row.id === surchargeId ? ({ ...row, ...body } as SurchargeRecord) : row,
		)
	if (path === SURCHARGE_ARCHIVE_PATH)
		surcharges = surcharges.map((row) =>
			row.id === surchargeId
				? { ...row, archived_at: '2026-11-08T10:00:00Z' }
				: row,
		)
	if (path === SURCHARGE_RESTORE_PATH)
		surcharges = surcharges.map((row) =>
			row.id === surchargeId ? { ...row, archived_at: null } : row,
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

async function renderExtra(patch: Partial<ExtraFeatureProps> = {}) {
	installFakeApi()
	return renderWithRouter(
		providers(
			<ExtraFeature
				tab="declarations"
				onTabChange={() => undefined}
				status="all"
				memberId="all"
				onFiltersChange={() => undefined}
				creating={false}
				onCreatingChange={() => undefined}
				{...patch}
			/>,
		),
	)
}

const PENDING = () =>
	extraRecord('e-1', 'u-1', {
		reason: 'Rattrapage API',
		surcharge_id: 's-dimanche',
		rate_bp: 10000,
	})

describe('surcharges and extra periods bound to the API', () => {
	beforeEach(() => {
		surcharges = structuredClone(DEFAULT_SURCHARGE_RECORDS)
		extras = []
		writes = []
		refuse = null
		extrasForbidden = false
		permissions = ['MANAGE_PLANNING', 'MANAGE_REFERENCE']
	})

	it('spec_rh_6_5 reads the pending list and the history from the API', async () => {
		extras = [
			PENDING(),
			extraRecord('e-2', 'u-2', {
				reason: 'Déjà validée API',
				status: 'APPROVED',
				decided_by_member_id: 'u-1',
				decided_at: '2026-11-04T08:00:00Z',
			}),
		]
		await renderExtra()

		expect((await screen.findAllByText('Rattrapage API')).length).toBe(1)
		expect(screen.getAllByText('Bao Tran').length).toBeGreaterThan(0)
		expect(screen.queryByText(/Faure/)).toBeNull()
	})

	it('spec_rh_6_5 approves a pending period through the API', async () => {
		extras = [PENDING()]
		await renderExtra()
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', {
				name: 'Valider la déclaration de Alix Nova',
			}),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]).toMatchObject({
			method: 'post',
			path: APPROVE_PATH,
			params: { path: { organization_id: 'org-1', period_id: 'e-1' } },
		})
		await waitFor(() =>
			expect(
				screen.queryByRole('button', {
					name: 'Valider la déclaration de Alix Nova',
				}),
			).toBeNull(),
		)
	})

	it('spec_rh_6_5 says the period was already decided when the API refuses an approval', async () => {
		extras = [PENDING()]
		refuse = {
			status: 422,
			data: {
				message: 'status: the extra period was already decided',
				details: { field: 'status' },
			},
		}
		await renderExtra()
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', {
				name: 'Valider la déclaration de Alix Nova',
			}),
		)

		expect(
			await screen.findByText('Cette déclaration a déjà été traitée.'),
		).toBeDefined()
	})

	it('spec_rh_6_5 rejects with a comment through the API', async () => {
		extras = [PENDING()]
		await renderExtra()
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', {
				name: 'Refuser la déclaration de Alix Nova',
			}),
		)
		const dialog = await screen.findByRole('dialog')
		await user.type(
			within(dialog).getByLabelText('Commentaire'),
			'Pas de commande',
		)
		await user.click(
			within(dialog).getByRole('button', { name: 'Refuser la déclaration' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]).toMatchObject({
			method: 'post',
			path: REJECT_PATH,
			params: {
				path: { organization_id: 'org-1', period_id: 'e-1' },
				body: { comment: 'Pas de commande' },
			},
		})
		await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
	})

	it('spec_rh_6_5 refuses a blank rejection comment before calling the API', async () => {
		extras = [PENDING()]
		await renderExtra()
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', {
				name: 'Refuser la déclaration de Alix Nova',
			}),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Refuser la déclaration' }),
		)

		expect(
			await within(dialog).findByText(
				'Un refus doit être motivé par un commentaire.',
			),
		).toBeDefined()
		expect(writes).toHaveLength(0)
	})

	it('spec_rh_6_5 shows a refused rejection inside its dialog', async () => {
		extras = [PENDING()]
		refuse = {
			status: 422,
			data: {
				message: 'status: the extra period was already decided',
				details: { field: 'status' },
			},
		}
		await renderExtra()
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', {
				name: 'Refuser la déclaration de Alix Nova',
			}),
		)
		const dialog = await screen.findByRole('dialog')
		await user.type(within(dialog).getByLabelText('Commentaire'), 'Non')
		await user.click(
			within(dialog).getByRole('button', { name: 'Refuser la déclaration' }),
		)

		expect(
			await within(dialog).findByText('Cette déclaration a déjà été traitée.'),
		).toBeDefined()
		expect(screen.getByRole('dialog')).toBeDefined()
	})

	it('spec_rh_6_5 cancels an approved upcoming period through the API', async () => {
		extras = [
			extraRecord('e-1', 'u-1', {
				date: '2999-01-02',
				status: 'APPROVED',
				reason: 'À annuler API',
			}),
		]
		await renderExtra()
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', {
				name: /Actions pour la déclaration/,
			}),
		)
		await user.click(
			await screen.findByRole('menuitem', { name: 'Annuler la déclaration' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]).toMatchObject({
			method: 'post',
			path: CANCEL_PATH,
			params: { path: { organization_id: 'org-1', period_id: 'e-1' } },
		})
	})

	it('spec_rh_6_5 creates a period that copies the rate of the suggested surcharge', async () => {
		await renderExtra({
			creating: true,
			initial: { memberId: 'u-1', date: '2026-11-14' },
		})
		const dialog = await screen.findByRole('dialog')
		const user = userEvent.setup()
		fireEvent.change(within(dialog).getByLabelText('Début'), {
			target: { value: '08:00' },
		})
		fireEvent.change(within(dialog).getByLabelText('Fin'), {
			target: { value: '12:00' },
		})
		await user.type(within(dialog).getByLabelText('Raison'), 'Rattrapage')
		await user.click(within(dialog).getByRole('button', { name: 'Déclarer' }))

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]?.path).toBe(EXTRAS_PATH)
		expect(writes[0]?.params.body).toMatchObject({
			member_id: 'u-1',
			date: '2026-11-14',
			start: '08:00',
			end: '12:00',
			reason: 'Rattrapage',
			surcharge_id: 's-hs25',
			approve: false,
		})
		expect(writes[0]?.params.body?.rate_bp).toBeUndefined()
		expect(extras[0]?.rate_bp).toBe(2500)
	})

	it('spec_rh_6_5 shows a refused period inside the form and keeps it open', async () => {
		refuse = {
			status: 422,
			data: {
				message: 'surcharge_id: an archived surcharge cannot be chosen',
				details: { field: 'surcharge_id' },
			},
		}
		await renderExtra({
			creating: true,
			initial: { memberId: 'u-1', date: '2026-11-14' },
		})
		const dialog = await screen.findByRole('dialog')
		const user = userEvent.setup()
		fireEvent.change(within(dialog).getByLabelText('Début'), {
			target: { value: '08:00' },
		})
		fireEvent.change(within(dialog).getByLabelText('Fin'), {
			target: { value: '12:00' },
		})
		await user.type(within(dialog).getByLabelText('Raison'), 'Rattrapage')
		await user.click(within(dialog).getByRole('button', { name: 'Déclarer' }))

		expect(
			await within(dialog).findByText(
				'Cette majoration est archivée ou introuvable.',
			),
		).toBeDefined()
		expect(screen.getByRole('dialog')).toBeDefined()
	})

	it('spec_rh_6_5 creates a surcharge through the API in basis points', async () => {
		await renderExtra({ tab: 'majorations' })
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', { name: 'Nouvelle majoration' }),
		)
		const dialog = await screen.findByRole('dialog')
		await user.type(within(dialog).getByLabelText('Nom'), 'Astreinte')
		await user.type(within(dialog).getByLabelText('Taux'), '12.5')
		await user.click(
			within(dialog).getByRole('button', { name: 'Créer la majoration' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]).toMatchObject({
			method: 'post',
			path: SURCHARGES_PATH,
			params: {
				path: { organization_id: 'org-1' },
				body: { name: 'Astreinte', rate_bp: 1250, kind: 'OVERTIME' },
			},
		})
		expect(await screen.findByText('Astreinte')).toBeDefined()
	})

	it('spec_rh_6_5 edits a surcharge through the API', async () => {
		await renderExtra({ tab: 'majorations' })
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', {
				name: 'Modifier la majoration Heures supplémentaires',
			}),
		)
		const dialog = await screen.findByRole('dialog')
		const rate = within(dialog).getByLabelText('Taux')
		await user.clear(rate)
		await user.type(rate, '30')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]).toMatchObject({
			method: 'put',
			path: SURCHARGE_PATH,
			params: {
				path: { organization_id: 'org-1', surcharge_id: 's-hs25' },
				body: {
					name: 'Heures supplémentaires',
					rate_bp: 3000,
					kind: 'OVERTIME',
				},
			},
		})
	})

	it('spec_rh_6_5 shows a refused surcharge inside its dialog', async () => {
		refuse = {
			status: 422,
			data: { message: 'rate_bp', details: { field: 'rate_bp' } },
		}
		await renderExtra({ tab: 'majorations' })
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', {
				name: 'Modifier la majoration Heures supplémentaires',
			}),
		)
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		expect(
			await within(dialog).findByText(/Le taux doit être compris/),
		).toBeDefined()
		expect(screen.getByRole('dialog')).toBeDefined()
	})

	it('spec_rh_6_5 archives and restores a surcharge through the API', async () => {
		await renderExtra({ tab: 'majorations' })
		const user = userEvent.setup()
		await user.click(
			await screen.findByRole('button', {
				name: 'Archiver la majoration Travail du dimanche',
			}),
		)

		await waitFor(() => expect(writes).toHaveLength(1))
		expect(writes[0]).toMatchObject({
			method: 'post',
			path: SURCHARGE_ARCHIVE_PATH,
			params: {
				path: { organization_id: 'org-1', surcharge_id: 's-dimanche' },
			},
		})
		await user.click(await screen.findByText('Majorations archivées'))
		await user.click(
			await screen.findByRole('button', {
				name: 'Réactiver la majoration Travail du dimanche',
			}),
		)

		await waitFor(() => expect(writes).toHaveLength(2))
		expect(writes[1]).toMatchObject({
			method: 'post',
			path: SURCHARGE_RESTORE_PATH,
		})
	})

	it('spec_rh_6_5 falls back to an empty read-only screen on a 403', async () => {
		extras = [PENDING()]
		extrasForbidden = true
		permissions = []
		await renderExtra()

		expect(await screen.findByText('Heures extra')).toBeDefined()
		expect(screen.queryByRole('alert')).toBeNull()
		expect(screen.queryByText('Rattrapage API')).toBeNull()
		expect(
			screen.queryByRole('button', { name: 'Déclarer des heures' }),
		).toBeNull()
		expect(writes).toHaveLength(0)
	})

	it('spec_rh_6_5 counts the pending periods of the API in the sidebar', async () => {
		extras = [
			PENDING(),
			extraRecord('e-2', 'u-2', { status: 'APPROVED' }),
			extraRecord('e-3', 'u-2', { status: 'CANCELLED' }),
		]
		installFakeApi()
		await renderWithRouter(providers(<RhSidebarFeature />))

		const section = await screen.findByRole('region', { name: 'Heures extra' })
		const pending = within(section).getByRole('link', { name: /À valider/ })
		await waitFor(() => expect(pending.textContent).toMatch(/1/))
		const all = within(section).getByRole('link', {
			name: /Toutes les déclarations/,
		})
		expect(all.textContent).toMatch(/3/)
		const rates = within(section).getByRole('link', { name: /Majorations/ })
		expect(rates.textContent).toMatch(/3/)
	})
})
