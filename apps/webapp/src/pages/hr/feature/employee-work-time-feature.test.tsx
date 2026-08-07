import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { ActiveOrganizationProvider } from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { EmployeeWorkTimeFeature } from '#/pages/hr/feature/employee-work-time-feature'

// jsdom has no ResizeObserver, and Radix `Select`'s listbox needs
// `scrollIntoView`/pointer-capture methods it also doesn't implement.
// Stubbed locally — this workstream doesn't own `vitest.setup.ts`.
class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
// biome-ignore lint/suspicious/noExplicitAny: test-only global polyfills
const globalAny = globalThis as any
globalAny.ResizeObserver ??= ResizeObserverStub
Element.prototype.scrollIntoView ??= () => {}
Element.prototype.hasPointerCapture ??= () => false
Element.prototype.releasePointerCapture ??= () => {}

const EMPLOYEES_PATH = '/api/v1/organizations/{organization_id}/employees'
const EMPLOYEE_PATH = '/api/v1/employees/{employee_id}'
const WORK_TIME_PATH =
	'/api/v1/organizations/{organization_id}/employees/{employee_id}/work-time'
const RHYTHM_PATH =
	'/api/v1/organizations/{organization_id}/employees/{employee_id}/rhythm'
const WORK_SLOTS_PATH =
	'/api/v1/organizations/{organization_id}/employees/{employee_id}/work-slots'
const ABSENCES_PATH = '/api/v1/organizations/{organization_id}/absences'
const ABSENCE_PATH =
	'/api/v1/organizations/{organization_id}/absences/{absence_id}'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	owner_id: 'user-1',
	slug: 'atelier-bois',
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

interface FakeEmployee {
	id: string
	organization_id: string
	last_name: string
	first_name: string | null
	hourly_rate_cents: number
	user_id: string | null
	weekly_contract_minutes: number
	created_at: string
	updated_at: string
}

const EMPLOYEE: FakeEmployee = {
	id: 'employee-1',
	organization_id: 'org-1',
	last_name: 'Martin',
	first_name: 'Alix',
	hourly_rate_cents: 1500,
	user_id: null,
	weekly_contract_minutes: 2100,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const OPEN_RHYTHM = {
	id: 'rhythm-1',
	organization_id: 'org-1',
	employee_id: 'employee-1',
	effective_from: '2026-06-01',
	effective_to: null,
	slots: [{ weekday: 1, starts_minute: 480, ends_minute: 720 }],
	created_at: '2026-06-01T00:00:00Z',
	updated_at: '2026-06-01T00:00:00Z',
}

interface FakeApiHandlers {
	putRhythm?: (params: unknown) => unknown
	putWorkSlots?: (params: unknown) => unknown
	patchEmployee?: (params: unknown) => unknown
	postAbsence?: (params: unknown) => unknown
	patchAbsence?: (params: unknown) => unknown
	deleteAbsence?: (params: unknown) => unknown
	employees?: FakeEmployee[]
	absences?: Record<string, unknown>[]
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
	const calls: { method: string; path: string; params: unknown }[] = []
	const employees = handlers.employees ?? [EMPLOYEE]
	const absences = handlers.absences ?? []

	function queryKeyFor(path: string, params: unknown) {
		const p = (params ?? {}) as { path?: unknown; query?: unknown }
		return [{ _id: path, path: p.path, query: p.query }]
	}

	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = queryKeyFor(path, params)
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params })
						if (path === EMPLOYEES_PATH) {
							return { data: employees, pagination: null }
						}
						if (path === WORK_TIME_PATH) {
							return {
								data: { rhythms: [OPEN_RHYTHM], work_slots: [] },
								pagination: null,
							}
						}
						if (path === ABSENCES_PATH) {
							return { data: absences, pagination: null }
						}
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
					mutationFn: async (params: unknown) => {
						calls.push({ method, path, params })
						if (method === 'put' && path === RHYTHM_PATH) {
							if (!handlers.putRhythm) throw new Error('putRhythm not mocked')
							return handlers.putRhythm(params)
						}
						if (method === 'put' && path === WORK_SLOTS_PATH) {
							if (!handlers.putWorkSlots)
								throw new Error('putWorkSlots not mocked')
							return handlers.putWorkSlots(params)
						}
						if (method === 'patch' && path === EMPLOYEE_PATH) {
							if (!handlers.patchEmployee)
								throw new Error('patchEmployee not mocked')
							return handlers.patchEmployee(params)
						}
						if (method === 'post' && path === ABSENCES_PATH) {
							if (!handlers.postAbsence)
								throw new Error('postAbsence not mocked')
							return handlers.postAbsence(params)
						}
						if (method === 'patch' && path === ABSENCE_PATH) {
							if (!handlers.patchAbsence)
								throw new Error('patchAbsence not mocked')
							return handlers.patchAbsence(params)
						}
						if (method === 'delete' && path === ABSENCE_PATH) {
							if (!handlers.deleteAbsence)
								throw new Error('deleteAbsence not mocked')
							return handlers.deleteAbsence(params)
						}
						throw new Error(`unmocked mutation ${method} ${path}`)
					},
				},
			}
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return calls
}

