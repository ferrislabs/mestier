import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { PERMISSION_CATALOG } from '#/lib/permission-catalog'
import { ContractsFeature } from '#/pages/maintenance/feature/contracts-feature'
import type { MaintenanceContract } from '#/pages/maintenance/types'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

const MAINTENANCE = '/api/v1/organizations/{organization_id}/maintenance'
const CONTRACTS_PATH = `${MAINTENANCE}/contracts`
const CONTRACT_PATH = `${MAINTENANCE}/contracts/{contract_id}`
const DUPLICATE_PATH = `${MAINTENANCE}/contracts/{contract_id}/duplicate`
const GRID_PATH = `${MAINTENANCE}/grid`
const CUSTOMERS_PATH = '/api/v1/organizations/{organization_id}/customers'
const SITES_PATH = '/api/v1/customers/{customer_id}/customer-contexts'
const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Treyve Paysages',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	slug: 'treyve',
	field_clock_enabled: false,
	vat_on_debits: false,
	mcp_enabled: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const GAUME_2026: MaintenanceContract = {
	id: 'contract-gaume',
	customer_id: 'customer-gaume',
	customer_context_id: 'site-gaume',
	project_id: 'project-1',
	reference: 'C25029',
	season: { year: 2026, first_week: 12, last_week: 48 },
	manager_member_id: 'member-1',
	amount_cents: 125000,
	status: 'ACTIVE',
	lines: [
		{
			id: 'line-tonte',
			label: 'Tonte',
			cadence: { kind: 'ROLLING', every_days: 14 },
			minutes_per_intervention: 90,
		},
	],
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

function page<T>(data: T) {
	return { data, pagination: null }
}

interface Call {
	method: string
	path: string
	params: unknown
}

function installFakeApi(initial: MaintenanceContract[]) {
	const calls: Call[] = []
	let contracts = initial
	let failWith: Error | null = null
	const fakeApi = {
		get(path: string, params: unknown) {
			const { path: pathParams, query } = (params ?? {}) as {
				path?: unknown
				query?: unknown
			}
			const queryKey = [{ _id: path, path: pathParams, query }]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						switch (path) {
							case CONTRACTS_PATH:
								return page(contracts)
							case CUSTOMERS_PATH:
								return page([
									{ id: 'customer-gaume', name: 'GAUME' },
									{ id: 'customer-other', name: 'Mairie de Vichy' },
								])
							case SITES_PATH:
								return page([
									{
										id: 'site-gaume',
										customer_id: 'customer-gaume',
										label: 'Jardin',
									},
								])
							case MEMBERS_PATH:
								return page([
									{ id: 'member-1', display_name: 'Bernard Treyve' },
								])
							default:
								throw new Error(`unmocked GET ${path}`)
						}
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
						if (failWith) {
							const failure = failWith
							failWith = null
							throw failure
						}
						if (path === CONTRACT_PATH) {
							const { path: ids, body } = params as {
								path: { contract_id: string }
								body: Partial<MaintenanceContract>
							}
							// What the API does: replace the contract's fields, keep its identity.
							contracts = contracts.map((contract) =>
								contract.id === ids.contract_id
									? ({ ...contract, ...body } as MaintenanceContract)
									: contract,
							)
							return page(contracts.find((c) => c.id === ids.contract_id))
						}
						if (path === DUPLICATE_PATH) {
							return page({
								...GAUME_2026,
								id: 'contract-gaume-2027',
								season: { ...GAUME_2026.season, year: 2027 },
							})
						}
						return page(GAUME_2026)
					},
				},
			}
		},
	}
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return {
		calls,
		failNextWith(error: Error) {
			failWith = error
		},
	}
}

