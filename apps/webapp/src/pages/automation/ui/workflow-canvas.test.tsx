import {
	fireEvent,
	render,
	screen,
	waitFor,
	within,
} from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useRef, useState } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import type { NodePosition } from '#/pages/automation/lib/graph'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'
import {
	dragNodeBy,
	flowPositionOf,
	installFlowTestEnvironment,
} from '#/pages/automation/test/flow-test-env'
import {
	isBranchDeclared,
	type LastRunData,
	WorkflowCanvas,
	type WorkflowCanvasProps,
} from '#/pages/automation/ui/workflow-canvas'

beforeEach(() => {
	installFlowTestEnvironment()
})

function connector(
	id: string,
	kind: string,
	config: Record<string, unknown> = {},
): Schemas.PlacedConnectorDto {
	return { id, kind, version: 1, config }
}

function trigger(
	id: string,
	kind: Schemas.TriggerKindDto = 'Manual',
): Schemas.PlacedTriggerDto {
	return { id, kind }
}

function descriptor(
	kind: string,
	label: string,
	branches: Schemas.BranchDto[] = [],
	fields: Schemas.FieldResponse[] = [],
): Schemas.ConnectorDescriptorResponse {
	return {
		auth: 'None',
		branches,
		family: 'test',
		fields,
		kind,
		label,
		output_example: null,
		version: 1,
	}
}

function event(
	name: string,
	payloadExample: unknown = { id: 1 },
): Schemas.EventDescriptorResponse {
	return {
		name,
		label: name,
		subject_kind: name.split('.')[0] ?? name,
		version: 1,
		payload_example: payloadExample,
	}
}

function textField(name: string, label: string): Schemas.FieldResponse {
	return {
		expression: false,
		kind: 'Text',
		label,
		name,
		required: false,
		secret: false,
		visible_when: null,
	}
}

const SIMPLE_KIND = 'test.simple'
const BRANCHING_KIND = 'test.branching'
const OTHER_KIND = 'test.other'

const SIMPLE_DESCRIPTOR = descriptor(SIMPLE_KIND, 'Étape simple')
const BRANCHING_DESCRIPTOR = descriptor(
	BRANCHING_KIND,
	'Étape à embranchements',
	['Then', 'Else'],
)
const OTHER_DESCRIPTOR = descriptor(OTHER_KIND, 'Autre étape')

function descriptorMap(
	...entries: Schemas.ConnectorDescriptorResponse[]
): Map<string, Schemas.ConnectorDescriptorResponse> {
	return new Map(entries.map((entry) => [entry.kind, entry]))
}

function Harness({
	graph,
	layout,
	descriptors,
	connectorErrors = new Map(),
	runStatuses = new Map(),
	events = [],
	lastRun = null,
	onEvaluateExpression = () => Promise.resolve(null),
	credentials = [],
	authSchemes = [],
	onCreateCredential = () => Promise.reject(new Error('unused in this test')),
	onSaveSpy,
}: {
	graph: Schemas.GraphDto
	layout: Map<string, NodePosition>
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	connectorErrors?: Map<string, ConnectorValidationError[]>
	runStatuses?: Map<string, string>
	events?: Schemas.EventDescriptorResponse[]
	lastRun?: LastRunData | null
	onEvaluateExpression?: WorkflowCanvasProps['onEvaluateExpression']
	credentials?: Schemas.CredentialResponse[]
	authSchemes?: Schemas.AuthSchemeResponse[]
	onCreateCredential?: WorkflowCanvasProps['onCreateCredential']
	onSaveSpy: (
		graph: Schemas.GraphDto,
		layout: Map<string, NodePosition>,
	) => void
}) {
	const [isDirty, setDirty] = useState(false)
	const [isSaving, setSaving] = useState(false)
	const current = useRef({ graph, layout })

	const props: WorkflowCanvasProps = {
		graph,
		layout,
		descriptors,
		connectorErrors,
		runStatuses,
		events,
		lastRun,
		onEvaluateExpression,
		credentials,
		authSchemes,
		onCreateCredential,
		isDirty,
		isSaving,
		onChange: (nextGraph, nextLayout) => {
			current.current = { graph: nextGraph, layout: nextLayout }
			setDirty(true)
		},
		onSave: () => {
			setSaving(true)
			onSaveSpy(current.current.graph, current.current.layout)
			setDirty(false)
			setSaving(false)
		},
	}

	return <WorkflowCanvas {...props} />
}

function renderHarness(
	overrides: Partial<Parameters<typeof Harness>[0]> & {
		graph: Schemas.GraphDto
		layout: Map<string, NodePosition>
		descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	},
) {
	const onSaveSpy = vi.fn()
	render(<Harness {...overrides} onSaveSpy={onSaveSpy} />)
	return { onSaveSpy }
}

async function clickSave() {
	fireEvent.click(await screen.findByRole('button', { name: /Enregistrer/ }))
}

describe('WorkflowCanvas — load and save round trip', () => {
	it('saves a graph with branches back unchanged', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				connector('c1', BRANCHING_KIND),
				connector('c2', SIMPLE_KIND),
				connector('c3', SIMPLE_KIND),
			],
			edges: [
				{ from: 'c1', to: 'c2', branch: 'Then' },
				{ from: 'c1', to: 'c3', branch: 'Else' },
			],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([
			['c1', { x: 0, y: 0 }],
			['c2', { x: 280, y: 0 }],
			['c3', { x: 280, y: 140 }],
		])

		const { onSaveSpy } = renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR, BRANCHING_DESCRIPTOR),
		})

		await clickSave()

		expect(onSaveSpy).toHaveBeenCalledTimes(1)
		const [savedGraph, savedLayout] = onSaveSpy.mock.calls[0] as [
			Schemas.GraphDto,
			Map<string, NodePosition>,
		]
		expect(savedGraph).toEqual(graph)
		expect([...savedLayout]).toEqual([...layout])
	})
})

