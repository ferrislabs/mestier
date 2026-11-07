import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { type ReactNode, useState } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import { WorkflowPageFeature } from '#/pages/automation/feature/workflow-page-feature'
import type { EditorSession } from '#/pages/automation/feature/workflow-top-bar-feature'
import { installFakeTanstackApi } from '#/pages/automation/test/fake-tanstack-api'
import type { EditorView } from '#/pages/automation/ui/editor-top-bar'
import { renderWithRouter } from '#/test/render-with-router'

vi.mock('#/pages/automation/feature/workflow-canvas-feature', () => ({
	WorkflowCanvasFeature: ({ session }: { session: EditorSession }) => {
		return (
			<div>
				<p>éditeur {session.workflowId ?? 'brouillon'}</p>
				<p>
					{session.snapshot().graph.triggers.length} déclencheur(s) dans le
					brouillon
				</p>
				<button
					type="button"
					onClick={() =>
						session.onGraphChange({
							graph: {
								connectors: [],
								edges: [],
								triggers: [{ id: 't1', kind: 'Manual' }],
							},
							layout: new Map(),
						})
					}
				>
					Poser un déclencheur
				</button>
				<p>{session.dirty ? 'sale' : 'propre'}</p>
				<p>{session.enabled ? 'actif' : 'inactif'}</p>
				<button type="button" onClick={() => session.onEnabledChange(true)}>
					Activer
				</button>
				<button
					type="button"
					onClick={() => session.onNameChange('Relance des devis')}
				>
					Nommer
				</button>
				<button type="button" onClick={session.onSave}>
					Enregistrer
				</button>
				{session.saveError ? <p>{session.saveError}</p> : null}
			</div>
		)
	},
}))

vi.mock('#/pages/automation/feature/workflow-executions-feature', () => ({
	WorkflowExecutionsFeature: ({
		workflowId,
		executionNumber,
	}: {
		workflowId: string
		executionNumber?: number
	}) => (
		<p>
			exécutions {workflowId} {executionNumber ?? 'sans exécution'}
		</p>
	),
}))

const WORKFLOWS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows'
const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'
const VERSIONS_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions'
const MY_PERMISSIONS_PATH =
	'/api/v1/organizations/{organization_id}/members/me/permissions'

