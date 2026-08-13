import { fireEvent, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type { WorkflowEditorUIProps } from '#/pages/automation/ui/workflow-editor-ui'
import { WorkflowEditorUI } from '#/pages/automation/ui/workflow-editor-ui'
import { renderWithRouter } from '#/test/render-with-router'

// jsdom implements neither `ResizeObserver` (React Flow measures its
// viewport with it) nor pointer-capture/`scrollIntoView` (Radix's `Select`,
// used by the branch picker). Scoped to this file — see `field-form.test.tsx`'s
// identical comment for why this is not in `vitest.setup.ts`.
class ResizeObserverStub {
	observe() {}
	unobserve() {}
	disconnect() {}
}
;(globalThis as { ResizeObserver?: unknown }).ResizeObserver ??=
	ResizeObserverStub
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

function baseProps(
	overrides: Partial<WorkflowEditorUIProps> = {},
): WorkflowEditorUIProps {
	return {
		organizationSlug: 'atelier-bois',
		workflowId: 'wf-1',
		workflowName: 'Relance client',
		enabled: true,
		isLoading: false,
		error: null,
		saveError: null,
		isSaving: false,
		onSave: vi.fn(),
		families: [
			{
				family: 'http',
				connectors: [{ kind: 'http.request', label: 'Requête HTTP' }],
			},
		],
		onAddConnector: vi.fn(),
		nodes: [],
		edges: [],
		positions: new Map(),
		onConnect: vi.fn(),
		onSelectConnector: vi.fn(),
		onSelectEdge: vi.fn(),
		onDeselect: vi.fn(),
		selection: null,
		onFieldChange: vi.fn(),
		onCredentialChange: vi.fn(),
		onRemoveConnector: vi.fn(),
		onCreateCredential: vi.fn(),
		onBranchChange: vi.fn(),
		onRemoveEdge: vi.fn(),
		expressionTargetField: null,
		upstreamForExpression: [],
		onInsertExpression: vi.fn(),
		onInsertExpressionValue: vi.fn(),
		onCloseExpressionPicker: vi.fn(),
		...overrides,
	}
}

describe('WorkflowEditorUI — canvas selection', () => {
	// `userEvent.click` fires a real `mousedown`, which reaches
	// `@xyflow/react`'s d3-drag node handler — that handler reads
	// `event.view.document`, and jsdom's synthetic `MouseEvent` has no
	// `view`, so it throws. `fireEvent.click` dispatches only the `click`
	// React Flow's own selection logic listens to, without the drag
	// machinery in between — the same distinction the codebase already
	// draws for Radix internals in other test files, just a different
	// third-party quirk.
	it('clicking a node selects it', () => {
		const onSelectConnector = vi.fn()
		return renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					nodes: [
						{
							id: 'c1',
							label: 'Ma requête',
							kindLabel: 'http.request',
							hasError: false,
						},
					],
					positions: new Map([['c1', { x: 0, y: 0 }]]),
					onSelectConnector,
				})}
			/>,
		).then(() => {
			fireEvent.click(screen.getByText('Ma requête'))
			expect(onSelectConnector).toHaveBeenCalledWith('c1')
		})
	})
})

describe('WorkflowEditorUI — loading', () => {
	it('shows a loading state instead of the canvas', async () => {
		await renderWithRouter(
			<WorkflowEditorUI {...baseProps({ isLoading: true })} />,
		)

		expect(screen.getByText('Chargement…')).toBeDefined()
	})
})

describe('WorkflowEditorUI — palette', () => {
	it('groups connectors by family and adds one on click', async () => {
		const user = userEvent.setup()
		const onAddConnector = vi.fn()
		await renderWithRouter(
			<WorkflowEditorUI {...baseProps({ onAddConnector })} />,
		)

		expect(screen.getByText('http')).toBeDefined()
		await user.click(screen.getByText('Requête HTTP'))

		expect(onAddConnector).toHaveBeenCalledWith('http.request')
	})
})

describe('WorkflowEditorUI — errors', () => {
	it('shows a load error and a save error as two distinct banners', async () => {
		await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					error: 'Workflow introuvable',
					saveError: 'La sauvegarde a échoué',
				})}
			/>,
		)

		expect(screen.getByText('Workflow introuvable')).toBeDefined()
		expect(screen.getByText('La sauvegarde a échoué')).toBeDefined()
	})
})

describe('WorkflowEditorUI — save', () => {
	it('calls onSave when clicked', async () => {
		const user = userEvent.setup()
		const onSave = vi.fn()
		await renderWithRouter(<WorkflowEditorUI {...baseProps({ onSave })} />)

		await user.click(screen.getByRole('button', { name: 'Enregistrer' }))

		expect(onSave).toHaveBeenCalled()
	})

	it('disables the button while a save is in flight', async () => {
		await renderWithRouter(
			<WorkflowEditorUI {...baseProps({ isSaving: true })} />,
		)

		const button = screen.getByRole('button', {
			name: /Enregistrer/,
		}) as HTMLButtonElement
		expect(button.disabled).toBe(true)
	})
})