describe('WorkflowCanvas — dragging', () => {
	it('comes back where it was left after a node is moved and saved', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([['c1', { x: 0, y: 0 }]])

		const { onSaveSpy } = renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const node = await screen.findByTestId('rf__node-c1')
		dragNodeBy(node, 60, 45)

		await waitFor(() => {
			expect(node.style.transform).toContain('60px')
		})

		await clickSave()

		const [, savedLayout] = onSaveSpy.mock.calls[0] as [
			Schemas.GraphDto,
			Map<string, NodePosition>,
		]
		expect(savedLayout.get('c1')).toEqual({ x: 60, y: 45 })
	})
})

describe('WorkflowCanvas — branches drive handles', () => {
	it('shows one handle per declared branch, and a single handle when unbranched', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				connector('c1', BRANCHING_KIND),
				connector('c2', SIMPLE_KIND),
			],
			edges: [],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([
			['c1', { x: 0, y: 0 }],
			['c2', { x: 280, y: 0 }],
		])

		renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR, BRANCHING_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const branchingNode = await screen.findByTestId('rf__node-c1')
		expect(branchingNode.querySelector('[data-handleid="Then"]')).not.toBeNull()
		expect(branchingNode.querySelector('[data-handleid="Else"]')).not.toBeNull()
		expect(
			branchingNode.querySelectorAll('.react-flow__handle.source').length,
		).toBe(2)

		const simpleNode = await screen.findByTestId('rf__node-c2')
		const simpleSourceHandles = simpleNode.querySelectorAll(
			'.react-flow__handle.source',
		)
		expect(simpleSourceHandles.length).toBe(1)
		expect(simpleSourceHandles[0]?.getAttribute('data-handleid')).toBeNull()
	})
})

describe('isBranchDeclared', () => {
	it('accepts wiring a trigger to a connector by hand', () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [trigger('t1')],
		}

		expect(
			isBranchDeclared(
				{ source: 't1', sourceHandle: null, target: 'c1', targetHandle: null },
				graph,
				descriptorMap(SIMPLE_DESCRIPTOR),
			),
		).toBe(true)
	})

	it('accepts a plain connection between two branchless connectors', () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND), connector('c2', SIMPLE_KIND)],
			edges: [],
			triggers: [],
		}

		expect(
			isBranchDeclared(
				{ source: 'c1', sourceHandle: null, target: 'c2', targetHandle: null },
				graph,
				descriptorMap(SIMPLE_DESCRIPTOR),
			),
		).toBe(true)
	})

	it('accepts a connection whose branch the source descriptor declares', () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				connector('c1', BRANCHING_KIND),
				connector('c2', SIMPLE_KIND),
			],
			edges: [],
			triggers: [],
		}
		const descriptors = descriptorMap(SIMPLE_DESCRIPTOR, BRANCHING_DESCRIPTOR)

		expect(
			isBranchDeclared(
				{
					source: 'c1',
					sourceHandle: 'Then',
					target: 'c2',
					targetHandle: null,
				},
				graph,
				descriptors,
			),
		).toBe(true)
	})

	it('refuses a connection whose branch the source descriptor does not declare', () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				connector('c1', BRANCHING_KIND),
				connector('c2', SIMPLE_KIND),
			],
			edges: [],
			triggers: [],
		}
		const descriptors = descriptorMap(SIMPLE_DESCRIPTOR, BRANCHING_DESCRIPTOR)

		expect(
			isBranchDeclared(
				{
					source: 'c1',
					sourceHandle: 'Otherwise',
					target: 'c2',
					targetHandle: null,
				},
				graph,
				descriptors,
			),
		).toBe(false)
	})

	it('requires no branch on the connection when the source is unbranched', () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND), connector('c2', SIMPLE_KIND)],
			edges: [],
			triggers: [],
		}
		const descriptors = descriptorMap(SIMPLE_DESCRIPTOR)

		expect(
			isBranchDeclared(
				{
					source: 'c1',
					sourceHandle: 'Then',
					target: 'c2',
					targetHandle: null,
				},
				graph,
				descriptors,
			),
		).toBe(false)
		expect(
			isBranchDeclared(
				{ source: 'c1', sourceHandle: null, target: 'c2', targetHandle: null },
				graph,
				descriptors,
			),
		).toBe(true)
	})

	it('refuses a branched edge leaving a trigger', () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [trigger('t1')],
		}

		expect(
			isBranchDeclared(
				{
					source: 't1',
					sourceHandle: 'Then',
					target: 'c1',
					targetHandle: null,
				},
				graph,
				descriptorMap(SIMPLE_DESCRIPTOR),
			),
		).toBe(false)
	})
})

