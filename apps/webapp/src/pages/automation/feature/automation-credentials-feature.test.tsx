import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { AutomationCredentialsFeature } from '#/pages/automation/feature/automation-credentials-feature'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

/**
 * jsdom has no pointer-capture APIs or `scrollIntoView`. Radix's `Select`
 * calls both when an item is actually *selected* — opening the popover
 * alone never reaches this path, which is why no other test in this repo
 * needed it before this file, the first to click a `SelectItem` rather
 * than just asserting it exists.
 *
 * Deliberately scoped to this file rather than `vitest.setup.ts`: filling
 * the gap globally changes real (if accidental) behavior every other test
 * that opens a Radix `Select`/`Popover` was already running against —
 * tried once, reverted after it broke 24 unrelated tests across 9 files.
 */
for (const method of [
	'hasPointerCapture',
	'setPointerCapture',
	'releasePointerCapture',
	'scrollIntoView',
] as const) {
	if (typeof Element.prototype[method] !== 'function') {
		Element.prototype[method] = (() => false) as never
	}
}

const CONNECTORS_PATH =
	'/api/v1/organizations/{organization_id}/automation/connectors'
const CREDENTIALS_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials'
const CREDENTIAL_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}'
const CREDENTIAL_ROTATE_PATH =
	'/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/rotate'

const ORGANIZATION: Organization = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
	field_clock_enabled: false,
	vat_on_debits: false,
	created_at: '2026-01-01T00:00:00Z',
	updated_at: '2026-01-01T00:00:00Z',
}

const AUTH_SCHEMES = [
	{
		kind: 'bearer_token',
		label: 'Bearer token',
		fields: [
			{
				name: 'token',
				label: 'Token',
				required: true,
				kind: 'Text',
				expression: false,
				secret: true,
			},
		],
	},
]

function credential(overrides: Record<string, unknown> = {}) {
	return {
		id: 'cred-1',
		organization_id: 'org-1',
		name: 'Signature sortante',
		kind: 'bearer_token',
		origin: 'generated',
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}

type Handler = (params: unknown) => unknown

interface FakeApi {
	calls: { method: string; path: string; params: unknown }[]
	mockGet: (path: string, handler: Handler) => void
	mockMutation: (method: string, path: string, handler: Handler) => void
}

function installFakeTanstackApi(): FakeApi {
	const calls: { method: string; path: string; params: unknown }[] = []
	const getHandlers = new Map<string, Handler>()
	const mutationHandlers = new Map<string, Handler>()

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
						const handler = getHandlers.get(path)
						if (!handler) throw new Error(`unmocked GET ${path}`)
						return handler(params)
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
						const handler = mutationHandlers.get(`${method}:${path}`)
						if (!handler) {
							throw new Error(`unmocked ${method.toUpperCase()} ${path}`)
						}
						return handler(params)
					},
				},
			}
		},
	}

	// biome-ignore lint/suspicious/noExplicitAny: test-only fake, shape matches TanstackQueryApiClient's used surface
	;(window as any).tanstackApi = fakeApi

	return {
		calls,
		mockGet: (path, handler) => getHandlers.set(path, handler),
		mockMutation: (method, path, handler) =>
			mutationHandlers.set(`${method}:${path}`, handler),
	}
}

function renderFeature(
	configure: (api: FakeApi) => void,
	permissions: string[] = ['VIEW_AUTOMATION', 'MANAGE_AUTOMATION'],
) {
	const api = installFakeTanstackApi()
	api.mockGet(CONNECTORS_PATH, () => ({
		data: { connectors: [], auth_schemes: AUTH_SCHEMES },
	}))
	api.mockGet(CREDENTIALS_PATH, () => ({ data: [] }))
	configure(api)

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

	render(
		<Providers>
			<AutomationCredentialsFeature />
		</Providers>,
	)

	return api
}

