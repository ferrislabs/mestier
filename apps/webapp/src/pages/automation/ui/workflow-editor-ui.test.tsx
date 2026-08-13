import { fireEvent, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import type {
	ConnectorPanelData,
	WorkflowEditorUIProps,
} from '#/pages/automation/ui/workflow-editor-ui'
import { WorkflowEditorUI } from '#/pages/automation/ui/workflow-editor-ui'
import { renderWithRouter } from '#/test/render-with-router'

// jsdom has no `ResizeObserver` at all (React Flow observes every node with
// one, to react to size changes after the initial render). A true no-op is
// enough: each node carries `initialWidth`/`initialHeight`
// (`workflow-editor-ui.tsx`) precisely so React Flow never needs a real
// measurement to consider it visible — a stub that actually invoked the
// resize callback would reach React Flow's own `new
// DOMMatrixReadOnly(style.transform)`, which jsdom doesn't implement
// either. Scoped to this file — see `field-form.test.tsx`'s identical
// comment for why this is not in `vitest.setup.ts`.
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
		rootConnectorIds: [],
		nodes: [],
		edges: [],
		positions: new Map(),
		onConnect: vi.fn(),
		onAddConnectorFrom: vi.fn(),
		onDuplicateConnector: vi.fn(),
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
							family: 'http',
							hasError: false,
							previewLines: [],
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

describe('WorkflowEditorUI — start node', () => {
	it('always renders a Départ node, even on an empty graph', async () => {
		await renderWithRouter(<WorkflowEditorUI {...baseProps()} />)

		expect(screen.getByText('Départ')).toBeDefined()
	})

	it('right-clicking it does nothing — it is not a real connector', async () => {
		const onDuplicateConnector = vi.fn()
		await renderWithRouter(
			<WorkflowEditorUI {...baseProps({ onDuplicateConnector })} />,
		)

		fireEvent.contextMenu(screen.getByText('Départ'))

		expect(screen.queryByText('Dupliquer')).toBeNull()
	})
})

describe('WorkflowEditorUI — node preview lines', () => {
	it("shows the connector's own config preview on its card", async () => {
		await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					nodes: [
						{
							id: 'c1',
							label: 'Requête HTTP',
							kindLabel: 'http.request',
							family: 'http',
							hasError: false,
							previewLines: ['URL: https://api.example.com'],
						},
					],
					positions: new Map([['c1', { x: 0, y: 0 }]]),
				})}
			/>,
		)

		expect(screen.getByText('URL: https://api.example.com')).toBeDefined()
	})
})

describe('WorkflowEditorUI — adding a connector from an existing node', () => {
	it('the "+" button opens the search list and wires the new connector', async () => {
		const user = userEvent.setup()
		const onAddConnectorFrom = vi.fn()
		await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					onAddConnectorFrom,
					nodes: [
						{
							id: 'c1',
							label: 'Requête HTTP',
							kindLabel: 'http.request',
							family: 'http',
							hasError: false,
							previewLines: [],
						},
					],
					positions: new Map([['c1', { x: 0, y: 0 }]]),
				})}
			/>,
		)

		// Marked `nodrag` (`workflow-editor-ui.tsx`) precisely so React Flow's
		// own node-drag handling never reaches it — safe for a normal
		// `userEvent.click`, unlike a click landing on the bare node itself.
		await user.click(screen.getByTitle('Ajouter un connecteur après celui-ci'))
		// Scoped to the search list's own container: the palette shows the
		// same catalogue and would otherwise match "Requête HTTP" too.
		const searchList = screen.getByTestId('connector-search-list')
		await user.click(within(searchList).getByText('Requête HTTP'))

		expect(onAddConnectorFrom).toHaveBeenCalledWith('c1', 'http.request')
	})
})

describe('WorkflowEditorUI — node "..." menu', () => {
	function singleNodeProps(overrides: Partial<WorkflowEditorUIProps> = {}) {
		return baseProps({
			nodes: [
				{
					id: 'c1',
					label: 'Requête HTTP',
					kindLabel: 'http.request',
					family: 'http',
					hasError: false,
					previewLines: [],
				},
			],
			positions: new Map([['c1', { x: 0, y: 0 }]]),
			...overrides,
		})
	}

	it('duplicates the connector', async () => {
		const user = userEvent.setup()
		const onDuplicateConnector = vi.fn()
		await renderWithRouter(
			<WorkflowEditorUI {...singleNodeProps({ onDuplicateConnector })} />,
		)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByText('Dupliquer'))

		expect(onDuplicateConnector).toHaveBeenCalledWith('c1')
	})

	it('removes the connector', async () => {
		const user = userEvent.setup()
		const onRemoveConnector = vi.fn()
		await renderWithRouter(
			<WorkflowEditorUI {...singleNodeProps({ onRemoveConnector })} />,
		)

		await user.click(screen.getByRole('button', { name: 'Actions' }))
		await user.click(screen.getByText('Supprimer'))

		expect(onRemoveConnector).toHaveBeenCalledWith('c1')
	})
})

