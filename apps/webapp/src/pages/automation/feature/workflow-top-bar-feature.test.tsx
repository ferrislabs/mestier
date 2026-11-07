import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import {
	ActiveOrganizationProvider,
	OrganizationListProvider,
} from '#/hooks/use-active-organization'
import type { Organization } from '#/hooks/use-organizations'
import {
	type EditorSession,
	EditorSessionContext,
	WorkflowTopBarFeature,
} from '#/pages/automation/feature/workflow-top-bar-feature'
import { NEW_WORKFLOW_ID } from '#/pages/automation/lib/automation-paths'
import { installFakeTanstackApi } from '#/pages/automation/test/fake-tanstack-api'
import type { EditorView } from '#/pages/automation/ui/editor-top-bar'
import { renderWithRouter } from '#/test/render-with-router'

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

function sessionStub(overrides: Partial<EditorSession> = {}): EditorSession {
	return {
		workflowId: 'workflow-1',
		name: 'Relance en cours',
		enabled: false,
		dirty: true,
		isSaving: false,
		saveError: null,
		validation: { connectorErrors: new Map(), graphErrors: [] },
		snapshot: () => ({
			graph: { connectors: [], edges: [], triggers: [] },
			layout: new Map(),
		}),
		onNameChange: vi.fn(),
		onNameBlur: vi.fn(),
		onEnabledChange: vi.fn(),
		onGraphChange: vi.fn(),
		onSave: vi.fn(),
		...overrides,
	}
}

