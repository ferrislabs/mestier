import {
	fireEvent,
	render,
	screen,
	waitFor,
	within,
} from '@testing-library/react'
import { useRef, useState } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import type { NodePosition } from '#/pages/automation/lib/graph'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'
import {
	dragNodeBy,
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
	events = [],
	triggerEventNames = ['quote.accepted'],
	onSaveTrigger = () => {},
	isSavingTrigger = false,
	triggerSaveError = null,
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
	events?: Schemas.EventDescriptorResponse[]
	triggerEventNames?: string[]
	onSaveTrigger?: WorkflowCanvasProps['onSaveTrigger']
	isSavingTrigger?: boolean
	triggerSaveError?: string | null
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
		events,
		triggerEventNames,
		onSaveTrigger,
		isSavingTrigger,
		triggerSaveError,
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
	it('accepts a connection whose branch the source descriptor declares', () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				connector('c1', BRANCHING_KIND),
				connector('c2', SIMPLE_KIND),
			],
			edges: [],
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
})

describe('WorkflowCanvas — the virtual trigger', () => {
	it('connects to every root, unwired ones included', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [
				connector('c1', SIMPLE_KIND),
				connector('c2', SIMPLE_KIND),
				connector('c3', SIMPLE_KIND),
			],
			edges: [{ from: 'c1', to: 'c2', branch: null }],
		}
		const layout = new Map<string, NodePosition>([
			['c1', { x: 0, y: 0 }],
			['c2', { x: 280, y: 0 }],
			['c3', { x: 0, y: 140 }],
		])

		renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		await screen.findByTestId('rf__node-c1')
		expect(
			document.querySelector('[data-testid^="rf__edge-__trigger__->c1"]'),
		).not.toBeNull()
		expect(
			document.querySelector('[data-testid^="rf__edge-__trigger__->c3"]'),
		).not.toBeNull()
		expect(
			document.querySelector('[data-testid^="rf__edge-__trigger__->c2"]'),
		).toBeNull()
	})

	it('warns when the workflow has no event configured', async () => {
		renderHarness({
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			triggerEventNames: [],
			onSaveSpy: vi.fn(),
		})

		expect(await screen.findByText('Aucun événement configuré')).toBeDefined()
	})

	it('does not warn once an event is configured', async () => {
		renderHarness({
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			triggerEventNames: ['quote.accepted'],
			onSaveSpy: vi.fn(),
		})

		await screen.findByTestId('rf__node-c1')
		expect(screen.queryByText('Aucun événement configuré')).toBeNull()
	})

	it('drops the warning once the panel saves an event, without needing a remount', async () => {
		function ToggleHarness() {
			const [names, setNames] = useState<string[]>([])
			return (
				<Harness
					graph={{ connectors: [connector('c1', SIMPLE_KIND)], edges: [] }}
					layout={new Map([['c1', { x: 0, y: 0 }]])}
					descriptors={descriptorMap(SIMPLE_DESCRIPTOR)}
					triggerEventNames={names}
					onSaveTrigger={setNames}
					events={[event('quote.accepted')]}
					onSaveSpy={vi.fn()}
				/>
			)
		}

		render(<ToggleHarness />)
		expect(await screen.findByText('Aucun événement configuré')).toBeDefined()

		fireEvent.click(await screen.findByTestId('rf__node-__trigger__'))
		fireEvent.click(screen.getByRole('checkbox', { name: 'quote.accepted' }))
		fireEvent.click(
			within(screen.getByTestId('trigger-config-panel')).getByRole('button', {
				name: 'Enregistrer',
			}),
		)

		await waitFor(() => {
			expect(screen.queryByText('Aucun événement configuré')).toBeNull()
		})
	})
})

