import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
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

const MEMBERS_PATH = '/api/v1/organizations/{organization_id}/members'
const EMPLOYEE_PROFILES_PATH =
	'/api/v1/organizations/{organization_id}/employee-profiles'
const EMPLOYEE_PROFILE_PATH = '/api/v1/members/{member_id}/employee-profile'
const WORK_TIME_PATH = '/api/v1/members/{member_id}/work-time'
const RHYTHM_PATH = '/api/v1/members/{member_id}/rhythm'
const WORK_SLOTS_PATH = '/api/v1/members/{member_id}/work-slots'
const ABSENCES_PATH = '/api/v1/organizations/{organization_id}/absences'
const ABSENCE_PATH =
	'/api/v1/organizations/{organization_id}/absences/{absence_id}'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	slug: 'atelier-bois',
	field_clock_enabled: false,
	vat_on_debits: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

interface FakeMember {
	id: string
	organization_id: string
	last_name: string
	first_name: string | null
	display_name: string
	account: { email: string; name: string } | null
	joined_at: string | null
	created_at: string
}

interface FakeEmployeeProfile {
	id: string
	organization_id: string
	member_id: string
	hourly_rate_cents: number | null
	weekly_contract_minutes: number
	created_at: string
	updated_at: string
}

const MEMBER: FakeMember = {
	id: 'member-1',
	organization_id: 'org-1',
	last_name: 'Martin',
	first_name: 'Alix',
	display_name: 'Martin Alix',
	account: null,
	joined_at: null,
	created_at: '2026-01-01T00:00:00Z',
}

const PROFILE: FakeEmployeeProfile = {
	id: 'employee-1',
	organization_id: 'org-1',
	member_id: 'member-1',
	hourly_rate_cents: 1500,
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
	upsertProfile?: (params: unknown) => unknown
	postAbsence?: (params: unknown) => unknown
	patchAbsence?: (params: unknown) => unknown
	deleteAbsence?: (params: unknown) => unknown
	members?: FakeMember[]
	profiles?: FakeEmployeeProfile[]
	absences?: Record<string, unknown>[]
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
	const calls: { method: string; path: string; params: unknown }[] = []
	const members = handlers.members ?? [MEMBER]
	const profiles = handlers.profiles ?? [PROFILE]
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
						if (path === MEMBERS_PATH) {
							return { data: members, pagination: null }
						}
						if (path === EMPLOYEE_PROFILES_PATH) {
							return { data: profiles, pagination: null }
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
						if (method === 'put' && path === EMPLOYEE_PROFILE_PATH) {
							if (!handlers.upsertProfile)
								throw new Error('upsertProfile not mocked')
							return handlers.upsertProfile(params)
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
			<EmployeeWorkTimeFeature memberId="member-1" />
		</Providers>,
	)

	return { calls }
}

describe('EmployeeWorkTimeFeature', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it("loads and shows the member's current rhythm", async () => {
		renderFeature()

		expect(await screen.findByText('Martin Alix')).toBeDefined()
		const lundiSection = await screen.findByRole('region', { name: 'Lundi' })
		expect(within(lundiSection).getByLabelText('Début')).toBeDefined()
	})

	it('sends the right PUT rhythm payload on save', async () => {
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
			path: { member_id: 'member-1' },
			body: {
				effective_from: '2026-06-01',
				effective_to: null,
				slots: [{ weekday: 1, starts_minute: 480, ends_minute: 720 }],
			},
		})
	})

	it('shows a clear message — not a raw error — when the rhythm returns a 409', async () => {
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

	it('sends the right PUT work-slots payload on save', async () => {
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
			path: { member_id: 'member-1' },
			query: expect.objectContaining({
				from: expect.any(String),
				to: expect.any(String),
			}),
			body: { slots: expect.arrayContaining([expect.any(Object)]) },
		})
	})

	it('sends the right PUT employee-profile payload for the contractual baseline, preserving the existing rate', async () => {
		const user = userEvent.setup()
		const upsertProfile = vi.fn().mockResolvedValue(PROFILE)
		const { calls } = renderFeature({ upsertProfile })

		await screen.findByText('Martin Alix')
		const contractInput = screen.getByLabelText('Base contractuelle')
		await user.clear(contractInput)
		await user.type(contractInput, '30h00')
		await user.click(
			screen.getByRole('button', { name: 'Enregistrer la base contractuelle' }),
		)

		await waitFor(() => expect(upsertProfile).toHaveBeenCalledTimes(1))
		const call = calls.find(
			(c) => c.method === 'put' && c.path === EMPLOYEE_PROFILE_PATH,
		)
		expect(call?.params).toMatchObject({
			path: { member_id: 'member-1' },
			body: {
				hourly_rate_cents: 1500,
				weekly_contract_minutes: 1800,
			},
		})
	})

	it('a member with no employee profile shows an unset rate and a zero contractual baseline', async () => {
		renderFeature({ members: [MEMBER], profiles: [] })

		await screen.findByText('Martin Alix')
		expect(screen.getByText('Non renseigné')).toBeDefined()
		expect(
			(screen.getByLabelText('Base contractuelle') as HTMLInputElement).value,
		).toBe('0h00')
	})
})

describe('EmployeeWorkTimeFeature — absences', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	const MEMBER_2: FakeMember = {
		id: 'member-2',
		organization_id: 'org-1',
		last_name: 'Petit',
		first_name: null,
		display_name: 'Petit',
		account: null,
		joined_at: null,
		created_at: '2026-01-01T00:00:00Z',
	}

	function absence(overrides: Record<string, unknown> = {}) {
		return {
			id: 'ab-1',
			organization_id: 'org-1',
			member_id: 'member-1',
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

	it("shows only the absences of the page's member", async () => {
		renderFeature({
			absences: [
				absence({ id: 'ab-1', member_id: 'member-1' }),
				absence({ id: 'ab-2', member_id: 'member-2' }),
			],
		})

		await screen.findByText('Martin Alix')
		expect(await screen.findByText(/Congé —/)).toBeDefined()
		expect(screen.getAllByText(/Congé —/)).toHaveLength(1)
	})

	it('editing an absence prefills the form and sends a PATCH without member_id', async () => {
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
		expect(call?.params).not.toHaveProperty('body.member_id')
	})

	it('deleting fires a DELETE and closes the form', async () => {
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

	it("creating prefills the page's member and sends a POST with their member_id", async () => {
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
			body: { member_id: 'member-1', kind: 'LEAVE' },
		})
	})

	it('the member picker shows the roster’s display name, including a member with no first name', async () => {
		const user = userEvent.setup()
		renderFeature({ members: [MEMBER, MEMBER_2] })

		await screen.findByText('Martin Alix')
		await user.click(
			screen.getByRole('button', { name: /Ajouter une absence/ }),
		)
		const sheet = await screen.findByRole('dialog')
		await user.click(within(sheet).getByRole('combobox', { name: 'Personne' }))

		expect(
			await screen.findByRole('option', { name: 'Martin Alix' }),
		).toBeDefined()
		expect(screen.getByRole('option', { name: 'Petit' })).toBeDefined()
	})
})