describe('WorkflowCanvas — a manual and an event trigger coexist', () => {
	it('reads a manual trigger and an event trigger differently on the canvas', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [
				trigger('t1', 'Manual'),
				trigger('t2', { Events: ['quote.accepted'] }),
			],
		}
		const layout = new Map<string, NodePosition>([
			['t1', { x: -220, y: 0 }],
			['t2', { x: -220, y: 140 }],
			['c1', { x: 0, y: 0 }],
		])

		renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const manualNode = await screen.findByTestId('rf__node-t1')
		const eventNode = await screen.findByTestId('rf__node-t2')

		expect(within(manualNode).getByText('Déclenchement manuel')).toBeDefined()
		expect(within(eventNode).getByText('quote.accepted')).toBeDefined()
		expect(within(eventNode).queryByText('Déclenchement manuel')).toBeNull()
	})

	it('shows the count once an event trigger carries more than one event', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [],
			edges: [],
			triggers: [trigger('t1', { Events: ['quote.accepted', 'invoice.paid'] })],
		}

		renderHarness({
			graph,
			layout: new Map([['t1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		expect(await screen.findByText('2 événements')).toBeDefined()
	})

	it('warns on an event trigger with nothing selected, never on a manual one', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [],
			edges: [],
			triggers: [trigger('t1', 'Manual'), trigger('t2', { Events: [] })],
		}

		renderHarness({
			graph,
			layout: new Map([
				['t1', { x: 0, y: 0 }],
				['t2', { x: 0, y: 140 }],
			]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const manualNode = await screen.findByTestId('rf__node-t1')
		const eventNode = await screen.findByTestId('rf__node-t2')

		expect(
			within(eventNode).getByText('Aucun événement configuré'),
		).toBeDefined()
		expect(
			within(manualNode).queryByText('Aucun événement configuré'),
		).toBeNull()
	})
})

describe('WorkflowCanvas — adding a second trigger', () => {
	it('adds a trigger from the pane menu, configures it, and wires a connector from its own +', async () => {
		const user = userEvent.setup()
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [trigger('t1', 'Manual')],
		}
		const layout = new Map<string, NodePosition>([
			['t1', { x: -220, y: 0 }],
			['c1', { x: 0, y: 0 }],
		])

		const { onSaveSpy } = renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			events: [event('quote.accepted')],
		})

		await screen.findByTestId('rf__node-t1')
		const pane = document.querySelector('.react-flow__pane') as HTMLElement
		fireEvent.contextMenu(pane, { clientX: 400, clientY: 400 })
		fireEvent.click(await screen.findByText('Ajouter un déclencheur'))
		fireEvent.click(await screen.findByText('Sur événement(s)'))

		const newTriggerNode = await screen.findByTestId('rf__node-t2')
		expect(
			within(newTriggerNode).getByText('Aucun événement configuré'),
		).toBeDefined()

		const panel = await screen.findByTestId('trigger-config-panel')
		await user.click(
			within(panel).getByRole('checkbox', { name: 'quote.accepted' }),
		)

		await waitFor(() => {
			expect(within(newTriggerNode).getByText('quote.accepted')).toBeDefined()
		})

		fireEvent.click(
			within(newTriggerNode).getByRole('button', {
				name: 'Ajouter un connecteur après le déclencheur t2',
			}),
		)
		fireEvent.click(await screen.findByRole('button', { name: 'Étape simple' }))

		await screen.findByTestId('rf__node-c2')
		await clickSave()

		const [savedGraph] = onSaveSpy.mock.calls[0] as [Schemas.GraphDto]
		expect(savedGraph.triggers).toEqual([
			{ id: 't1', kind: 'Manual' },
			{ id: 't2', kind: { Events: ['quote.accepted'] } },
		])
		expect(savedGraph.connectors.map((c) => c.id)).toEqual(['c1', 'c2'])
		expect(savedGraph.edges).toEqual([{ from: 't2', to: 'c2', branch: null }])
	})

	it('places the new trigger at the point that was right-clicked', async () => {
		renderHarness({
			graph: { connectors: [], edges: [], triggers: [] },
			layout: new Map(),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const pane = document.querySelector('.react-flow__pane') as HTMLElement
		fireEvent.contextMenu(pane, { clientX: 500, clientY: 300 })
		const expectedPosition = flowPositionOf(500, 300)
		fireEvent.click(await screen.findByText('Ajouter un déclencheur'))
		fireEvent.click(await screen.findByText('Manuel'))

		await screen.findByTestId('rf__node-t1')
		const node = screen.getByTestId('rf__node-t1')
		expect(node.style.transform).toContain(`${expectedPosition.x}px`)
	})
})

describe('WorkflowCanvas — deleting a trigger', () => {
	it('removes the trigger and every edge it fed, immediately, with no confirmation dialog', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND), connector('c2', SIMPLE_KIND)],
			edges: [
				{ from: 't1', to: 'c1', branch: null },
				{ from: 't2', to: 'c2', branch: null },
			],
			triggers: [trigger('t1'), trigger('t2', { Events: ['quote.accepted'] })],
		}
		const layout = new Map<string, NodePosition>([
			['t1', { x: -220, y: 0 }],
			['t2', { x: -220, y: 140 }],
			['c1', { x: 0, y: 0 }],
			['c2', { x: 0, y: 140 }],
		])

		const { onSaveSpy } = renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const triggerNode = await screen.findByTestId('rf__node-t1')
		fireEvent.click(
			within(triggerNode).getByRole('button', {
				name: 'Supprimer le déclencheur t1',
			}),
		)

		expect(screen.queryByRole('alertdialog')).toBeNull()
		await waitFor(() => {
			expect(screen.queryByTestId('rf__node-t1')).toBeNull()
		})

		await clickSave()

		const [savedGraph] = onSaveSpy.mock.calls[0] as [Schemas.GraphDto]
		expect(savedGraph.triggers).toEqual([
			{ id: 't2', kind: { Events: ['quote.accepted'] } },
		])
		expect(savedGraph.edges).toEqual([{ from: 't2', to: 'c2', branch: null }])
	})

	it('closes the trigger panel once the open trigger is deleted', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [],
			edges: [],
			triggers: [trigger('t1')],
		}

		renderHarness({
			graph,
			layout: new Map([['t1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const triggerNode = await screen.findByTestId('rf__node-t1')
		fireEvent.click(triggerNode)
		await screen.findByTestId('trigger-config-panel')

		fireEvent.click(
			within(triggerNode).getByRole('button', {
				name: 'Supprimer le déclencheur t1',
			}),
		)

		await waitFor(() => {
			expect(screen.queryByTestId('trigger-config-panel')).toBeNull()
		})
	})
})

describe('WorkflowCanvas — the trigger config panel', () => {
	it('opens on a trigger click, scoped to the trigger clicked', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [],
			edges: [],
			triggers: [
				trigger('t1', { Events: ['quote.accepted'] }),
				trigger('t2', { Events: ['invoice.paid'] }),
			],
		}

		renderHarness({
			graph,
			layout: new Map([
				['t1', { x: 0, y: 0 }],
				['t2', { x: 0, y: 140 }],
			]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			events: [event('quote.accepted'), event('invoice.paid')],
			onSaveSpy: vi.fn(),
		})

		fireEvent.click(await screen.findByTestId('rf__node-t2'))

		const panel = await screen.findByTestId('trigger-config-panel')
		expect(
			within(panel).getByText('Déclencheur sur événement(s)'),
		).toBeDefined()
		expect(
			within(panel)
				.getByRole('checkbox', { name: 'invoice.paid' })
				.getAttribute('aria-checked'),
		).toBe('true')
		expect(
			within(panel)
				.getByRole('checkbox', { name: 'quote.accepted' })
				.getAttribute('aria-checked'),
		).toBe('false')
	})

	it('propagates a kind change straight through onChange, with no save button of its own', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [],
			edges: [],
			triggers: [trigger('t1', { Events: ['quote.accepted'] })],
		}

		const { onSaveSpy } = renderHarness({
			graph,
			layout: new Map([['t1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			events: [event('quote.accepted'), event('invoice.paid')],
		})

		fireEvent.click(await screen.findByTestId('rf__node-t1'))
		const panel = await screen.findByTestId('trigger-config-panel')
		expect(
			within(panel).queryByRole('button', { name: 'Enregistrer' }),
		).toBeNull()

		fireEvent.click(screen.getByRole('checkbox', { name: 'invoice.paid' }))

		await clickSave()

		const [savedGraph] = onSaveSpy.mock.calls[0] as [Schemas.GraphDto]
		expect(savedGraph.triggers).toEqual([
			{ id: 't1', kind: { Events: ['quote.accepted', 'invoice.paid'] } },
		])
	})

	it('closes the connector panel when the trigger node opens, and vice versa', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [trigger('t1')],
		}

		renderHarness({
			graph,
			layout: new Map([
				['t1', { x: -220, y: 0 }],
				['c1', { x: 0, y: 0 }],
			]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			events: [event('quote.accepted')],
			onSaveSpy: vi.fn(),
		})

		const connectorNode = await screen.findByTestId('rf__node-c1')
		fireEvent.click(connectorNode)
		expect(screen.getByTestId('connector-config-panel')).toBeDefined()

		fireEvent.click(await screen.findByTestId('rf__node-t1'))
		expect(screen.queryByTestId('connector-config-panel')).toBeNull()
		expect(await screen.findByTestId('trigger-config-panel')).toBeDefined()
	})

	it('closes on request', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [],
			edges: [],
			triggers: [trigger('t1')],
		}

		renderHarness({
			graph,
			layout: new Map([['t1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		fireEvent.click(await screen.findByTestId('rf__node-t1'))
		await screen.findByTestId('trigger-config-panel')

		fireEvent.click(screen.getByRole('button', { name: 'Fermer' }))

		expect(screen.queryByTestId('trigger-config-panel')).toBeNull()
	})
})

describe('WorkflowCanvas — validation badges', () => {
	it('badges the connector an error names', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [],
		}

		renderHarness({
			graph,
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			connectorErrors: new Map([
				['c1', [{ field: 'url', message: 'URL manquante' }]],
			]),
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		expect(within(node).getByLabelText(/Erreur/)).toBeDefined()
	})

	it('does not badge a connector with no error', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [],
			},
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			connectorErrors: new Map(),
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		expect(within(node).queryByLabelText(/Erreur/)).toBeNull()
	})

	it('badges a trigger wired to nothing, exactly as a connector error badges its connector', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [],
			edges: [],
			triggers: [trigger('t1', { Events: ['quote.accepted'] })],
		}

		renderHarness({
			graph,
			layout: new Map([['t1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			connectorErrors: new Map([
				[
					't1',
					[{ field: null, message: 'Ce déclencheur ne mène nulle part.' }],
				],
			]),
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-t1')
		expect(within(node).getByLabelText(/Erreur/)).toBeDefined()
	})

	it('shows the message naming the trigger inside its own config panel', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [],
			edges: [],
			triggers: [trigger('t1', { Events: ['quote.accepted'] })],
		}

		renderHarness({
			graph,
			layout: new Map([['t1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			events: [event('quote.accepted')],
			connectorErrors: new Map([
				[
					't1',
					[{ field: null, message: 'Ce déclencheur ne mène nulle part.' }],
				],
			]),
			onSaveSpy: vi.fn(),
		})

		fireEvent.click(await screen.findByTestId('rf__node-t1'))

		expect(
			await screen.findByText('Ce déclencheur ne mène nulle part.'),
		).toBeDefined()
	})
})

describe('WorkflowCanvas — dirty indicator', () => {
	it('is absent before any change and appears once the graph changes', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([['c1', { x: 0, y: 0 }]])

		renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		expect(screen.queryByText(/non enregistr/i)).toBeNull()

		const node = await screen.findByTestId('rf__node-c1')
		dragNodeBy(node, 30, 30)

		await waitFor(() => {
			expect(screen.getByText(/non enregistr/i)).toBeDefined()
		})
	})

	it('clears once the change is saved', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([['c1', { x: 0, y: 0 }]])

		renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		dragNodeBy(node, 30, 30)

		await waitFor(() => {
			expect(screen.getByText(/non enregistr/i)).toBeDefined()
		})

		await clickSave()

		await waitFor(() => {
			expect(screen.queryByText(/non enregistr/i)).toBeNull()
		})
	})
})

describe('WorkflowCanvas — adding a node from a handle', () => {
	it('wires the new node on the branch whose + was pressed', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', BRANCHING_KIND)],
			edges: [],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([['c1', { x: 0, y: 0 }]])

		const { onSaveSpy } = renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR, BRANCHING_DESCRIPTOR),
		})

		const branchingNode = await screen.findByTestId('rf__node-c1')
		fireEvent.click(
			within(branchingNode).getByRole('button', {
				name: 'Ajouter un connecteur sur la branche Then',
			}),
		)
		fireEvent.click(await screen.findByText('Étape simple'))

		await screen.findByTestId('rf__node-c2')
		await clickSave()

		expect(onSaveSpy).toHaveBeenCalledTimes(1)
		const [savedGraph] = onSaveSpy.mock.calls[0] as [Schemas.GraphDto]
		expect(savedGraph.connectors).toEqual([
			connector('c1', BRANCHING_KIND),
			connector('c2', SIMPLE_KIND),
		])
		expect(savedGraph.edges).toEqual([{ from: 'c1', to: 'c2', branch: 'Then' }])
	})

	it('wires on else when the else handle is the one pressed', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', BRANCHING_KIND)],
			edges: [],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([['c1', { x: 0, y: 0 }]])

		const { onSaveSpy } = renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR, BRANCHING_DESCRIPTOR),
		})

		const branchingNode = await screen.findByTestId('rf__node-c1')
		fireEvent.click(
			within(branchingNode).getByRole('button', {
				name: 'Ajouter un connecteur sur la branche Else',
			}),
		)
		fireEvent.click(await screen.findByText('Étape simple'))

		await screen.findByTestId('rf__node-c2')
		await clickSave()

		const [savedGraph] = onSaveSpy.mock.calls[0] as [Schemas.GraphDto]
		expect(savedGraph.edges).toEqual([{ from: 'c1', to: 'c2', branch: 'Else' }])
	})

	it("wires the first connector of a workflow from its real trigger's +", async () => {
		const graph: Schemas.GraphDto = {
			connectors: [],
			edges: [],
			triggers: [trigger('t1')],
		}

		const { onSaveSpy } = renderHarness({
			graph,
			layout: new Map([['t1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const triggerNode = await screen.findByTestId('rf__node-t1')
		fireEvent.click(
			within(triggerNode).getByRole('button', {
				name: 'Ajouter un connecteur après le déclencheur t1',
			}),
		)
		fireEvent.click(await screen.findByText('Étape simple'))

		await screen.findByTestId('rf__node-c1')
		expect(
			document.querySelector('[data-testid^="rf__edge-t1->c1"]'),
		).not.toBeNull()

		await clickSave()

		const [savedGraph] = onSaveSpy.mock.calls[0] as [Schemas.GraphDto]
		expect(savedGraph).toEqual({
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [{ from: 't1', to: 'c1', branch: null }],
			triggers: [{ id: 't1', kind: 'Manual' }],
		})
	})
})

describe('WorkflowCanvas — deleting a node', () => {
	it('removes the node and its edges once confirmed', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND), connector('c2', SIMPLE_KIND)],
			edges: [{ from: 'c1', to: 'c2', branch: null }],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([
			['c1', { x: 0, y: 0 }],
			['c2', { x: 280, y: 0 }],
		])

		const { onSaveSpy } = renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const leafNode = await screen.findByTestId('rf__node-c2')
		fireEvent.click(
			within(leafNode).getByRole('button', { name: 'Supprimer Étape simple' }),
		)

		await screen.findByRole('alertdialog')
		fireEvent.click(screen.getByRole('button', { name: 'Supprimer' }))

		await waitFor(() => {
			expect(screen.queryByTestId('rf__node-c2')).toBeNull()
		})

		await clickSave()

		const [savedGraph] = onSaveSpy.mock.calls[0] as [Schemas.GraphDto]
		expect(savedGraph).toEqual({
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [],
		})
	})

	it('names the connectors whose expressions reference the one being deleted', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				connector('c1', SIMPLE_KIND),
				connector('c2', OTHER_KIND, {
					url: '{{ connectors.c1.output.href }}',
				}),
			],
			edges: [],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([
			['c1', { x: 0, y: 0 }],
			['c2', { x: 280, y: 0 }],
		])

		renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR, OTHER_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const sourceNode = await screen.findByTestId('rf__node-c1')
		fireEvent.click(
			within(sourceNode).getByRole('button', {
				name: 'Supprimer Étape simple',
			}),
		)

		const dialog = await screen.findByRole('alertdialog')
		expect(within(dialog).getByText(/Autre étape/)).toBeDefined()
	})

	it('mentions no other connector when nothing references the one being deleted', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [],
		}

		renderHarness({
			graph,
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.click(
			within(node).getByRole('button', { name: 'Supprimer Étape simple' }),
		)

		const dialog = await screen.findByRole('alertdialog')
		expect(within(dialog).queryByText(/Autre étape/)).toBeNull()
	})

	it('leaves the neighbours of a deleted middle node unwired, rather than stitching them together', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				connector('c1', SIMPLE_KIND),
				connector('c2', SIMPLE_KIND),
				connector('c3', SIMPLE_KIND),
			],
			edges: [
				{ from: 'c1', to: 'c2', branch: null },
				{ from: 'c2', to: 'c3', branch: null },
			],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([
			['c1', { x: 0, y: 0 }],
			['c2', { x: 280, y: 0 }],
			['c3', { x: 560, y: 0 }],
		])

		const { onSaveSpy } = renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const middleNode = await screen.findByTestId('rf__node-c2')
		fireEvent.click(
			within(middleNode).getByRole('button', {
				name: 'Supprimer Étape simple',
			}),
		)
		await screen.findByRole('alertdialog')
		fireEvent.click(screen.getByRole('button', { name: 'Supprimer' }))

		await waitFor(() => {
			expect(screen.queryByTestId('rf__node-c2')).toBeNull()
		})

		await clickSave()

		const [savedGraph] = onSaveSpy.mock.calls[0] as [Schemas.GraphDto]
		expect(savedGraph.connectors.map((c) => c.id)).toEqual(['c1', 'c3'])
		expect(savedGraph.edges).toEqual([])
	})
})