describe('AutomationCredentialsFeature — creation', () => {
	it('reveals a generated secret exactly once after creation', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockMutation('post', CREDENTIALS_PATH, () => ({
				data: credential({ secret: 'c2VjcmV0LWJ5dGVz' }),
			}))
		})

		await user.click(await screen.findByRole('button', { name: 'Ajouter' }))
		await user.type(screen.getByLabelText('Nom'), 'Signature sortante')
		await user.click(screen.getByRole('combobox', { name: 'Origine' }))
		await user.click(
			screen.getByRole('option', { name: /Générée par Mestier/ }),
		)
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(await screen.findByText('Secret généré')).toBeDefined()
		expect(screen.getByDisplayValue('c2VjcmV0LWJ5dGVz')).toBeDefined()
	})

	it('never shows a secret reveal for a supplied credential', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockMutation('post', CREDENTIALS_PATH, (params) => ({
				data: credential({
					origin: 'supplied',
					secret: (params as { body: { data: unknown } }).body.data,
				}),
			}))
		})

		await user.click(await screen.findByRole('button', { name: 'Ajouter' }))
		await user.type(screen.getByLabelText('Nom'), 'Odoo')
		await user.type(screen.getByLabelText('Token'), 'shh')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		await waitFor(() => {
			expect(screen.queryByText('Secret généré')).toBeNull()
		})
	})

	it('requires every scheme field before submitting a supplied credential', async () => {
		const user = userEvent.setup()
		const api = renderFeature(() => {})

		await user.click(await screen.findByRole('button', { name: 'Ajouter' }))
		await user.type(screen.getByLabelText('Nom'), 'Sans token')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(screen.getByText('Token est requis')).toBeDefined()
		expect(
			api.calls.some((c) => c.method === 'post' && c.path === CREDENTIALS_PATH),
		).toBe(false)
	})
})

describe('AutomationCredentialsFeature — rotation', () => {
	it('reveals the freshly rotated secret exactly once', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({ data: [credential()] }))
			api.mockMutation('post', CREDENTIAL_ROTATE_PATH, () => ({
				data: credential({ secret: 'cm90YXRlZC1zZWNyZXQ' }),
			}))
		})

		await user.click(await screen.findByRole('button', { name: 'Régénérer' }))

		expect(await screen.findByText('Secret généré')).toBeDefined()
		expect(screen.getByDisplayValue('cm90YXRlZC1zZWNyZXQ')).toBeDefined()
	})

	it('offers no rotation on a supplied credential', async () => {
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({
				data: [credential({ origin: 'supplied' })],
			}))
		})

		await screen.findByText('Signature sortante')
		expect(screen.queryByRole('button', { name: 'Régénérer' })).toBeNull()
	})
})

describe('AutomationCredentialsFeature — deletion refused', () => {
	it('names the workflows the backend reports when a referenced credential cannot be deleted', async () => {
		const user = userEvent.setup()
		renderFeature((api) => {
			api.mockGet(CREDENTIALS_PATH, () => ({ data: [credential()] }))
			api.mockMutation('delete', CREDENTIAL_PATH, () => {
				throw new Error(
					'credential is still referenced by workflow(s): Créer une facture Odoo',
				)
			})
		})

		await user.click(await screen.findByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: 'Supprimer' }))

		expect(
			await screen.findByText(
				'credential is still referenced by workflow(s): Créer une facture Odoo',
			),
		).toBeDefined()
	})
})

describe('AutomationCredentialsFeature — permission gating', () => {
	it('hides every mutation without MANAGE_AUTOMATION', async () => {
		renderFeature(
			(api) => {
				api.mockGet(CREDENTIALS_PATH, () => ({ data: [credential()] }))
			},
			['VIEW_AUTOMATION'],
		)

		await screen.findByText('Signature sortante')
		expect(screen.queryByRole('button', { name: 'Ajouter' })).toBeNull()
		expect(screen.queryByRole('button', { name: 'Régénérer' })).toBeNull()
		expect(screen.queryByRole('button', { name: 'Actions' })).toBeNull()
	})
})