describe('WorkflowCanvas — the trigger picker', () => {
	it('opens the event picker when the trigger node is clicked', async () => {
		renderHarness({
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			events: [event('quote.accepted'), event('invoice.paid')],
			onSaveSpy: vi.fn(),
		})

		fireEvent.click(await screen.findByTestId('rf__node-__trigger__'))

		expect(
			await screen.findByRole('checkbox', { name: 'quote.accepted' }),
		).toBeDefined()
		expect(screen.getByRole('checkbox', { name: 'invoice.paid' })).toBeDefined()
	})

	it('saves the picked selection as a full replacement', async () => {
		const onSaveTrigger = vi.fn()
		renderHarness({
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			events: [event('quote.accepted'), event('invoice.paid')],
			triggerEventNames: ['quote.accepted'],
			onSaveTrigger,
			onSaveSpy: vi.fn(),
		})

		fireEvent.click(await screen.findByTestId('rf__node-__trigger__'))
		await screen.findByRole('checkbox', { name: 'quote.accepted' })
		fireEvent.click(screen.getByRole('checkbox', { name: 'invoice.paid' }))
		fireEvent.click(
			within(screen.getByTestId('trigger-config-panel')).getByRole('button', {
				name: 'Enregistrer',
			}),
		)

		expect(onSaveTrigger).toHaveBeenCalledWith([
			'quote.accepted',
			'invoice.paid',
		])
	})

	it('closes the connector panel when the trigger node opens, and vice versa', async () => {
		renderHarness({
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			events: [event('quote.accepted')],
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.click(node)
		expect(screen.getByTestId('connector-config-panel')).toBeDefined()

		fireEvent.click(await screen.findByTestId('rf__node-__trigger__'))
		expect(screen.queryByTestId('connector-config-panel')).toBeNull()
		expect(
			await screen.findByRole('checkbox', { name: 'quote.accepted' }),
		).toBeDefined()
	})
})

describe('WorkflowCanvas — validation badges', () => {
	it('badges the connector an error names', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
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
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			connectorErrors: new Map(),
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		expect(within(node).queryByLabelText(/Erreur/)).toBeNull()
	})
})

describe('WorkflowCanvas — dirty indicator', () => {
	it('is absent before any change and appears once the graph changes', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
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

	it("creates the first node of an empty workflow from the trigger's +", async () => {
		const { onSaveSpy } = renderHarness({
			graph: { connectors: [], edges: [] },
			layout: new Map(),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const triggerNode = await screen.findByTestId('rf__node-__trigger__')
		fireEvent.click(
			within(triggerNode).getByRole('button', {
				name: 'Ajouter le premier connecteur',
			}),
		)
		fireEvent.click(await screen.findByText('Étape simple'))

		await screen.findByTestId('rf__node-c1')
		expect(
			document.querySelector('[data-testid^="rf__edge-__trigger__->c1"]'),
		).not.toBeNull()

		await clickSave()

		const [savedGraph] = onSaveSpy.mock.calls[0] as [Schemas.GraphDto]
		expect(savedGraph).toEqual({
			connectors: [connector('c1', SIMPLE_KIND)],
			edges: [],
		})
	})
})

describe('WorkflowCanvas — deleting a node', () => {
	it('removes the node and its edges once confirmed', async () => {
		const graph: Schemas.GraphDto = {
			connectors: [connector('c1', SIMPLE_KIND), connector('c2', SIMPLE_KIND)],
			edges: [{ from: 'c1', to: 'c2', branch: null }],
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
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
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

	it('does not open from a click on the virtual trigger', async () => {
		renderHarness({
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(NOTED_DESCRIPTOR),
			onSaveSpy: vi.fn(),
		})

		const trigger = await screen.findByTestId('rf__node-__trigger__')
		fireEvent.click(trigger)

		expect(screen.queryByText('Paramètres')).toBeNull()
	})

	it('does not open from pressing delete on the node', async () => {
		renderHarness({
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
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
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
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
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
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
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
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

	it('fills the trigger branch from the selected event, before any run exists', async () => {
		renderHarness({
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			events: [event('quote.accepted', { quote_id: 'q-1' })],
			triggerEventNames: ['quote.accepted'],
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.click(node)

		await screen.findByText('Paramètres')
		fireEvent.click(screen.getByRole('button', { name: 'trigger' }))

		expect(screen.getByText('quote_id')).toBeDefined()
	})

	it('switches to the real values once a last run is supplied', async () => {
		renderHarness({
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			events: [event('quote.accepted', { quote_id: 'q-1' })],
			triggerEventNames: ['quote.accepted'],
			lastRun: {
				triggerPayload: { quote_id: 'q-real' },
				connectorOutputs: {},
			},
			onSaveSpy: vi.fn(),
		})

		const node = await screen.findByTestId('rf__node-c1')
		fireEvent.click(node)

		await screen.findByText('Paramètres')
		const lastRunButton = screen.getByRole('button', {
			name: 'Dernière exécution',
		}) as HTMLButtonElement
		expect(lastRunButton.disabled).toBe(false)

		fireEvent.click(lastRunButton)
		fireEvent.click(screen.getByRole('button', { name: 'trigger' }))

		expect(screen.getByText('quote_id')).toBeDefined()
	})
})
