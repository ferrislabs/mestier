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
							meta: {},
						})
					}
				>
					Poser un déclencheur
				</button>
				<p>{session.dirty ? 'sale' : 'propre'}</p>
			</div>
		)
	},
}))

vi.mock('#/pages/automation/feature/workflow-executions-feature', () => ({
	WorkflowExecutionsFeature: ({
		workflowId,
		runId,
	}: {
		workflowId: string
		runId?: string
	}) => (
		<p>
			exécutions {workflowId} {runId ?? 'sans exécution'}
		</p>
	),
}))

const WORKFLOW_PATH =
	'/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}'
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
	return api
}

function Harness({
	initialId,
	initialView = 'editor',
	executionId,
}: {
	initialId: string
	initialView?: EditorView
	executionId?: string
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
				executionId={executionId}
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
			executionId: 'run-1',
		})

		expect(await screen.findByText('exécutions workflow-1 run-1')).toBeTruthy()
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