const ORGANIZATION: Organization = {
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

function workflow(id: string) {
	return {
		id,
		organization_id: 'org-1',
		name: `Workflow ${id}`,
		description: null,
		enabled: true,
		folder_id: null,
		tags: [],
		archived_at: null,
		layout: null,
		current_version: null,
		created_at: '2026-08-01T00:00:00Z',
		updated_at: '2026-08-01T00:00:00Z',
	}
}

function Providers({ children }: { children: ReactNode }) {
	const [client] = useState(
		() => new QueryClient({ defaultOptions: { queries: { retry: false } } }),
	)
	return (
		<QueryClientProvider client={client}>
			<OrganizationListProvider organizations={[ORGANIZATION]}>
				<ActiveOrganizationProvider activeOrganization={ORGANIZATION}>
					{children}
				</ActiveOrganizationProvider>
			</OrganizationListProvider>
		</QueryClientProvider>
	)
}

function installApi(unknown: readonly string[] = []) {
	const api = installFakeTanstackApi()
	api.mockGet(WORKFLOW_PATH, (params) => {
		const id = (params as { path: { workflow_id: string } }).path.workflow_id
		if (unknown.includes(id))
			throw Object.assign(new Error('Not found'), { status: 404 })
		return { data: workflow(id), pagination: null }
	})
	api.mockGet(MY_PERMISSIONS_PATH, () => ({
		data: { permissions: ['VIEW_AUTOMATION', 'MANAGE_AUTOMATION'] },
		pagination: null,
	}))
	api.mockMutation('post', WORKFLOWS_PATH, (params) => {
		const body = (params as { body: Record<string, unknown> }).body
		return {
			data: { ...workflow('workflow-new'), ...body },
			pagination: null,
		}
	})
	api.mockMutation('patch', WORKFLOW_PATH, (params) => {
		const body = (params as { body: Record<string, unknown> }).body
		return { data: { ...workflow('workflow-1'), ...body }, pagination: null }
	})
	api.mockMutation('put', VERSIONS_PATH, () => ({ data: {}, pagination: null }))
	return api
}

function Harness({
	initialId,
	initialView = 'editor',
	executionNumber,
}: {
	initialId: string
	initialView?: EditorView
	executionNumber?: number
}) {
	const [workflowId, setWorkflowId] = useState(initialId)
	const [view, setView] = useState<EditorView>(initialView)
	return (
		<>
			<button type="button" onClick={() => setView('executions')}>
				voir les exécutions
			</button>
			<button type="button" onClick={() => setView('editor')}>
				voir l’éditeur
			</button>
			<button type="button" onClick={() => setWorkflowId('workflow-2')}>
				ouvrir workflow-2
			</button>
			<WorkflowPageFeature
				workflowId={workflowId}
				view={view}
				executionNumber={executionNumber}
			/>
		</>
	)
}

async function renderPage(props: Parameters<typeof Harness>[0]) {
	await renderWithRouter(
		<Providers>
			<Harness {...props} />
		</Providers>,
	)
}

describe('WorkflowPageFeature', () => {
	it('opens the editor of a stored workflow once it has loaded', async () => {
		installApi()

		await renderPage({ initialId: 'workflow-1' })

		expect(await screen.findByText('éditeur workflow-1')).toBeTruthy()
	})

	it('renders the executions for the executions view, without a selected run', async () => {
		installApi()

		await renderPage({ initialId: 'workflow-1', initialView: 'executions' })

		expect(
			await screen.findByText('exécutions workflow-1 sans exécution'),
		).toBeTruthy()
	})

	it('hands the selected run to the executions', async () => {
		installApi()

		await renderPage({
			initialId: 'workflow-1',
			initialView: 'executions',
			executionNumber: 1285,
		})

		expect(await screen.findByText('exécutions workflow-1 1285')).toBeTruthy()
	})

	it('shows the not-found screen for an id the server does not know', async () => {
		installApi(['ghost'])

		await renderPage({ initialId: 'ghost' })

		expect(
			await screen.findByRole('heading', { name: 'Workflow introuvable' }),
		).toBeTruthy()
		expect(screen.queryByText('éditeur ghost')).toBeNull()
	})

	it('keeps the unsaved graph above the switch between the editor and the executions', async () => {
		const user = userEvent.setup()
		installApi()
		await renderPage({ initialId: 'workflow-1' })
		await user.click(
			await screen.findByRole('button', { name: 'Poser un déclencheur' }),
		)
		expect(screen.getByText('sale')).toBeTruthy()

		await user.click(
			screen.getByRole('button', { name: 'voir les exécutions' }),
		)
		expect(await screen.findByText(/^exécutions workflow-1/)).toBeTruthy()
		await user.click(screen.getByRole('button', { name: 'voir l’éditeur' }))

		expect(
			await screen.findByText('1 déclencheur(s) dans le brouillon'),
		).toBeTruthy()
		expect(screen.getByText('sale')).toBeTruthy()
	})

	it('opens a draft on a blank graph, with nothing fetched for it', async () => {
		const api = installApi()

		await renderPage({ initialId: 'nouveau' })

		expect(await screen.findByText('éditeur brouillon')).toBeTruthy()
		expect(screen.getByText('0 déclencheur(s) dans le brouillon')).toBeTruthy()
		expect(screen.getByText('propre')).toBeTruthy()
		expect(api.calls.filter((call) => call.method === 'get')).toEqual([])
	})

	it('keeps the draft above the switch too', async () => {
		const user = userEvent.setup()
		installApi()
		await renderPage({ initialId: 'nouveau' })
		await user.click(
			await screen.findByRole('button', { name: 'Poser un déclencheur' }),
		)

		await user.click(
			screen.getByRole('button', { name: 'voir les exécutions' }),
		)
		await user.click(screen.getByRole('button', { name: 'voir l’éditeur' }))

		expect(
			await screen.findByText('1 déclencheur(s) dans le brouillon'),
		).toBeTruthy()
	})

	it('drops the draft when another workflow is opened, and loads that one', async () => {
		const user = userEvent.setup()
		installApi()
		await renderPage({ initialId: 'nouveau' })
		await user.click(
			await screen.findByRole('button', { name: 'Poser un déclencheur' }),
		)

		await user.click(screen.getByRole('button', { name: 'ouvrir workflow-2' }))

		expect(await screen.findByText('éditeur workflow-2')).toBeTruthy()
		await waitFor(() => {
			expect(screen.getByText('propre')).toBeTruthy()
		})
	})
})

describe('WorkflowPageFeature — saving', () => {
	function callsOf(api: ReturnType<typeof installApi>, method: string) {
		return api.calls.filter((call) => call.method === method)
	}

	it('creates a draft with one POST that carries the name and the switch, then saves the graph, with no PATCH', async () => {
		const user = userEvent.setup()
		const api = installApi()
		await renderPage({ initialId: 'nouveau' })
		await user.click(
			await screen.findByRole('button', { name: 'Poser un déclencheur' }),
		)
		await user.click(screen.getByRole('button', { name: 'Nommer' }))
		await user.click(screen.getByRole('button', { name: 'Activer' }))

		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => expect(callsOf(api, 'put')).toHaveLength(1))
		expect(callsOf(api, 'post')).toHaveLength(1)
		expect(callsOf(api, 'post')[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: { name: 'Relance des devis', enabled: true },
		})
		expect(callsOf(api, 'put')[0]?.params).toMatchObject({
			path: { organization_id: 'org-1', workflow_id: 'workflow-new' },
			body: { graph: { triggers: [{ id: 't1', kind: 'Manual' }] } },
		})
		expect(callsOf(api, 'patch')).toEqual([])
	})

	it('creates a draft left off with enabled: false, and still no PATCH', async () => {
		const user = userEvent.setup()
		const api = installApi()
		await renderPage({ initialId: 'nouveau' })

		await user.click(await screen.findByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => expect(callsOf(api, 'put')).toHaveLength(1))
		expect(callsOf(api, 'post')[0]?.params).toEqual({
			path: { organization_id: 'org-1' },
			body: { name: 'Mon workflow', enabled: false },
		})
		expect(callsOf(api, 'patch')).toEqual([])
	})

	it('tells the user when the draft could not be created, and saves no graph', async () => {
		const user = userEvent.setup()
		const api = installApi()
		api.mockMutation('post', WORKFLOWS_PATH, () => {
			throw new Error('Nom déjà pris')
		})
		await renderPage({ initialId: 'nouveau' })

		await user.click(await screen.findByRole('button', { name: 'Enregistrer' }))

		expect(
			await screen.findByText('L’enregistrement a échoué : Nom déjà pris'),
		).toBeTruthy()
		expect(callsOf(api, 'put')).toEqual([])
		expect(callsOf(api, 'patch')).toEqual([])
	})

	it('updates a stored workflow with one PATCH and reloads its detail through the update hook', async () => {
		const user = userEvent.setup()
		const api = installApi()
		await renderPage({ initialId: 'workflow-1' })
		await user.click(await screen.findByRole('button', { name: 'Nommer' }))
		const loadedBefore = callsOf(api, 'get').filter(
			(call) => call.path === WORKFLOW_PATH,
		).length

		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		await waitFor(() => expect(callsOf(api, 'patch')).toHaveLength(1))
		expect(callsOf(api, 'patch')[0]?.params).toEqual({
			path: { organization_id: 'org-1', workflow_id: 'workflow-1' },
			body: { name: 'Relance des devis' },
		})
		await waitFor(() =>
			expect(
				callsOf(api, 'get').filter((call) => call.path === WORKFLOW_PATH)
					.length,
			).toBeGreaterThan(loadedBefore),
		)
		expect(callsOf(api, 'post')).toEqual([])
		expect(callsOf(api, 'put')).toEqual([])
	})
})