describe('WorkflowCanvas — the config panel', () => {
	const NOTED_DESCRIPTOR = descriptor(
		SIMPLE_KIND,
		'Étape simple',
		[],
		[textField('note', 'Note')],
	)

	it('opens on a connector click, showing its fields', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [],
			},
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(NOTED_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		expect(screen.queryByText('Paramètres')).toBeNull()

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.click(node)

		expect(await screen.findByText('Paramètres')).toBeDefined()
		expect(screen.getByLabelText('Note')).toBeDefined()
	})

	it('does not open the connector panel from a click on a trigger node', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [trigger('t1')],
			},
			layout: new Map([
				['t1', { x: -220, y: 0 }],
				['c1', { x: 0, y: 0 }],
			]),
			descriptors: descriptorMap(NOTED_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const triggerNode = await screen.findByTestId('rf__node-t1')
		fireEvent.click(triggerNode)

		expect(screen.queryByText('Paramètres')).toBeNull()
	})

	it('does not open from pressing delete on the node', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [],
			},
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(NOTED_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.click(
			within(node).getByRole('button', { name: 'Supprimer Étape simple' }),
		)

		await screen.findByRole('alertdialog')
		expect(screen.queryByText('Paramètres')).toBeNull()
	})

	it('does not open from pressing add on the node', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [],
			},
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(NOTED_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.click(
			within(node).getByRole('button', {
				name: 'Ajouter un connecteur après Étape simple',
			}),
		)

		expect(
			await screen.findByRole('button', { name: 'Étape simple' }),
		).toBeDefined()
		expect(screen.queryByText('Paramètres')).toBeNull()
	})

	it('closes on request', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [],
			},
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(NOTED_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		fireEvent.click(await screen.findByTestId('rf__node-c1'))
		await screen.findByText('Paramètres')

		fireEvent.click(screen.getByRole('button', { name: 'Fermer' }))

		expect(screen.queryByText('Paramètres')).toBeNull()
	})

	it('closes once the open connector is deleted', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [],
			},
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(NOTED_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.click(node)
		await screen.findByText('Paramètres')

		fireEvent.click(
			within(node).getByRole('button', { name: 'Supprimer Étape simple' }),
		)
		await screen.findByRole('alertdialog')
		fireEvent.click(screen.getByRole('button', { name: 'Supprimer' }))

		await waitFor(() => {
			expect(screen.queryByText('Paramètres')).toBeNull()
		})
	})

	it('routes a field edit through the same pipeline as a structural change, so a later drag never loses it', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([['c1', { x: 0, y: 0 }]])

		const { onSaveSpy } = renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(NOTED_DESCRIPTOR),
		})

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.click(node)
		await screen.findByText('Paramètres')

		fireEvent.change(screen.getByLabelText('Note'), {
			target: { value: 'hello' },
		})

		dragNodeBy(node, 20, 20)

		await waitFor(() => {
			expect(screen.getByText(/non enregistr/i)).toBeDefined()
		})

		await clickSave()

		const [savedGraph] = onSaveSpy.mock.calls[0] as [Schemas.GraphDto]
		expect(savedGraph.connectors[0]?.config).toEqual({ note: 'hello' })
	})
})

