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
import { AbsencesOverviewFeature } from '#/pages/hr/feature/absences-overview-feature'

// jsdom has no ResizeObserver, and Radix `Select`'s listbox needs
// `scrollIntoView`/pointer-capture methods it also doesn't implement — the
// absence form's member picker uses one. Stubbed locally, mirroring
// `employee-work-time-feature.test.tsx` (this workstream doesn't own
// `vitest.setup.ts`).
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

const MEMBER_1: FakeMember = {
	id: 'member-1',
	organization_id: 'org-1',
	last_name: 'Martin',
	first_name: 'Alix',
	display_name: 'Martin Alix',
	account: null,
	joined_at: null,
	created_at: '2026-01-01T00:00:00Z',
}

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

interface FakeApiHandlers {
	members?: FakeMember[]
	absences?: Record<string, unknown>[]
	postAbsence?: (params: unknown) => unknown
	patchAbsence?: (params: unknown) => unknown
	deleteAbsence?: (params: unknown) => unknown
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
	const calls: { method: string; path: string; params: unknown }[] = []
	const members = handlers.members ?? [MEMBER_1, MEMBER_2]
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
						if (method === 'post' && path === ABSENCES_PATH) {
							if (!handlers.postAbsence) {
								throw new Error('postAbsence not mocked')
							}
							return handlers.postAbsence(params)
						}
						if (method === 'patch' && path === ABSENCE_PATH) {
							if (!handlers.patchAbsence) {
								throw new Error('patchAbsence not mocked')
							}
							return handlers.patchAbsence(params)
						}
						if (method === 'delete' && path === ABSENCE_PATH) {
							if (!handlers.deleteAbsence) {
								throw new Error('deleteAbsence not mocked')
							}
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
			<AbsencesOverviewFeature />
		</Providers>,
	)

	return { calls }
}

describe('AbsencesOverviewFeature — listing', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('lists every absence in the organization, across members, with the resolved name', async () => {
		renderFeature({
			absences: [
				absence({ id: 'ab-1', member_id: 'member-1' }),
				absence({ id: 'ab-2', member_id: 'member-2' }),
			],
		})

		expect(await screen.findByText('Martin Alix')).toBeDefined()
		expect(screen.getByText('Petit')).toBeDefined()
		expect(screen.getByText('Absences (2)')).toBeDefined()
	})

	it('falls back to a placeholder name when the absence points at an unknown member', async () => {
		renderFeature({
			absences: [absence({ id: 'ab-1', member_id: 'member-ghost' })],
		})

		expect(await screen.findByText('Personne inconnue')).toBeDefined()
	})
})

describe('AbsencesOverviewFeature — create', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('opens the shared form and sends a POST with the picked member', async () => {
		const user = userEvent.setup()
		const postAbsence = vi.fn().mockResolvedValue(absence({ id: 'ab-new' }))
		const { calls } = renderFeature({ postAbsence })

		await screen.findByText('Absences (0)')
		await user.click(
			screen.getByRole('button', { name: /Ajouter une absence/ }),
		)

		const sheet = await screen.findByRole('dialog')
		expect(within(sheet).getByText('Nouvelle absence')).toBeDefined()

		await user.click(within(sheet).getByRole('combobox', { name: 'Personne' }))
		await user.click(await screen.findByRole('option', { name: 'Martin Alix' }))
		await user.click(
			within(sheet).getByRole('button', { name: /Créer l’absence/ }),
		)

		await waitFor(() => expect(postAbsence).toHaveBeenCalledTimes(1))
		const call = calls.find(
			(c) => c.method === 'post' && c.path === ABSENCES_PATH,
		)
		expect(call?.params).toMatchObject({
			path: { organization_id: 'org-1' },
			body: { member_id: 'member-1', kind: 'LEAVE' },
		})
	})
})

describe('AbsencesOverviewFeature — edit and delete from the row', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('"Modifier" prefills the shared form and sends a PATCH without member_id', async () => {
		const user = userEvent.setup()
		const patchAbsence = vi.fn().mockResolvedValue(absence())
		const { calls } = renderFeature({
			absences: [absence()],
			patchAbsence,
		})

		await screen.findByText('Martin Alix')
		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Modifier/ }))

		const sheet = await screen.findByRole('dialog')
		expect(within(sheet).getByText('Modifier l’absence')).toBeDefined()

		await user.click(within(sheet).getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => expect(patchAbsence).toHaveBeenCalledTimes(1))
		const call = calls.find(
			(c) => c.method === 'patch' && c.path === ABSENCE_PATH,
		)
		expect(call?.params).toMatchObject({
			path: { organization_id: 'org-1', absence_id: 'ab-1' },
		})
		expect(call?.params).not.toHaveProperty('body.member_id')
	})

	it('"Supprimer" from the row menu asks for confirmation, then sends a DELETE', async () => {
		const user = userEvent.setup()
		const deleteAbsence = vi.fn().mockResolvedValue(undefined)
		const { calls } = renderFeature({
			absences: [absence()],
			deleteAbsence,
		})

		await screen.findByText('Martin Alix')
		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: /Supprimer/ }))

		expect(deleteAbsence).not.toHaveBeenCalled()
		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		await waitFor(() => expect(deleteAbsence).toHaveBeenCalledTimes(1))
		const call = calls.find(
			(c) => c.method === 'delete' && c.path === ABSENCE_PATH,
		)
		expect(call?.params).toMatchObject({
			path: { organization_id: 'org-1', absence_id: 'ab-1' },
		})
	})
})
