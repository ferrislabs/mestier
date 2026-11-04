import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { SupportSection } from '#/pages/settings/sections/support-section'
import { renderWithRouter } from '#/test/render-with-router'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

const SETTINGS_PATH = '/api/v1/organizations/{organization_id}/support/settings'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	field_clock_enabled: false,
	vat_on_debits: false,
	mcp_enabled: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const LABEL = 'Clés des champs personnalisés'

interface FakeApiHandlers {
	displayFields?: string[]
	settingsError?: Error
	putSettings?: (params: unknown) => unknown
}

function installFakeTanstackApi(handlers: FakeApiHandlers) {
	const calls: { method: string; path: string; params: unknown }[] = []

	const fakeApi = {
		get(path: string, params: unknown) {
			const queryKey = [
				{ _id: path, path: (params as { path?: unknown })?.path },
			]
			return {
				queryKey,
				queryOptions: {
					queryKey,
					queryFn: async () => {
						calls.push({ method: 'get', path, params })
						if (path === SETTINGS_PATH) {
							if (handlers.settingsError) throw handlers.settingsError
							return {
								data: { display_fields: handlers.displayFields ?? [] },
							}
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
						if (method === 'put' && path === SETTINGS_PATH) {
							if (!handlers.putSettings) {
								return { data: (params as { body: unknown }).body }
							}
							return handlers.putSettings(params)
						}
						throw new Error(`unmocked mutation ${method} ${path}`)
					},
				},
			}
		},
	}

	Object.assign(window, { tanstackApi: fakeApi })

	return calls
}

interface RenderOptions extends FakeApiHandlers {
	permissions?: string[]
}

async function renderSection({
	permissions = ['MANAGE_SUPPORT'],
	...handlers
}: RenderOptions = {}) {
	const calls = installFakeTanstackApi(handlers)
	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCacheForOrganization(queryClient, ORGANIZATION.id, permissions)

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

	const result = await renderWithRouter(
		<Providers>
			<SupportSection />
		</Providers>,
	)
	return { ...result, calls }
}

function putCalls<Call extends { method: string; path: string }>(
	calls: Call[],
) {
	return calls.filter(
		(call) => call.method === 'put' && call.path === SETTINGS_PATH,
	)
}

async function flush() {
	await new Promise((resolve) => setTimeout(resolve, 30))
}

describe('SupportSection', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('shows the saved list, one key per line', async () => {
		await renderSection({ displayFields: ['numero_contrat', 'chantier'] })

		const field = (await screen.findByLabelText(LABEL)) as HTMLTextAreaElement
		expect(field.value).toBe('numero_contrat\nchantier')
	})

	it('shows an empty field when no key is configured', async () => {
		await renderSection({ displayFields: [] })

		const field = (await screen.findByLabelText(LABEL)) as HTMLTextAreaElement
		expect(field.value).toBe('')
	})

	it('saves the edited list as trimmed ordered keys without the empty lines', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection({ displayFields: ['chantier'] })

		const field = await screen.findByLabelText(LABEL)
		await user.clear(field)
		await user.type(field, '  marque {enter}{enter}   {enter}chantier  {enter}')
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => {
			expect(putCalls(calls)).toHaveLength(1)
		})
		expect(putCalls(calls)[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: { display_fields: ['marque', 'chantier'] },
		})
	})

	it('saves an empty list when every line is removed', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection({ displayFields: ['chantier'] })

		await user.clear(await screen.findByLabelText(LABEL))
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => {
			expect(putCalls(calls)).toHaveLength(1)
		})
		expect(putCalls(calls)[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: { display_fields: [] },
		})
	})

	it('refuses a duplicated key, names it, and sends no request', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection({ displayFields: [] })

		const field = await screen.findByLabelText(LABEL)
		await user.type(field, 'chantier{enter}marque{enter}chantier')
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(
			screen.getByText('La clé « chantier » est listée deux fois.'),
		).toBeDefined()
		await flush()
		expect(putCalls(calls)).toHaveLength(0)
	})

	it('refuses nine keys and sends no request', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection({ displayFields: [] })

		const field = await screen.findByLabelText(LABEL)
		fireEvent.change(field, {
			target: { value: 'a\nb\nc\nd\ne\nf\ng\nh\ni' },
		})
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(screen.getByText('Huit clés au plus.')).toBeDefined()
		await flush()
		expect(putCalls(calls)).toHaveLength(0)
	})

	it('clears the form error and sends the request once the list is fixed', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection({ displayFields: [] })

		const field = await screen.findByLabelText(LABEL)
		fireEvent.change(field, { target: { value: 'chantier\nchantier' } })
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))
		expect(
			screen.getByText('La clé « chantier » est listée deux fois.'),
		).toBeDefined()

		fireEvent.change(field, { target: { value: 'chantier\nmarque' } })
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => {
			expect(putCalls(calls)).toHaveLength(1)
		})
		expect(
			screen.queryByText('La clé « chantier » est listée deux fois.'),
		).toBeNull()
	})

	it('is read-only without MANAGE_SUPPORT, with no save button', async () => {
		await renderSection({ permissions: [], displayFields: ['chantier'] })

		const field = (await screen.findByLabelText(LABEL)) as HTMLTextAreaElement
		expect(field.value).toBe('chantier')
		expect(field.readOnly).toBe(true)
		expect(screen.queryByRole('button', { name: 'Enregistrer' })).toBeNull()
		expect(
			screen.getByText(
				'Seuls les membres autorisés à traiter les tickets peuvent modifier ce réglage.',
			),
		).toBeDefined()
	})

	it('sends no request when a submit is forced without MANAGE_SUPPORT', async () => {
		const { calls } = await renderSection({
			permissions: ['VIEW_SUPPORT'],
			displayFields: ['chantier'],
		})

		const field = await screen.findByLabelText(LABEL)
		const form = field.closest('form')
		if (!form) throw new Error('form not found')
		fireEvent.submit(form)

		await flush()
		expect(putCalls(calls)).toHaveLength(0)
		expect(screen.queryByText('Huit clés au plus.')).toBeNull()
	})

	it('sends the request on the same forced submit when MANAGE_SUPPORT is held', async () => {
		const { calls } = await renderSection({ displayFields: ['chantier'] })

		const field = await screen.findByLabelText(LABEL)
		const form = field.closest('form')
		if (!form) throw new Error('form not found')
		fireEvent.submit(form)

		await waitFor(() => {
			expect(putCalls(calls)).toHaveLength(1)
		})
	})

	it('shows the API refusal of a 422 as the save error', async () => {
		const user = userEvent.setup()
		await renderSection({
			displayFields: ['chantier'],
			putSettings: () => {
				throw Object.assign(new Error('Clé de champ invalide.'), {
					status: 422,
				})
			},
		})

		await screen.findByLabelText(LABEL)
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(await screen.findByText('Clé de champ invalide.')).toBeDefined()
	})

	it('words a 403 on save as a revoked permission', async () => {
		const user = userEvent.setup()
		await renderSection({
			displayFields: ['chantier'],
			putSettings: () => {
				throw Object.assign(new Error('Forbidden'), { status: 403 })
			},
		})

		await screen.findByLabelText(LABEL)
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(
			await screen.findByText(
				"Vous n'avez plus la permission nécessaire pour cette action.",
			),
		).toBeDefined()
		expect(screen.queryByText('Forbidden')).toBeNull()
	})

	it('shows the load failure and sends nothing when the settings cannot be read', async () => {
		const { calls } = await renderSection({
			settingsError: new Error('Réglages indisponibles.'),
		})

		expect(await screen.findByText('Réglages indisponibles.')).toBeDefined()
		expect(screen.queryByRole('button', { name: 'Enregistrer' })).toBeNull()

		const form = screen.getByLabelText(LABEL).closest('form')
		if (!form) throw new Error('form not found')
		fireEvent.submit(form)
		await flush()
		expect(putCalls(calls)).toHaveLength(0)
	})
})
