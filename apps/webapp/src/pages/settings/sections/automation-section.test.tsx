import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import { AutomationSection } from '#/pages/settings/sections/automation-section'
import { renderWithRouter } from '#/test/render-with-router'

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
const SETTINGS_PATH =
	'/api/v1/organizations/{organization_id}/automation/settings'
const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
const RUNS_PATH = '/api/v1/organizations/{organization_id}/automation/runs'
const RUN_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}'
const RUN_REPLAY_PATH =
	'/api/v1/organizations/{organization_id}/automation/runs/{run_id}/replay'

const ORGANIZATION = {
	id: 'org-1',
	name: 'Atelier Bois & Co',
	slug: 'atelier-bois',
	owner_id: 'user-1',
	missing_legal_identity_fields: [],
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

const SETTINGS = {
	event_retention_seconds: 7_776_000,
	succeeded_run_retention_seconds: 2_592_000,
	retry_schedule_seconds: [5, 30],
	disable_target_after: 20,
}

const WORKFLOW = {
	id: 'workflow-1',
	organization_id: 'org-1',
	name: 'Créer une facture Odoo',
	description: null,
	enabled: true,
	current_version_id: 'v1',
	created_at: '2026-08-01T00:00:00Z',
	updated_at: '2026-08-01T00:00:00Z',
}

interface FakeApiHandlers {
	credentials?: unknown[]
	runs?: unknown[]
	postCredential?: (params: unknown) => unknown
	putSettings?: (params: unknown) => unknown
	getRun?: (params: unknown) => unknown
	replayRun?: (params: unknown) => unknown
}

function installFakeTanstackApi(handlers: FakeApiHandlers = {}) {
	const calls: { method: string; path: string; params: unknown }[] = []
	const credentials = handlers.credentials ?? []
	const runs = handlers.runs ?? []

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
						if (path === CONNECTORS_PATH) {
							return { data: { connectors: [], auth_schemes: AUTH_SCHEMES } }
						}
						if (path === CREDENTIALS_PATH) return { data: credentials }
						if (path === SETTINGS_PATH) return { data: SETTINGS }
						if (path === WORKFLOWS_PATH) return { data: [WORKFLOW] }
						if (path === RUNS_PATH) return { data: runs }
						if (path === RUN_PATH) {
							if (!handlers.getRun) throw new Error('getRun not mocked')
							return handlers.getRun(params)
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
						if (method === 'post' && path === CREDENTIALS_PATH) {
							if (!handlers.postCredential) {
								throw new Error('postCredential not mocked')
							}
							return handlers.postCredential(params)
						}
						if (method === 'put' && path === SETTINGS_PATH) {
							if (!handlers.putSettings)
								throw new Error('putSettings not mocked')
							return handlers.putSettings(params)
						}
						if (method === 'post' && path === RUN_REPLAY_PATH) {
							if (!handlers.replayRun) throw new Error('replayRun not mocked')
							return handlers.replayRun(params)
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

async function renderSection(handlers: FakeApiHandlers = {}) {
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

	const result = await renderWithRouter(
		<Providers>
			<AutomationSection />
		</Providers>,
	)
	return { ...result, calls }
}

describe('AutomationSection — credentials', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('reveals a generated secret exactly once after creation', async () => {
		const user = userEvent.setup()
		await renderSection({
			postCredential: () => ({
				data: {
					id: 'cred-1',
					name: 'Signature sortante',
					kind: 'bearer_token',
					origin: 'generated',
					organization_id: 'org-1',
					created_at: '2026-08-01T00:00:00Z',
					updated_at: '2026-08-01T00:00:00Z',
					secret: 'c2VjcmV0LWJ5dGVz',
				},
			}),
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
		await renderSection({
			postCredential: (params) => ({
				data: {
					id: 'cred-2',
					name: 'Odoo',
					kind: 'bearer_token',
					origin: 'supplied',
					organization_id: 'org-1',
					created_at: '2026-08-01T00:00:00Z',
					updated_at: '2026-08-01T00:00:00Z',
					secret: (params as { body: { data: unknown } }).body.data,
				},
			}),
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
		const { calls } = await renderSection({})

		await user.click(await screen.findByRole('button', { name: 'Ajouter' }))
		await user.type(screen.getByLabelText('Nom'), 'Sans token')
		await user.click(screen.getByRole('button', { name: 'Créer' }))

		expect(screen.getByText('Token est requis')).toBeDefined()
		expect(
			calls.some((c) => c.method === 'post' && c.path === CREDENTIALS_PATH),
		).toBe(false)
	})
})

describe('AutomationSection — settings', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('loads the existing settings into the form', async () => {
		await renderSection({})

		expect(await screen.findByDisplayValue('7776000')).toBeDefined()
		expect(screen.getByDisplayValue('5, 30')).toBeDefined()
	})

	it('refuses an invalid retry schedule without calling the API', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection({})

		const field = await screen.findByLabelText(
			'Plan de nouvelles tentatives (secondes, séparées par des virgules)',
		)
		await user.clear(field)
		await user.type(field, '5, abc')
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(
			screen.getByText(/liste de secondes séparées par des virgules/),
		).toBeDefined()
		expect(
			calls.some((c) => c.method === 'put' && c.path === SETTINGS_PATH),
		).toBe(false)
	})

	it('submits the parsed body on a valid change', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection({
			putSettings: (params) => ({ data: (params as { body: unknown }).body }),
		})

		const field = await screen.findByLabelText('Rétention des événements')
		await user.clear(field)
		await user.type(field, '864000')
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => {
			const call = calls.find(
				(c) => c.method === 'put' && c.path === SETTINGS_PATH,
			)
			expect(call?.params).toMatchObject({
				path: { organization_id: 'org-1' },
				body: { event_retention_seconds: 864_000 },
			})
		})
	})
})

describe('AutomationSection — run log', () => {
	afterEach(() => {
		vi.restoreAllMocks()
	})

	it('resolves the workflow name and shows the run status', async () => {
		await renderSection({
			runs: [
				{
					id: 'run-1',
					organization_id: 'org-1',
					workflow_id: 'workflow-1',
					workflow_version_id: 'v1',
					status: 'failed',
					error: 'HTTP 500',
					created_at: '2026-08-01T00:00:00Z',
				},
			],
		})

		expect(await screen.findByText('Créer une facture Odoo')).toBeDefined()
		expect(screen.getByText('Échoué')).toBeDefined()
		expect(screen.getByText('HTTP 500')).toBeDefined()
	})

	it('offers to replay a failed step from a settled run, and calls replay with its connector id', async () => {
		const user = userEvent.setup()
		const { calls } = await renderSection({
			runs: [
				{
					id: 'run-1',
					organization_id: 'org-1',
					workflow_id: 'workflow-1',
					workflow_version_id: 'v1',
					status: 'failed',
					created_at: '2026-08-01T00:00:00Z',
				},
			],
			getRun: () => ({
				data: {
					id: 'run-1',
					organization_id: 'org-1',
					workflow_id: 'workflow-1',
					workflow_version_id: 'v1',
					status: 'failed',
					created_at: '2026-08-01T00:00:00Z',
					steps: [
						{
							id: 'step-1',
							connector_id: 'send-request',
							status: 'failed',
							attempts: 3,
							error: 'timeout',
							created_at: '2026-08-01T00:00:00Z',
							iteration_path: '',
						},
					],
				},
			}),
			replayRun: (params) => ({
				data: { id: 'run-1', status: 'pending', ...(params as object) },
			}),
		})

		await user.click(await screen.findByRole('button', { name: 'Détails' }))
		expect(await screen.findByText('send-request')).toBeDefined()

		await user.click(
			screen.getByRole('button', { name: /Relancer depuis ici/ }),
		)

		await waitFor(() => {
			const call = calls.find(
				(c) => c.method === 'post' && c.path === RUN_REPLAY_PATH,
			)
			expect(call?.params).toMatchObject({
				path: { organization_id: 'org-1', run_id: 'run-1' },
				body: { connector_id: 'send-request' },
			})
		})
	})

	it('hides the replay action for a run still pending or running', async () => {
		const user = userEvent.setup()
		await renderSection({
			runs: [
				{
					id: 'run-2',
					organization_id: 'org-1',
					workflow_id: 'workflow-1',
					workflow_version_id: 'v1',
					status: 'running',
					created_at: '2026-08-01T00:00:00Z',
				},
			],
			getRun: () => ({
				data: {
					id: 'run-2',
					organization_id: 'org-1',
					workflow_id: 'workflow-1',
					workflow_version_id: 'v1',
					status: 'running',
					created_at: '2026-08-01T00:00:00Z',
					steps: [
						{
							id: 'step-1',
							connector_id: 'send-request',
							status: 'running',
							attempts: 1,
							created_at: '2026-08-01T00:00:00Z',
							iteration_path: '',
						},
					],
				},
			}),
		})

		await user.click(await screen.findByRole('button', { name: 'Détails' }))
		expect(await screen.findByText('send-request')).toBeDefined()

		expect(
			screen.queryByRole('button', { name: /Relancer depuis ici/ }),
		).toBeNull()
	})
})