function renderFeature(handlers: FakeApiHandlers = {}) {
	const calls = installFakeTanstackApi(handlers)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider client={queryClient}>
				<ActiveOrganizationProvider organizations={[ORGANIZATION]}>
					{children}
				</ActiveOrganizationProvider>
			</QueryClientProvider>
		)
	}

	render(
		<Providers>
			<EmployeeWorkTimeFeature employeeId="employee-1" />
		</Providers>,
	)

	return { calls }
}

describe('EmployeeWorkTimeFeature', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it("charge et affiche le rythme actuel de l'employé", async () => {
		renderFeature()

		expect(await screen.findByText('Martin Alix')).toBeDefined()
		const lundiSection = await screen.findByRole('region', { name: 'Lundi' })
		expect(within(lundiSection).getByLabelText('Début')).toBeDefined()
	})

	it('envoie le bon payload PUT rhythm à la sauvegarde', async () => {
		const user = userEvent.setup()
		const putRhythm = vi.fn().mockResolvedValue(OPEN_RHYTHM)
		const { calls } = renderFeature({ putRhythm })

		await screen.findByText('Martin Alix')
		await user.click(
			screen.getByRole('button', { name: 'Enregistrer le rythme' }),
		)

		await waitFor(() => expect(putRhythm).toHaveBeenCalledTimes(1))
		const call = calls.find((c) => c.method === 'put' && c.path === RHYTHM_PATH)
		expect(call?.params).toMatchObject({
			path: { organization_id: 'org-1', employee_id: 'employee-1' },
			body: {
				effective_from: '2026-06-01',
				effective_to: null,
				slots: [{ weekday: 1, starts_minute: 480, ends_minute: 720 }],
			},
		})
	})

	it('affiche un message clair — pas une erreur brute — quand le rythme renvoie un 409', async () => {
		const user = userEvent.setup()
		const conflictError: Error & { status?: number } = new Error(
			'Conflict: cannot start a rhythm version before the one currently in effect',
		)
		conflictError.status = 409
		const putRhythm = vi.fn().mockRejectedValue(conflictError)
		renderFeature({ putRhythm })

		await screen.findByText('Martin Alix')
		await user.click(
			screen.getByRole('button', { name: 'Enregistrer le rythme' }),
		)

		await waitFor(() => expect(putRhythm).toHaveBeenCalledTimes(1))
		expect(
			await screen.findByText(/Impossible de faire démarrer cette version/),
		).toBeDefined()
		expect(
			screen.queryByText(
				'Conflict: cannot start a rhythm version before the one currently in effect',
			),
		).toBeNull()
	})

	it('envoie le bon payload PUT work-slots à la sauvegarde', async () => {
		const user = userEvent.setup()
		const putWorkSlots = vi.fn().mockResolvedValue([])
		const { calls } = renderFeature({ putWorkSlots })

		await screen.findByText('Martin Alix')
		await user.click(screen.getByRole('button', { name: 'Ajouter une plage' }))
		await user.click(
			screen.getByRole('button', { name: 'Enregistrer les plages' }),
		)

		await waitFor(() => expect(putWorkSlots).toHaveBeenCalledTimes(1))
		const call = calls.find(
			(c) => c.method === 'put' && c.path === WORK_SLOTS_PATH,
		)
		expect(call?.params).toMatchObject({
			path: { organization_id: 'org-1', employee_id: 'employee-1' },
			query: expect.objectContaining({
				from: expect.any(String),
				to: expect.any(String),
			}),
			body: { slots: expect.arrayContaining([expect.any(Object)]) },
		})
	})

	it('envoie le bon payload PATCH employee pour la base contractuelle', async () => {
		const user = userEvent.setup()
		const patchEmployee = vi.fn().mockResolvedValue(EMPLOYEE)
		const { calls } = renderFeature({ patchEmployee })

		await screen.findByText('Martin Alix')
		const contractInput = screen.getByLabelText('Base contractuelle')
		await user.clear(contractInput)
		await user.type(contractInput, '30h00')
		await user.click(
			screen.getByRole('button', { name: 'Enregistrer la base contractuelle' }),
		)

		await waitFor(() => expect(patchEmployee).toHaveBeenCalledTimes(1))
		const call = calls.find(
			(c) => c.method === 'patch' && c.path === EMPLOYEE_PATH,
		)
		expect(call?.params).toMatchObject({
			path: { employee_id: 'employee-1' },
			body: {
				last_name: 'Martin',
				first_name: 'Alix',
				hourly_rate_cents: 1500,
				user_id: null,
				weekly_contract_minutes: 1800,
			},
		})
	})
})

