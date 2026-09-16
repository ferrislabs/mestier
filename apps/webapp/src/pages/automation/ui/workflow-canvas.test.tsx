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
): Schemas.ConnectorDescriptorResponse {
	return {
		auth: 'None',
		branches,
		family: 'test',
		fields: [],
		kind,
		label,
		output_example: null,
		version: 1,
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
	hasTriggerEvent = true,
	onSaveSpy,
}: {
	graph: Schemas.GraphDto
	layout: Map<string, NodePosition>
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	connectorErrors?: Map<string, ConnectorValidationError[]>
	hasTriggerEvent?: boolean
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
		hasTriggerEvent,
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
			hasTriggerEvent: false,
			onSaveSpy: vi.fn(),
		})

		expect(await screen.findByText('Aucun événement configuré')).toBeDefined()
	})

	it('does not warn once an event is configured', async () => {
		renderHarness({
			graph: { connectors: [connector('c1', SIMPLE_KIND)], edges: [] },
			layout: new Map([['c1', { x: 0, y: 0 }]]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			hasTriggerEvent: true,
			onSaveSpy: vi.fn(),
		})

		await screen.findByTestId('rf__node-c1')
		expect(screen.queryByText('Aucun événement configuré')).toBeNull()
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