describe('WorkflowCanvas — the available-data tree', () => {
	const UPSTREAM_DESCRIPTOR = descriptor(SIMPLE_KIND, 'En amont', [], [])
	const SIBLING_DESCRIPTOR = descriptor(
		OTHER_KIND,
		'Sur l’autre branche',
		[],
		[],
	)

	it('offers only the connectors upstream of the open node, never a sibling branch', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				connector('c1', BRANCHING_KIND),
				connector('c2', SIMPLE_KIND),
				connector('c3', OTHER_KIND),
				connector('c4', SIMPLE_KIND),
			],
			edges: [
				{ from: 'c1', to: 'c2', branch: 'Then' },
				{ from: 'c1', to: 'c3', branch: 'Else' },
				{ from: 'c2', to: 'c4' },
			],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([
			['c1', { x: 0, y: 0 }],
			['c2', { x: 280, y: 0 }],
			['c3', { x: 280, y: 140 }],
			['c4', { x: 560, y: 0 }],
		])

		renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(
				BRANCHING_DESCRIPTOR,
				UPSTREAM_DESCRIPTOR,
				SIBLING_DESCRIPTOR,
			),
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c4')
		fireEvent.click(node)

		await screen.findByText('Paramètres')
		expect(
			screen.getByRole('button', { name: /c1 · Étape à embranchements/ }),
		).toBeDefined()
		expect(screen.getByRole('button', { name: /c2 · En amont/ })).toBeDefined()
		expect(
			screen.queryByRole('button', { name: /c3 · Sur l’autre branche/ }),
		).toBeNull()
	})

	it('fills the trigger branch from a trigger anywhere in the graph, before any run exists', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [trigger('t1', { Events: ['quote.accepted'] })],
			},
			layout: new Map([
				['t1', { x: -220, y: 0 }],
				['c1', { x: 0, y: 0 }],
			]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			events: [event('quote.accepted', { quote_id: 'q-1' })],
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.click(node)

		await screen.findByText('Paramètres')
		fireEvent.click(screen.getByRole('button', { name: 'trigger' }))

		expect(screen.getByText('quote_id')).toBeDefined()
	})

	it('shows the real values once a last run is supplied', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [],
			},
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			lastRun: {
				triggerPayload: { quote_id: 'q-real' },
				connectorOutputs: {},
			},
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.click(node)

		await screen.findByText('Paramètres')
		fireEvent.click(screen.getByRole('button', { name: 'trigger' }))

		expect(screen.getByText('quote_id')).toBeDefined()
	})
})

