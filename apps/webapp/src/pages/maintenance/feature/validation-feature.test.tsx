import { QueryClient } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { MaintenanceIntervention } from '#/hooks/use-maintenance-validation'
import { ValidationFeature } from '#/pages/maintenance/feature/validation-feature'
import { wrapWithPermissions } from '#/test/with-permissions'

const INTERVENTIONS_PATH =
	'/api/v1/organizations/{organization_id}/maintenance/interventions'
const GRID_PATH = '/api/v1/organizations/{organization_id}/maintenance/grid'
const PLANNING_PATH = '/api/v1/organizations/{organization_id}/planning'
const VALIDATE_PATH =
	'/api/v1/organizations/{organization_id}/maintenance/interventions/{intervention_id}/validate'
const REJECT_PATH =
	'/api/v1/organizations/{organization_id}/maintenance/interventions/{intervention_id}/reject'

function visit(
	id: string,
	lineId: string,
	doneOn: string,
): MaintenanceIntervention {
	return {
		id,
		line_id: lineId,
		done_on: doneOn,
		status: 'PROPOSED',
		proposed_by: 'member-1',
		created_at: `${doneOn}T16:00:00Z`,
	}
}

const GRID = {
	year: 2026,
	contracts: [
		{
			id: 'contract-1',
			reference: 'C25029',
			site: { customer_name: 'Gaume', site_label: 'Siège', city: 'Vichy' },
			lines: [
				{ id: 'line-tonte', label: 'Tonte' },
				{ id: 'line-haie', label: 'Taille de haie' },
			],
		},
	],
}

/**
 * The queue is server state: validating or rejecting a visit takes it out,
 * and the screen only learns that by asking again.
 */
function installFakeApi(initial: MaintenanceIntervention[]) {
	let queue = [...initial]
	const calls = { validate: vi.fn(), reject: vi.fn() }
	let validateError: Error | null = null

	function resolve(params: unknown) {
		const { intervention_id } = (
			params as { path: { intervention_id: string } }
		).path
		queue = queue.filter((item) => item.id !== intervention_id)
		return { data: {}, pagination: null }
	}
	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = {
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
						if (path === INTERVENTIONS_PATH) {
							return {
								data: queue,
								pagination: { total: queue.length, page: 1, per_page: 100 },
							}
						}
						if (path === GRID_PATH) return { data: GRID, pagination: null }
						if (path === PLANNING_PATH) {
							return {
								data: {
									resources: [
										{ member_id: 'member-1', display_name: 'Bernard Petit' },
									],
									entries: [],
									work_time: [],
								},
								pagination: null,
							}
						}
						throw new Error(`unmocked GET ${path}`)
					},
				},
			}
		},
		mutation(method: string, path: string) {
			return {
				mutationOptions: {
					mutationKey: [{ method, path }],
					mutationFn: async (params: unknown) => {
						if (path === VALIDATE_PATH) {
							calls.validate(params)
							if (validateError) throw validateError
							return resolve(params)
						}
						if (path === REJECT_PATH) {
							calls.reject(params)
							return resolve(params)
						}
						throw new Error(`unmocked ${method} ${path}`)
					},
				},
			}
		},
	}

	return {
		...calls,
		failValidateWith(error: Error) {
			validateError = error
		},
	}
}

function renderQueue(initial: MaintenanceIntervention[]) {
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	const ui = wrapWithPermissions(<ValidationFeature />, { queryClient })
	const api = installFakeApi(initial)
	render(ui)
	return api
}

function rowOf(text: string): HTMLElement {
	const row = screen.getByText(text).closest('li')
	if (!row) throw new Error(`no row for ${text}`)
	return row
}

describe('ValidationFeature', () => {
	it('lists each proposed visit with its line, its site and who proposed it', async () => {
		renderQueue([visit('v1', 'line-tonte', '2026-10-14')])

		expect(await screen.findByText('Tonte')).toBeTruthy()
		const row = rowOf('Tonte')
		expect(within(row).getByText(/gaume · siège \(vichy\)/i)).toBeTruthy()
		expect(within(row).getByText(/bernard petit/i)).toBeTruthy()
		expect(within(row).getByText(/14 oct\./i)).toBeTruthy()
	})

	it('validating a visit removes it from the queue', async () => {
		const user = userEvent.setup()
		const api = renderQueue([
			visit('v1', 'line-tonte', '2026-10-14'),
			visit('v2', 'line-haie', '2026-10-13'),
		])
		await screen.findByText('Tonte')

		await user.click(
			within(rowOf('Tonte')).getByRole('button', { name: /valider/i }),
		)

		await waitFor(() => expect(screen.queryByText('Tonte')).toBeNull())
		expect(screen.getByText('Taille de haie')).toBeTruthy()
		expect(api.validate).toHaveBeenCalledTimes(1)
		expect(api.validate).toHaveBeenCalledWith({
			path: { organization_id: 'org-1', intervention_id: 'v1' },
		})
	})

	it('rejecting a visit sends the note and removes it from the queue', async () => {
		const user = userEvent.setup()
		const api = renderQueue([visit('v1', 'line-tonte', '2026-10-14')])
		await screen.findByText('Tonte')

		await user.click(
			within(rowOf('Tonte')).getByRole('button', { name: /rejeter/i }),
		)
		await user.type(screen.getByLabelText(/motif du rejet/i), 'Pas ce site')
		await user.click(
			screen.getByRole('button', { name: /confirmer le rejet/i }),
		)

		await waitFor(() => expect(screen.queryByText('Tonte')).toBeNull())
		expect(api.reject).toHaveBeenCalledWith({
			path: { organization_id: 'org-1', intervention_id: 'v1' },
			body: { note: 'Pas ce site' },
		})
		expect(screen.getByText(/aucune visite à valider/i)).toBeTruthy()
	})

	it('lets the manager back out of a rejection', async () => {
		const user = userEvent.setup()
		const api = renderQueue([visit('v1', 'line-tonte', '2026-10-14')])
		await screen.findByText('Tonte')

		await user.click(
			within(rowOf('Tonte')).getByRole('button', { name: /rejeter/i }),
		)
		await user.click(screen.getByRole('button', { name: /annuler/i }))

		expect(screen.queryByLabelText(/motif du rejet/i)).toBeNull()
		expect(api.reject).not.toHaveBeenCalled()
	})

	it('says so when someone else already resolved the visit', async () => {
		const user = userEvent.setup()
		const api = renderQueue([visit('v1', 'line-tonte', '2026-10-14')])
		api.failValidateWith(Object.assign(new Error('resolved'), { status: 409 }))
		await screen.findByText('Tonte')

		await user.click(
			within(rowOf('Tonte')).getByRole('button', { name: /valider/i }),
		)

		const alert = await screen.findByRole('alert')
		expect(within(alert).getByText(/déjà été traitée/i)).toBeTruthy()
	})

	it('says so when the queue is empty', async () => {
		renderQueue([])

		expect(await screen.findByText(/aucune visite à valider/i)).toBeTruthy()
	})
})