describe('EmployeeWorkTimeFeature — absences', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	const EMPLOYEE_2 = {
		id: 'employee-2',
		organization_id: 'org-1',
		last_name: 'Petit',
		first_name: null,
		hourly_rate_cents: 1400,
		user_id: null,
		weekly_contract_minutes: 2100,
		created_at: '2026-01-01T00:00:00Z',
		updated_at: '2026-01-01T00:00:00Z',
	}

	function absence(overrides: Record<string, unknown> = {}) {
		return {
			id: 'ab-1',
			organization_id: 'org-1',
			employee_id: 'employee-1',
			kind: 'LEAVE',
			all_day: true,
			starts_at: '2026-08-10T00:00:00Z',
			ends_at: '2026-08-11T00:00:00Z',
			note: 'Vacances',
			created_at: '2026-08-01T00:00:00Z',
			updated_at: '2026-08-01T00:00:00Z',
			...overrides,
		}
	}

	it("affiche uniquement les absences de l'employé de la page", async () => {
		renderFeature({
			absences: [
				absence({ id: 'ab-1', employee_id: 'employee-1' }),
				absence({ id: 'ab-2', employee_id: 'employee-2' }),
			],
		})

		await screen.findByText('Martin Alix')
		expect(await screen.findByText(/Congé —/)).toBeDefined()
		expect(screen.getAllByText(/Congé —/)).toHaveLength(1)
	})

	it("l'édition d'une absence pré-remplit le formulaire et envoie un PATCH sans employee_id", async () => {
		const user = userEvent.setup()
		const patchAbsence = vi.fn().mockResolvedValue(absence())
		const { calls } = renderFeature({
			absences: [absence()],
			patchAbsence,
		})

		await screen.findByText('Martin Alix')
		await user.click(await screen.findByText(/Congé —/))

		const sheet = await screen.findByRole('dialog')
		expect(within(sheet).getByText('Modifier l’absence')).toBeDefined()

		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => expect(patchAbsence).toHaveBeenCalledTimes(1))
		const call = calls.find(
			(c) => c.method === 'patch' && c.path === ABSENCE_PATH,
		)
		expect(call?.params).toMatchObject({
			path: { organization_id: 'org-1', absence_id: 'ab-1' },
		})
		expect(call?.params).not.toHaveProperty('body.employee_id')
	})

	it('la suppression déclenche un DELETE et ferme le formulaire', async () => {
		const user = userEvent.setup()
		const deleteAbsence = vi.fn().mockResolvedValue(undefined)
		const { calls } = renderFeature({
			absences: [absence()],
			deleteAbsence,
		})

		await screen.findByText('Martin Alix')
		await user.click(await screen.findByText(/Congé —/))
		await screen.findByRole('dialog')

		await user.click(screen.getByRole('button', { name: /Supprimer/ }))

		await waitFor(() => expect(deleteAbsence).toHaveBeenCalledTimes(1))
		const call = calls.find(
			(c) => c.method === 'delete' && c.path === ABSENCE_PATH,
		)
		expect(call?.params).toMatchObject({
			path: { organization_id: 'org-1', absence_id: 'ab-1' },
		})
		await waitFor(() => expect(screen.queryByRole('dialog')).toBeNull())
	})

	it("la création pré-remplit l'employé de la page et envoie un POST avec son employee_id", async () => {
		const user = userEvent.setup()
		const postAbsence = vi.fn().mockResolvedValue(absence({ id: 'ab-new' }))
		const { calls } = renderFeature({ postAbsence })

		await screen.findByText('Martin Alix')
		await user.click(
			screen.getByRole('button', { name: /Ajouter une absence/ }),
		)

		const sheet = await screen.findByRole('dialog')
		expect(within(sheet).getByText('Nouvelle absence')).toBeDefined()

		await user.click(screen.getByRole('button', { name: /Créer l’absence/ }))

		await waitFor(() => expect(postAbsence).toHaveBeenCalledTimes(1))
		const call = calls.find(
			(c) => c.method === 'post' && c.path === ABSENCES_PATH,
		)
		expect(call?.params).toMatchObject({
			path: { organization_id: 'org-1' },
			body: { employee_id: 'employee-1', kind: 'LEAVE' },
		})
	})

	it('le sélecteur employé affiche « {nom} {prénom} » et le nom seul sans prénom', async () => {
		const user = userEvent.setup()
		renderFeature({ employees: [EMPLOYEE, EMPLOYEE_2] })

		await screen.findByText('Martin Alix')
		await user.click(
			screen.getByRole('button', { name: /Ajouter une absence/ }),
		)
		const sheet = await screen.findByRole('dialog')
		await user.click(within(sheet).getByRole('combobox', { name: 'Employé' }))

		expect(
			await screen.findByRole('option', { name: 'Martin Alix' }),
		).toBeDefined()
		expect(screen.getByRole('option', { name: 'Petit' })).toBeDefined()
	})
})