describe('WorkflowCanvas — framing the graph on load', () => {
	it('moves the camera onto the nodes instead of leaving them off-screen', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [],
			},
			layout: new Map([['c1', { x: 1800, y: 1400 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		await screen.findByTestId('rf__node-c1')

		const viewport = document.querySelector('.react-flow__viewport')
		await waitFor(() => {
			const transform = (viewport as HTMLElement).style.transform
			expect(transform).not.toBe('')
			expect(transform).not.toMatch(/translate\(0px,\s*0px\)\s*scale\(1\)/)
		})
	})
})

describe('WorkflowCanvas — a node opens where you just made it', () => {
	it("opens the configuration panel on the connector it just added from a trigger's +", async () => {
		renderHarness({
			graph: { connectors: [], edges: [], triggers: [trigger('t1')] },
			layout: new Map([['t1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const triggerNode = await screen.findByTestId('rf__node-t1')
		fireEvent.click(
			within(triggerNode).getByRole('button', {
				name: 'Ajouter un connecteur après le déclencheur t1',
			}),
		)
		fireEvent.click(await screen.findByText('Étape simple'))

		const panel = await screen.findByTestId('connector-config-panel')
		expect(within(panel).getByText('Étape simple')).toBeDefined()
	})

	it('closes the trigger panel rather than stacking two panels', async () => {
		renderHarness({
			graph: { connectors: [], edges: [], triggers: [trigger('t1')] },
			layout: new Map([['t1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const triggerNode = await screen.findByTestId('rf__node-t1')
		fireEvent.click(triggerNode)
		await screen.findByTestId('trigger-config-panel')

		fireEvent.click(
			within(triggerNode).getByRole('button', {
				name: 'Ajouter un connecteur après le déclencheur t1',
			}),
		)
		fireEvent.click(await screen.findByText('Étape simple'))

		await screen.findByTestId('connector-config-panel')
		expect(screen.queryByTestId('trigger-config-panel')).toBeNull()
	})
})

describe('WorkflowCanvas — the camera follows a new node', () => {
	it('moves the viewport when a node is added off-screen', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [],
			},
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const source = await screen.findByTestId('rf__node-c1')
		const viewport = document.querySelector(
			'.react-flow__viewport',
		) as HTMLElement
		const before = viewport.style.transform

		fireEvent.click(
			within(source).getByRole('button', {
				name: 'Ajouter un connecteur après Étape simple',
			}),
		)
		const options = await screen.findAllByText('Étape simple')
		const option = options.find((element) => element.closest('button'))
		fireEvent.click(option as HTMLElement)
		await screen.findByTestId('rf__node-c2')

		await waitFor(() => {
			expect(viewport.style.transform).not.toBe(before)
		})
	})
})

describe('WorkflowCanvas — the pane context menu', () => {
	it('opens on a right-click on empty canvas, offering a connector, a trigger, and framing — never the old workflow-level trigger panel', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [],
			},
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		await screen.findByTestId('rf__node-c1')
		const pane = document.querySelector('.react-flow__pane') as HTMLElement

		fireEvent.contextMenu(pane, { clientX: 200, clientY: 150 })

		expect(await screen.findByText('Ajouter un connecteur')).toBeDefined()
		expect(screen.getByText('Ajouter un déclencheur')).toBeDefined()
		expect(screen.getByText('Cadrer le graphe')).toBeDefined()
		expect(screen.queryByText('Configurer le déclencheur')).toBeNull()
	})

	it('does not open from a right-click on a node', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [],
			},
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.contextMenu(node, { clientX: 200, clientY: 150 })

		expect(screen.queryByText('Ajouter un connecteur')).toBeNull()
	})

	it('places a connector at the clicked point, unwired, when chosen from the menu', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([['c1', { x: 0, y: 0 }]])

		const { onSaveSpy } = renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		await screen.findByTestId('rf__node-c1')
		const pane = document.querySelector('.react-flow__pane') as HTMLElement

		fireEvent.contextMenu(pane, { clientX: 640, clientY: 260 })
		const expectedPosition = flowPositionOf(640, 260)

		fireEvent.click(await screen.findByText('Ajouter un connecteur'))
		fireEvent.click(await screen.findByRole('button', { name: 'Étape simple' }))

		await screen.findByTestId('rf__node-c2')
		await clickSave()

		const [savedGraph, savedLayout] = onSaveSpy.mock.calls[0] as [
			Schemas.GraphDto,
			Map<string, NodePosition>,
		]
		expect(savedGraph.connectors.map((c) => c.id)).toEqual(['c1', 'c2'])
		expect(savedGraph.edges).toEqual([])
		expect(savedLayout.get('c2')).toEqual(expectedPosition)
	})

	it('cadre le graphe on request, moving the viewport back over every node', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND), connector('c2', SIMPLE_KIND)],
			edges: [],
			triggers: [],
		}
		const layout = new Map<string, NodePosition>([
			['c1', { x: 0, y: 0 }],
			['c2', { x: 1800, y: 1400 }],
		])

		renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		await screen.findByTestId('rf__node-c1')
		const viewport = document.querySelector(
			'.react-flow__viewport',
		) as HTMLElement
		const pane = document.querySelector('.react-flow__pane') as HTMLElement

		fireEvent.contextMenu(pane, { clientX: 50, clientY: 50 })
		fireEvent.click(await screen.findByText('Ajouter un connecteur'))
		fireEvent.click(await screen.findByRole('button', { name: 'Étape simple' }))
		await screen.findByTestId('rf__node-c3')

		const afterAdd = viewport.style.transform

		fireEvent.contextMenu(pane, { clientX: 50, clientY: 50 })
		fireEvent.click(await screen.findByText('Cadrer le graphe'))

		await waitFor(() => {
			expect(viewport.style.transform).not.toBe(afterAdd)
		})
	})

	it('does not close an open panel the way a left pane click does', async () => {
		renderHarness({
			graph: {
				connectors: [connector('c1', SIMPLE_KIND)],
				edges: [],
				triggers: [],
			},
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.click(node)
		expect(await screen.findByTestId('connector-config-panel')).toBeDefined()

		const pane = document.querySelector('.react-flow__pane') as HTMLElement
		fireEvent.contextMenu(pane, { clientX: 50, clientY: 50 })

		expect(screen.getByTestId('connector-config-panel')).toBeDefined()
	})
})

describe('WorkflowCanvas — a run in progress', () => {
	function borderOf(nodeId: string): string {
		const node = screen.getByTestId(`rf__node-${nodeId}`)
		return (node.firstElementChild as HTMLElement).className
	}

	it('borders each connector by the status of its step', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				connector('c1', SIMPLE_KIND),
				connector('c2', SIMPLE_KIND),
				connector('c3', SIMPLE_KIND),
			],
			edges: [
				{ from: 't1', to: 'c1', branch: null },
				{ from: 'c1', to: 'c2', branch: null },
				{ from: 'c2', to: 'c3', branch: null },
			],
			triggers: [trigger('t1', 'Manual')],
		}

		renderHarness({
			graph,
			layout: new Map([
				['t1', { x: -220, y: 0 }],
				['c1', { x: 0, y: 0 }],
				['c2', { x: 200, y: 0 }],
				['c3', { x: 400, y: 0 }],
			]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			runStatuses: new Map([
				['c1', 'succeeded'],
				['c2', 'running'],
				['c3', 'failed'],
			]),
		})

		await screen.findByTestId('rf__node-c1')

		expect(borderOf('c1')).toContain('border-emerald-500')
		expect(borderOf('c2')).toContain('border-amber-500')
		expect(borderOf('c3')).toContain('border-destructive')
	})

	it('leaves a connector with no step of its own unmarked', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [{ from: 't1', to: 'c1', branch: null }],
			triggers: [trigger('t1', 'Manual')],
		}

		renderHarness({
			graph,
			layout: new Map([
				['t1', { x: -220, y: 0 }],
				['c1', { x: 0, y: 0 }],
			]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			runStatuses: new Map([['c1', 'pending']]),
		})

		await screen.findByTestId('rf__node-c1')

		expect(borderOf('c1')).not.toContain('border-amber-500')
		expect(borderOf('c1')).not.toContain('border-emerald-500')
		expect(borderOf('c1')).not.toContain('border-destructive')
	})
})