function renderContracts(contracts: MaintenanceContract[]) {
	const { calls, failNextWith } = installFakeApi(contracts)
	const onSeasonChange = vi.fn()
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCacheForOrganization(
		queryClient,
		ORGANIZATION.id,
		PERMISSION_CATALOG.map((entry) => entry.name),
	)

	function Providers({ children }: { children: ReactNode }) {
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

	render(
		<Providers>
			<ContractsFeature season={2026} onSeasonChange={onSeasonChange} />
		</Providers>,
	)

	return { calls, onSeasonChange, failNextWith, queryClient }
}

describe('ContractsFeature', () => {
	it('lists the season with customer, site, cadence and amount', async () => {
		renderContracts([GAUME_2026])

		const row = await screen.findByRole('row', { name: /GAUME - Jardin/ })

		expect(within(row).getByText('C25029')).toBeTruthy()
		expect(within(row).getByText('Tonte (Tous les 14 jours)')).toBeTruthy()
		expect(within(row).getByText(/Bernard Treyve/)).toBeTruthy()
		expect(row.textContent).toContain('S12 à S48')
	})

	it('duplicates a contract for the next year and offers to go there', async () => {
		const user = userEvent.setup()
		const { calls, onSeasonChange } = renderContracts([GAUME_2026])
		await screen.findByRole('row', { name: /GAUME - Jardin/ })

		await user.click(
			screen.getByRole('button', { name: /Dupliquer pour 2027/ }),
		)

		expect(
			await screen.findByText('Contrat dupliqué pour 2027, sans intervention.'),
		).toBeTruthy()
		expect(calls).toEqual([
			{
				method: 'post',
				path: DUPLICATE_PATH,
				params: {
					path: { organization_id: 'org-1', contract_id: 'contract-gaume' },
				},
			},
		])

		await user.click(
			screen.getByRole('button', { name: 'Voir la saison 2027' }),
		)
		expect(onSeasonChange).toHaveBeenCalledWith(2027)
	})

	it('creates a contract with a line on windows of weeks', async () => {
		const user = userEvent.setup()
		const { calls, onSeasonChange } = renderContracts([])
		await screen.findByText('Aucun contrat pour la saison 2026.')

		await user.click(screen.getByRole('button', { name: 'Nouveau contrat' }))
		const dialog = await screen.findByRole('dialog')
		await user.selectOptions(
			within(dialog).getByLabelText('Client'),
			'customer-gaume',
		)
		await user.selectOptions(
			await within(dialog).findByLabelText('Site'),
			'site-gaume',
		)
		await user.type(within(dialog).getByLabelText('Code affaire'), 'C25029')
		await user.type(
			within(dialog).getByLabelText('Prestation 1'),
			'Taille de printemps',
		)
		await user.selectOptions(
			within(dialog).getByLabelText('Périodicité'),
			'WINDOWS',
		)
		await user.type(
			within(dialog).getByLabelText('Fenêtre 1, première semaine'),
			'14',
		)
		await user.type(
			within(dialog).getByLabelText('Fenêtre 1, dernière semaine'),
			'17',
		)
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() => expect(calls).toHaveLength(1))
		expect(calls[0]).toEqual({
			method: 'post',
			path: CONTRACTS_PATH,
			params: {
				path: { organization_id: 'org-1' },
				body: {
					customer_id: 'customer-gaume',
					customer_context_id: 'site-gaume',
					reference: 'C25029',
					season: { year: 2026, first_week: 12, last_week: 48 },
					manager_member_id: null,
					amount_cents: null,
					quote_id: null,
					notes: null,
					status: 'ACTIVE',
					lines: [
						{
							id: null,
							label: 'Taille de printemps',
							cadence: {
								kind: 'WINDOWS',
								windows: [{ from_week: 14, to_week: 17 }],
							},
							minutes_per_intervention: null,
						},
					],
				},
			},
		})
		await waitFor(() => expect(onSeasonChange).toHaveBeenCalledWith(2026))
	})

	it('keeps the form open and says why when windows overlap', async () => {
		const user = userEvent.setup()
		const { calls } = renderContracts([GAUME_2026])
		await screen.findByRole('row', { name: /GAUME - Jardin/ })

		await user.click(screen.getByRole('button', { name: /Modifier GAUME/ }))
		const dialog = await screen.findByRole('dialog')
		await user.selectOptions(
			within(dialog).getByLabelText('Périodicité'),
			'WINDOWS',
		)
		await user.type(
			within(dialog).getByLabelText('Fenêtre 1, première semaine'),
			'20',
		)
		await user.type(
			within(dialog).getByLabelText('Fenêtre 1, dernière semaine'),
			'25',
		)
		await user.click(
			within(dialog).getByRole('button', { name: 'Ajouter une fenêtre' }),
		)
		await user.type(
			within(dialog).getByLabelText('Fenêtre 2, première semaine'),
			'25',
		)
		await user.type(
			within(dialog).getByLabelText('Fenêtre 2, dernière semaine'),
			'30',
		)
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		expect(
			await within(dialog).findByText('Deux fenêtres se chevauchent.'),
		).toBeTruthy()
		expect(calls).toEqual([])
	})

	it('sends back the quote link and the status on an edit, since a PUT replaces everything', async () => {
		const user = userEvent.setup()
		const { calls } = renderContracts([
			{ ...GAUME_2026, quote_id: 'quote-1', status: 'DRAFT' },
		])
		await screen.findByRole('row', { name: /GAUME - Jardin/ })

		await user.click(screen.getByRole('button', { name: /Modifier GAUME/ }))
		const dialog = await screen.findByRole('dialog')
		await user.click(
			within(dialog).getByRole('button', { name: 'Enregistrer' }),
		)

		await waitFor(() => expect(calls).toHaveLength(1))
		const { body } = calls[0].params as { body: Record<string, unknown> }
		expect(calls[0].method).toBe('put')
		expect(body.quote_id).toBe('quote-1')
		expect(body.status).toBe('DRAFT')
		expect(body.amount_cents).toBe(125000)
		expect((body.lines as { id: string }[])[0].id).toBe('line-tonte')
	})

	describe('draft contracts', () => {
		const DRAFT_VICHY: MaintenanceContract = {
			...GAUME_2026,
			id: 'contract-vichy',
			customer_id: 'customer-other',
			customer_context_id: 'site-vichy',
			reference: 'C26001',
			status: 'DRAFT',
			quote_id: 'quote-9',
			notes: 'Lu sur la feuille papier',
			lines: [
				{
					id: 'line-taille',
					label: 'Taille de printemps',
					cadence: {
						kind: 'WINDOWS',
						windows: [{ from_week: 14, to_week: 17 }],
					},
					minutes_per_intervention: null,
				},
			],
		}

		function draftsSection() {
			return screen.getByRole('region', { name: 'Brouillons à vérifier' })
		}

		function activeSection() {
			return screen.getByRole('region', { name: 'Contrats actifs' })
		}

		it('shows only the draft contracts in their own section, with a badge', async () => {
			renderContracts([GAUME_2026, DRAFT_VICHY])
			await screen.findByRole('region', { name: 'Brouillons à vérifier' })

			const drafts = within(draftsSection())
			expect(drafts.getByText('C26001')).toBeTruthy()
			expect(drafts.getByText('Brouillon')).toBeTruthy()
			expect(drafts.queryByText('C25029')).toBeNull()

			const active = within(activeSection())
			expect(active.getByText('C25029')).toBeTruthy()
			expect(active.queryByText('C26001')).toBeNull()
			expect(active.queryByText('Brouillon')).toBeNull()
		})

		it('has no draft section when every contract is active', async () => {
			renderContracts([GAUME_2026])
			await screen.findByRole('row', { name: /GAUME - Jardin/ })

			expect(
				screen.queryByRole('region', { name: 'Brouillons à vérifier' }),
			).toBeNull()
		})

		it('activates a draft after a confirmation: the same PUT with only the status changed, then it joins the active list', async () => {
			const user = userEvent.setup()
			const { calls } = renderContracts([GAUME_2026, DRAFT_VICHY])
			await screen.findByRole('region', { name: 'Brouillons à vérifier' })

			await user.click(
				within(draftsSection()).getByRole('button', { name: /Activer/ }),
			)
			expect(calls).toEqual([])
			const confirmation = await screen.findByRole('alertdialog')
			await user.click(
				within(confirmation).getByRole('button', { name: 'Activer' }),
			)

			await waitFor(() => expect(calls).toHaveLength(1))
			expect(calls[0]).toEqual({
				method: 'put',
				path: CONTRACT_PATH,
				params: {
					path: { organization_id: 'org-1', contract_id: 'contract-vichy' },
					body: {
						reference: 'C26001',
						season: { year: 2026, first_week: 12, last_week: 48 },
						manager_member_id: 'member-1',
						amount_cents: 125000,
						quote_id: 'quote-9',
						notes: 'Lu sur la feuille papier',
						status: 'ACTIVE',
						lines: [
							{
								id: 'line-taille',
								label: 'Taille de printemps',
								cadence: {
									kind: 'WINDOWS',
									windows: [{ from_week: 14, to_week: 17 }],
								},
								minutes_per_intervention: null,
							},
						],
					},
				},
			})

			await waitFor(() =>
				expect(
					screen.queryByRole('region', { name: 'Brouillons à vérifier' }),
				).toBeNull(),
			)
			const active = within(activeSection())
			expect(active.getByText('C26001')).toBeTruthy()
			expect(active.queryByText('Brouillon')).toBeNull()
		})

		it('sends nothing when the confirmation is cancelled', async () => {
			const user = userEvent.setup()
			const { calls } = renderContracts([DRAFT_VICHY])
			await screen.findByRole('region', { name: 'Brouillons à vérifier' })

			await user.click(
				within(draftsSection()).getByRole('button', { name: /Activer/ }),
			)
			const confirmation = await screen.findByRole('alertdialog')
			await user.click(
				within(confirmation).getByRole('button', { name: 'Annuler' }),
			)

			expect(calls).toEqual([])
			expect(within(draftsSection()).getByText('C26001')).toBeTruthy()
		})

		it.each([
			['409', 'Le contrat a été modifié : une semaine sort de la saison.'],
			['404', 'Contrat introuvable.'],
		])('shows the API refusal (%s) and keeps the contract a draft', async (_status, message) => {
			const user = userEvent.setup()
			const { calls, failNextWith } = renderContracts([DRAFT_VICHY])
			await screen.findByRole('region', { name: 'Brouillons à vérifier' })
			failNextWith(new Error(message))

			await user.click(
				within(draftsSection()).getByRole('button', { name: /Activer/ }),
			)
			const confirmation = await screen.findByRole('alertdialog')
			await user.click(
				within(confirmation).getByRole('button', { name: 'Activer' }),
			)

			expect((await screen.findByRole('alert')).textContent).toContain(message)
			expect(calls).toHaveLength(1)
			expect(within(draftsSection()).getByText('C26001')).toBeTruthy()
		})

		it('refreshes the grid and the recap after an activation', async () => {
			const user = userEvent.setup()
			const { calls, queryClient } = renderContracts([DRAFT_VICHY])
			await screen.findByRole('region', { name: 'Brouillons à vérifier' })
			const gridKey = [
				{
					_id: GRID_PATH,
					path: { organization_id: 'org-1' },
					query: { season: 2026 },
				},
			]
			queryClient.setQueryData(gridKey, { data: { year: 2026, contracts: [] } })

			await user.click(
				within(draftsSection()).getByRole('button', { name: /Activer/ }),
			)
			await user.click(
				within(await screen.findByRole('alertdialog')).getByRole('button', {
					name: 'Activer',
				}),
			)

			await waitFor(() => expect(calls).toHaveLength(1))
			await waitFor(() =>
				expect(queryClient.getQueryState(gridKey)?.isInvalidated).toBe(true),
			)
		})

		it('opens a draft in the contract form to edit it before activating', async () => {
			const user = userEvent.setup()
			const { calls } = renderContracts([DRAFT_VICHY])
			await screen.findByRole('region', { name: 'Brouillons à vérifier' })

			await user.click(
				within(draftsSection()).getByRole('button', { name: /Modifier/ }),
			)
			const dialog = await screen.findByRole('dialog')
			expect(
				(within(dialog).getByLabelText('Code affaire') as HTMLInputElement)
					.value,
			).toBe('C26001')
			await user.click(
				within(dialog).getByRole('button', { name: 'Enregistrer' }),
			)

			await waitFor(() => expect(calls).toHaveLength(1))
			const { body } = calls[0].params as { body: Record<string, unknown> }
			expect(calls[0].method).toBe('put')
			expect(body.status).toBe('DRAFT')
		})
	})
})