function render(
	view: EditorView,
	session?: EditorSession,
	workflowId = 'workflow-1',
	tags: string[] = [],
) {
	const api = installFakeTanstackApi()
	api.mockGet(WORKFLOW_PATH, () => ({
		data: {
			id: 'workflow-1',
			organization_id: 'org-1',
			name: 'Créer une facture Odoo',
			description: null,
			enabled: true,
			folder_id: null,
			tags,
			archived_at: null,
			layout: null,
			current_version: null,
			created_at: '2026-08-01T00:00:00Z',
			updated_at: '2026-08-01T00:00:00Z',
		},
		pagination: null,
	}))
	api.mockGet(MY_PERMISSIONS_PATH, () => ({
		data: { permissions: ['VIEW_AUTOMATION', 'MANAGE_AUTOMATION'] },
		pagination: null,
	}))

	function Providers({ children }: { children: ReactNode }) {
		return (
			<QueryClientProvider
				client={
					new QueryClient({ defaultOptions: { queries: { retry: false } } })
				}
			>
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
			<EditorSessionContext value={session ?? null}>
				<WorkflowTopBarFeature workflowId={workflowId} view={view} />
			</EditorSessionContext>
		</Providers>,
	).then((rendered) => ({ ...rendered, api }))
}

describe('WorkflowTopBarFeature — switching the view', () => {
	it('puts the executions view in the search of the workflow page', async () => {
		const user = userEvent.setup()
		const { router } = await render('editor')

		await user.click(await screen.findByRole('tab', { name: 'Exécutions' }))

		await waitFor(() =>
			expect(router.state.location.href).toBe(
				'/o/atelier-bois/automatisation/workflows/workflow-1?vue=executions',
			),
		)
	})

	it('comes back to the bare workflow page for the editor', async () => {
		const user = userEvent.setup()
		const { router } = await render('executions')

		await user.click(await screen.findByRole('tab', { name: 'Éditeur' }))

		await waitFor(() =>
			expect(router.state.location.href).toBe(
				'/o/atelier-bois/automatisation/workflows/workflow-1',
			),
		)
	})

	it('links the Workflows crumb to the workflow list', async () => {
		await render('editor')

		expect(
			(await screen.findByRole('link', { name: 'Workflows' })).getAttribute(
				'href',
			),
		).toBe('/o/atelier-bois/automatisation/workflows')
	})
})

describe('WorkflowTopBarFeature — without an editor session', () => {
	it('shows the stored name and a Save button that has nothing to save', async () => {
		await render('executions')

		const save = (await screen.findByRole('button', {
			name: 'Enregistrer',
		})) as HTMLButtonElement
		expect(save.disabled).toBe(true)
		expect(
			((await screen.findByLabelText('Nom du workflow')) as HTMLInputElement)
				.value,
		).toBe('Créer une facture Odoo')
	})
})

describe('WorkflowTopBarFeature — with an editor session', () => {
	it('shows the name, the switch and the dirty state of the session, in either view', async () => {
		const session = sessionStub()
		await render('executions', session)

		expect(
			((await screen.findByLabelText('Nom du workflow')) as HTMLInputElement)
				.value,
		).toBe('Relance en cours')
		expect(
			screen
				.getByRole('switch', { name: 'Actif' })
				.getAttribute('aria-checked'),
		).toBe('false')
		expect(screen.getByText('Modifications non enregistrées')).toBeDefined()
		expect(
			(screen.getByRole('button', { name: 'Enregistrer' }) as HTMLButtonElement)
				.disabled,
		).toBe(false)
	})

	it('hands the keystrokes, the switch and the save click to the session', async () => {
		const user = userEvent.setup()
		const session = sessionStub()
		await render('editor', session)

		await user.type(await screen.findByLabelText('Nom du workflow'), 'x')
		await user.tab()
		await user.click(screen.getByRole('switch', { name: 'Actif' }))
		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(session.onNameChange).toHaveBeenCalledWith('Relance en coursx')
		expect(session.onNameBlur).toHaveBeenCalled()
		expect(session.onEnabledChange).toHaveBeenCalledWith(true)
		expect(session.onSave).toHaveBeenCalled()
	})

	it('reads « Enregistré » once the session has nothing left to save', async () => {
		await render('editor', sessionStub({ dirty: false }))

		expect(await screen.findByText('Enregistré')).toBeDefined()
	})
})

describe('WorkflowTopBarFeature — the tags', () => {
	function tagsOfBar(): string[] {
		return within(screen.getByRole('list', { name: 'Tags du workflow' }))
			.queryAllByRole('listitem')
			.map((item) => item.textContent ?? '')
	}

	it('shows the tags the API sends for a stored workflow, without an editor session', async () => {
		await render('executions', undefined, 'workflow-1', ['Support'])

		await screen.findByDisplayValue('Créer une facture Odoo')
		expect(tagsOfBar()).toEqual(['Support'])
	})

	it('shows the tags of the session workflow, in the order of the API', async () => {
		await render('editor', sessionStub(), 'workflow-1', ['Ventes', 'Clients'])

		await waitFor(() => expect(tagsOfBar()).toEqual(['Ventes', 'Clients']))
	})

	it('shows no tag on a workflow that has none', async () => {
		const { api } = await render('editor', sessionStub())

		await waitFor(() =>
			expect(api.calls.some((call) => call.path === WORKFLOW_PATH)).toBe(true),
		)
		expect(tagsOfBar()).toEqual([])
	})

	it('shows no tag, and fetches nothing, on a draft whose session has no workflow yet', async () => {
		const { api } = await render(
			'editor',
			sessionStub({ workflowId: null }),
			NEW_WORKFLOW_ID,
			['Support'],
		)

		await screen.findByLabelText('Nom du workflow')
		expect(tagsOfBar()).toEqual([])
		expect(api.calls.some((call) => call.path === WORKFLOW_PATH)).toBe(false)
	})

	it('shows no tag, and fetches nothing, on a draft without an editor session', async () => {
		const { api } = await render('editor', undefined, NEW_WORKFLOW_ID, [
			'Support',
		])

		await screen.findByLabelText('Nom du workflow')
		expect(tagsOfBar()).toEqual([])
		expect(api.calls.some((call) => call.path === WORKFLOW_PATH)).toBe(false)
	})
})