describe('WorkflowEditorUI — right-click context menus', () => {
	it('on a node: duplicate and delete, same actions as the "..." menu', async () => {
		const user = userEvent.setup()
		const onRemoveConnector = vi.fn()
		await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					onRemoveConnector,
					nodes: [
						{
							id: 'c1',
							label: 'Mon connecteur',
							kindLabel: 'http.request',
							family: 'http',
							hasError: false,
							previewLines: [],
						},
					],
					positions: new Map([['c1', { x: 0, y: 0 }]]),
				})}
			/>,
		)

		fireEvent.contextMenu(screen.getByText('Mon connecteur'))
		await user.click(screen.getByText('Supprimer'))

		expect(onRemoveConnector).toHaveBeenCalledWith('c1')
	})

	it('on an edge: quick branch pick and delete', async () => {
		const user = userEvent.setup()
		const onBranchChange = vi.fn()
		const { container } = await renderWithRouter(
			<WorkflowEditorUI
				{...baseProps({
					onBranchChange,
					nodes: [
						{
							id: 'a',
							label: 'A',
							kindLabel: 'a.kind',
							family: 'a',
							hasError: false,
							previewLines: [],
						},
						{
							id: 'b',
							label: 'B',
							kindLabel: 'b.kind',
							family: 'b',
							hasError: false,
							previewLines: [],
						},
					],
					edges: [{ from: 'a', to: 'b', branch: null }],
					positions: new Map([
						['a', { x: 0, y: 0 }],
						['b', { x: 300, y: 0 }],
					]),
				})}
			/>,
		)

		const edge = container.querySelector('.react-flow__edge')
		expect(edge).not.toBeNull()
		if (!edge) return
		fireEvent.contextMenu(edge)
		await user.click(screen.getByText('Alors'))

		expect(onBranchChange).toHaveBeenCalledWith('a', 'b', 'Then')
	})

	it('on the empty pane: adds a connector via the search list', async () => {
		const user = userEvent.setup()
		const onAddConnector = vi.fn()
		const { container } = await renderWithRouter(
			<WorkflowEditorUI {...baseProps({ onAddConnector })} />,
		)

		const pane = container.querySelector('.react-flow__pane')
		expect(pane).not.toBeNull()
		if (!pane) return
		fireEvent.contextMenu(pane)
		// Scoped for the same reason as the "+" button test above: the
		// palette shows the same catalogue and would otherwise be an
		// equally valid, ambiguous match.
		const searchList = screen.getByTestId('connector-search-list')
		await user.click(within(searchList).getByText('Requête HTTP'))

		expect(onAddConnector).toHaveBeenCalledWith('http.request')
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

// The config popover only ever renders for the node whose id matches
// `selection.data.connectorId` (`workflow-editor-ui.tsx`'s `useCanvasState`)
// — every test below needs a matching `nodes` entry for that popover to
// exist at all, not just a `selection` prop on its own.
function connectorPanelProps(
	panelData: ConnectorPanelData,
	overrides: Partial<WorkflowEditorUIProps> = {},
) {
	return baseProps({
		nodes: [
			{
				id: panelData.connectorId,
				label: panelData.label,
				kindLabel: 'http.request',
				family: 'http',
				hasError: false,
				previewLines: [],
			},
		],
		positions: new Map([[panelData.connectorId, { x: 0, y: 0 }]]),
		selection: { type: 'connector', data: panelData },
		...overrides,
	})
}

describe('WorkflowEditorUI — connector panel (popover)', () => {
	it("renders the selected connector's fields and forwards edits", async () => {
		const user = userEvent.setup()
		const onFieldChange = vi.fn()
		await renderWithRouter(
			<WorkflowEditorUI
				{...connectorPanelProps(
					{
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
					{ onFieldChange },
				)}
			/>,
		)

		await user.type(screen.getByLabelText('URL'), 'x')

		expect(onFieldChange).toHaveBeenCalledWith('c1', 'url', 'x')
	})

	it('offers a credential picker only when the connector requires auth', async () => {
		await renderWithRouter(
			<WorkflowEditorUI
				{...connectorPanelProps({
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
				})}
			/>,
		)

		expect(screen.getByText('Identification')).toBeDefined()
	})

	it('shows a connector-level error message', async () => {
		await renderWithRouter(
			<WorkflowEditorUI
				{...connectorPanelProps({
					connectorId: 'c1',
					label: 'Requête HTTP',
					fields: [],
					values: {},
					fieldErrors: {},
					globalError: 'Ce connecteur ne peut pas être le premier du graphe',
					authRequired: false,
					credentialOptions: [],
					credentialId: null,
					signingCredentials: [],
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
				{...connectorPanelProps(
					{
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
					{ onRemoveConnector },
				)}
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
				{...connectorPanelProps(
					{
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
					{
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
					},
				)}
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