describe('WorkflowEditorUI — connector panel', () => {
	it("renders the selected connector's fields and forwards edits", async () => {
		const user = userEvent.setup()
		const onFieldChange = vi.fn()
		await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					onFieldChange,
					selection: {
						type: 'connector',
						data: {
							connectorId: 'c1',
							label: 'Requête HTTP',
							fields: [
								{
									name: 'url',
									label: 'URL',
									kind: 'Text',
									required: true,
									expression: false,
									secret: false,
								},
							],
							values: {},
							fieldErrors: {},
							globalError: null,
							authRequired: false,
							credentialOptions: [],
							credentialId: null,
							signingCredentials: [],
						},
					},
				})}
			/>,
		)

		await user.type(screen.getByLabelText('URL'), 'x')

		expect(onFieldChange).toHaveBeenCalledWith('c1', 'url', 'x')
	})

	it('offers a credential picker only when the connector requires auth', async () => {
		await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					selection: {
						type: 'connector',
						data: {
							connectorId: 'c1',
							label: 'Lecture Odoo',
							fields: [],
							values: {},
							fieldErrors: {},
							globalError: null,
							authRequired: true,
							credentialOptions: [{ id: 'cred-1', name: 'Odoo prod' }],
							credentialId: null,
							signingCredentials: [],
						},
					},
				})}
			/>,
		)

		expect(screen.getByText('Identification')).toBeDefined()
	})

	it('shows a connector-level error message', async () => {
		await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					selection: {
						type: 'connector',
						data: {
							connectorId: 'c1',
							label: 'Requête HTTP',
							fields: [],
							values: {},
							fieldErrors: {},
							globalError:
								'Ce connecteur ne peut pas être le premier du graphe',
							authRequired: false,
							credentialOptions: [],
							credentialId: null,
							signingCredentials: [],
						},
					},
				})}
			/>,
		)

		expect(
			screen.getByText('Ce connecteur ne peut pas être le premier du graphe'),
		).toBeDefined()
	})

	it('removing the connector calls onRemoveConnector with its id', async () => {
		const user = userEvent.setup()
		const onRemoveConnector = vi.fn()
		await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					onRemoveConnector,
					selection: {
						type: 'connector',
						data: {
							connectorId: 'c1',
							label: 'Requête HTTP',
							fields: [],
							values: {},
							fieldErrors: {},
							globalError: null,
							authRequired: false,
							credentialOptions: [],
							credentialId: null,
							signingCredentials: [],
						},
					},
				})}
			/>,
		)

		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		expect(onRemoveConnector).toHaveBeenCalledWith('c1')
	})
})

describe('WorkflowEditorUI — expression insertion', () => {
	it('shows the picker for the field last clicked and closes it once used', async () => {
		const user = userEvent.setup()
		const onInsertExpressionValue = vi.fn()
		const onCloseExpressionPicker = vi.fn()
		await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					expressionTargetField: 'predicate',
					upstreamForExpression: [
						{
							id: 'c0',
							label: 'Requête HTTP',
							paths: [{ path: '', preview: '{…}' }],
						},
					],
					onInsertExpressionValue,
					onCloseExpressionPicker,
					selection: {
						type: 'connector',
						data: {
							connectorId: 'c1',
							label: 'Condition',
							fields: [
								{
									name: 'predicate',
									label: 'Predicate',
									kind: 'Text',
									required: true,
									expression: true,
									secret: false,
								},
							],
							values: {},
							fieldErrors: {},
							globalError: null,
							authRequired: false,
							credentialOptions: [],
							credentialId: null,
							signingCredentials: [],
						},
					},
				})}
			/>,
		)

		expect(screen.getByText('Insérer dans « Predicate »')).toBeDefined()

		await user.click(screen.getByText('output'))
		expect(onInsertExpressionValue).toHaveBeenCalledWith(
			'{{ connectors.c0.output }}',
		)

		await user.click(screen.getByRole('button', { name: 'Fermer' }))
		expect(onCloseExpressionPicker).toHaveBeenCalled()
	})
})

describe('WorkflowEditorUI — edge panel', () => {
	it("renders the connection's endpoints and current branch", async () => {
		await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					selection: {
						type: 'edge',
						data: {
							from: 'c1',
							to: 'c2',
							fromLabel: 'Condition',
							toLabel: 'Envoyer un email',
							branch: 'Then',
						},
					},
				})}
			/>,
		)

		expect(screen.getByText('Condition → Envoyer un email')).toBeDefined()
	})

	it('changing the branch reports the new value', async () => {
		const user = userEvent.setup()
		const onBranchChange = vi.fn()
		await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					onBranchChange,
					selection: {
						type: 'edge',
						data: {
							from: 'c1',
							to: 'c2',
							fromLabel: 'Condition',
							toLabel: 'Envoyer un email',
							branch: null,
						},
					},
				})}
			/>,
		)

		await user.click(screen.getByRole('combobox'))
		await user.click(screen.getByRole('option', { name: 'Sinon' }))

		expect(onBranchChange).toHaveBeenCalledWith('c1', 'c2', 'Else')
	})

	it('removing the connection calls onRemoveEdge with both endpoints', async () => {
		const user = userEvent.setup()
		const onRemoveEdge = vi.fn()
		await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					onRemoveEdge,
					selection: {
						type: 'edge',
						data: {
							from: 'c1',
							to: 'c2',
							fromLabel: 'Condition',
							toLabel: 'Envoyer un email',
							branch: null,
						},
					},
				})}
			/>,
		)

		await user.click(screen.getByRole('button', { name: 'Supprimer' }))

		expect(onRemoveEdge).toHaveBeenCalledWith('c1', 'c2')
	})
})

describe('WorkflowEditorUI — no selection', () => {
	it('hints to select something instead of showing a panel', async () => {
		await renderWithRouter(<WorkflowEditorUI {...baseProps()} />)

		expect(
			screen.getByText(/Sélectionnez un connecteur ou une connexion/),
		).toBeDefined()
	})
})
