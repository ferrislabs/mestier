import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { AutomationWorkflowsFeature } from '#/pages/automation/feature/automation-workflows-feature'
import { renderWithRouter } from '#/test/render-with-router'
import { seedPermissionsCacheForOrganization } from '#/test/with-permissions'

const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'
const RUNS_PATH = '/api/v1/organizations/{organization_id}/automation/runs'

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

function workflow(overrides: Record<string, unknown> = {}) {
	return {
		id: 'workflow-1',
		organization_id: 'org-1',
		name: 'Créer une facture Odoo',
		description: null,
		enabled: true,
		current_version_id: null,
		trigger_mode: 'events',
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}

function run(overrides: Record<string, unknown> = {}) {
	return {
		id: 'run-1',
		organization_id: 'org-1',
		workflow_id: 'workflow-1',
		workflow_version_id: 'v1',
		status: 'succeeded',
		created_at: '2026-08-01T00:00:00Z',
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

function renderFeature(configure: (api: FakeApi) => void) {
	const api = installFakeTanstackApi()
	api.mockGet(WORKFLOWS_PATH, () => ({ data: [workflow()], pagination: null }))
	api.mockGet(RUNS_PATH, () => ({ data: [], pagination: null }))
	configure(api)

	const queryClient = new QueryClient({
		defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
	})
	seedPermissionsCacheForOrganization(queryClient, ORGANIZATION.id, [
		'VIEW_AUTOMATION',
		'MANAGE_AUTOMATION',
	])

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

	return renderWithRouter(
		<Providers>
			<AutomationWorkflowsFeature />
		</Providers>,
	).then(() => api)
}

describe('AutomationWorkflowsFeature — last run', () => {
	it('shows the most recent run for a workflow that ran several times', async () => {
		await renderFeature((api) => {
			api.mockGet(RUNS_PATH, () => ({
				data: [
					run({
						id: 'run-1',
						status: 'failed',
						created_at: '2026-08-01T00:00:00Z',
					}),
					run({
						id: 'run-2',
						status: 'succeeded',
						created_at: '2026-08-03T00:00:00Z',
					}),
				],
				pagination: null,
			}))
		})

		expect(await screen.findByText('Réussi')).toBeDefined()
		expect(screen.queryByText('Échoué')).toBeNull()
	})

	it('says plainly when a workflow has never run', async () => {
		await renderFeature((api) => {
			api.mockGet(RUNS_PATH, () => ({ data: [], pagination: null }))
		})

		expect(await screen.findByText('Jamais exécuté')).toBeDefined()
	})
})

describe('AutomationWorkflowsFeature — trigger mode', () => {
	it('shows a manual workflow as such', async () => {
		await renderFeature((api) => {
			api.mockGet(WORKFLOWS_PATH, () => ({
				data: [workflow({ trigger_mode: 'manual' })],
				pagination: null,
			}))
		})

		expect(await screen.findByText('Manuel')).toBeDefined()
	})
})

describe('AutomationWorkflowsFeature — create', () => {
	it('creates a workflow through the dialog and calls the create mutation', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			fakeApi.mockMutation('post', WORKFLOWS_PATH, (params) => ({
				data: workflow({
					id: 'workflow-2',
					...(params as { body: Record<string, unknown> }).body,
				}),
			}))
		})

		await user.click(
			await screen.findByRole('button', { name: 'Nouveau workflow' }),
		)
		await user.type(screen.getByLabelText('Nom'), 'Notifier le client')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'post' && c.path === WORKFLOWS_PATH,
			)
			expect(call?.params).toMatchObject({
				path: { organization_id: 'org-1' },
				body: { name: 'Notifier le client', description: null },
			})
		})
	})
})

describe('AutomationWorkflowsFeature — rename', () => {
	it('opens the dialog pre-filled and calls the update mutation with the new name', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			fakeApi.mockMutation('patch', WORKFLOW_PATH, (params) => ({
				data: workflow({
					...(params as { body: Record<string, unknown> }).body,
				}),
			}))
		})

		await user.click(await screen.findByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: 'Renommer' }))

		expect(screen.getByDisplayValue('Créer une facture Odoo')).toBeDefined()

		const nameField = screen.getByLabelText('Nom')
		await user.clear(nameField)
		await user.type(nameField, 'Facture Odoo renommée')
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'patch' && c.path === WORKFLOW_PATH,
			)
			expect(call?.params).toMatchObject({
				path: { organization_id: 'org-1', workflow_id: 'workflow-1' },
				body: { name: 'Facture Odoo renommée' },
			})
		})
	})
})

describe('AutomationWorkflowsFeature — enable/disable', () => {
	it('calls the update mutation with the flipped enabled flag', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			fakeApi.mockMutation('patch', WORKFLOW_PATH, (params) => ({
				data: workflow({
					...(params as { body: Record<string, unknown> }).body,
				}),
			}))
		})

		await user.click(await screen.findByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: 'Désactiver' }))

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'patch' && c.path === WORKFLOW_PATH,
			)
			expect(call?.params).toMatchObject({
				path: { organization_id: 'org-1', workflow_id: 'workflow-1' },
				body: { enabled: false },
			})
		})
	})
})

describe('AutomationWorkflowsFeature — delete', () => {
	it('calls the delete mutation once the confirmation is accepted', async () => {
		const user = userEvent.setup()
		const api = await renderFeature((fakeApi) => {
			fakeApi.mockMutation('delete', WORKFLOW_PATH, () => ({ data: undefined }))
		})

		await user.click(await screen.findByRole('button', { name: 'Actions' }))
		await user.click(screen.getByRole('menuitem', { name: 'Supprimer' }))
		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		await waitFor(() => {
			const call = api.calls.find(
				(c) => c.method === 'delete' && c.path === WORKFLOW_PATH,
			)
			expect(call?.params).toMatchObject({
				path: { organization_id: 'org-1', workflow_id: 'workflow-1' },
			})
		})
	})
})

describe('AutomationWorkflowsFeature — reaching the editor', () => {
	it('builds the editor link from the active organization, not from nothing', async () => {
		await renderFeature(() => {})

		const link = await screen.findByRole('link', {
			name: /Créer une facture Odoo/,
		})

		expect(link.getAttribute('href')).toBe(
			'/o/atelier-bois/automatisation/workflow-1',
		)
	})
})
