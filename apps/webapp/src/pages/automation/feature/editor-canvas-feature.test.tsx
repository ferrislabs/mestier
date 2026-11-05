import {
	fireEvent,
	render,
	screen,
	waitFor,
	within,
} from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { useRef } from 'react'
import { beforeEach, describe, expect, it, type Mock, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { TooltipProvider } from '#/components/ui/tooltip'
import {
	type LastRunData,
	WorkflowCanvas,
	type WorkflowCanvasProps,
} from '#/pages/automation/feature/editor-canvas-feature'
import type { GraphChange } from '#/pages/automation/feature/use-workflow-graph'
import type { NodeMetaMap } from '#/pages/automation/lib/editor-nodes'
import type { NodePosition } from '#/pages/automation/lib/graph'
import type { ConnectorValidationError } from '#/pages/automation/lib/validation'
import {
	dragNodeBy,
	flowPositionOf,
	installFlowTestEnvironment,
} from '#/pages/automation/test/flow-test-env'

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

function textField(
	name: string,
	label: string,
	expression = false,
): Schemas.FieldResponse {
	return {
		expression,
		kind: 'Text',
		label,
		name,
		required: false,
		secret: false,
		visible_when: null,
	}
}

function descriptor(
	kind: string,
	label: string,
	branches: Schemas.BranchDto[] = [],
	fields: Schemas.FieldResponse[] = [],
	extra: Partial<Schemas.ConnectorDescriptorResponse> = {},
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
		...extra,
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

const SIMPLE_KIND = 'test.simple'
const BRANCHING_KIND = 'test.branching'
const OTHER_KIND = 'test.other'
const MIRRORING_KIND = 'test.mirroring'

const SIMPLE_DESCRIPTOR = descriptor(
	SIMPLE_KIND,
	'Étape simple',
	[],
	[textField('url', 'URL', true)],
)
const BRANCHING_DESCRIPTOR = descriptor(
	BRANCHING_KIND,
	'Étape à embranchements',
	['Then', 'Else'],
)
const OTHER_DESCRIPTOR = descriptor(OTHER_KIND, 'Autre étape')
const MIRRORING_DESCRIPTOR = descriptor(MIRRORING_KIND, 'Variables', [], [], {
	output_mirrors_field: 'variables',
	output_example: { generic_example: 1 },
})

function descriptorMap(
	...entries: Schemas.ConnectorDescriptorResponse[]
): Map<string, Schemas.ConnectorDescriptorResponse> {
	return new Map(entries.map((entry) => [entry.kind, entry]))
}

type HarnessProps = Partial<Omit<WorkflowCanvasProps, 'onChange'>> & {
	graph: Schemas.GraphDto
	layout: Map<string, NodePosition>
	descriptors: Map<string, Schemas.ConnectorDescriptorResponse>
	connectorErrors?: Map<string, ConnectorValidationError[]>
	lastRun?: LastRunData | null
	meta?: NodeMetaMap
	onSaveSpy?: (change: GraphChange) => void
	onTest?: () => void
	running?: boolean
}

function Harness({
	graph,
	layout,
	descriptors,
	meta = {},
	connectorErrors = new Map(),
	runStatuses = new Map(),
	runSteps = new Map(),
	events = [],
	lastRun = null,
	onEvaluateExpression = () => Promise.resolve(null),
	credentials = [],
	authSchemes = [],
	onCreateCredential,
	onSaveSpy = vi.fn(),
	onTest = vi.fn(),
	running = false,
}: HarnessProps) {
	const current = useRef<GraphChange>({ graph, layout, meta })

	return (
		<TooltipProvider>
			<WorkflowCanvas
				graph={graph}
				layout={layout}
				meta={meta}
				organizationSlug="atelier"
				descriptors={descriptors}
				connectorErrors={connectorErrors}
				runStatuses={runStatuses}
				runSteps={runSteps}
				events={events}
				lastRun={lastRun}
				onEvaluateExpression={onEvaluateExpression}
				credentials={credentials}
				authSchemes={authSchemes}
				onCreateCredential={onCreateCredential}
				testRun={{ running, start: onTest }}
				onChange={(change) => {
					current.current = change
				}}
			/>
			<button type="button" onClick={() => onSaveSpy(current.current)}>
				Enregistrer
			</button>
		</TooltipProvider>
	)
}

type SaveSpy = Mock<(change: GraphChange) => void>

function renderHarness(props: HarnessProps) {
	const onSaveSpy: SaveSpy = vi.fn<(change: GraphChange) => void>()
	render(<Harness {...props} onSaveSpy={onSaveSpy} />)
	return { onSaveSpy }
}

function saved(onSaveSpy: SaveSpy): GraphChange {
	fireEvent.click(screen.getByText('Enregistrer'))
	return onSaveSpy.mock.calls.at(-1)?.[0] as GraphChange
}

function graphWith(
	connectors: Schemas.PlacedConnectorDto[],
	edges: Schemas.EdgeDto[] = [],
	triggers: Schemas.PlacedTriggerDto[] = [],
): Schemas.GraphDto {
	return { connectors, edges, triggers }
}

function layoutOf(
	...positions: [string, number, number][]
): Map<string, NodePosition> {
	return new Map(positions.map(([id, x, y]) => [id, { x, y }]))
}

function nodeButton(id: string): HTMLElement {
	return screen
		.getByTestId(`rf__node-${id}`)
		.querySelector('button') as HTMLElement
}

async function pickInCreator(
	user: ReturnType<typeof userEvent.setup>,
	search: string,
	name: RegExp,
) {
	const panel = await screen.findByRole('complementary', {
		name: 'Créateur de nœuds',
	})
	await user.type(
		within(panel).getByRole('searchbox', { name: 'Rechercher un nœud' }),
		search,
	)
	await user.click(within(panel).getByRole('button', { name }))
}

async function openDetails(id: string): Promise<HTMLElement> {
	fireEvent.doubleClick(await screen.findByTestId(`rf__node-${id}`))
	return screen.findByRole('dialog')
}

describe('WorkflowCanvas — load and save round trip', () => {
	it('saves a graph with branches back unchanged', async () => {
		const graph = graphWith(
			[
				connector('c1', BRANCHING_KIND),
				connector('c2', SIMPLE_KIND),
				connector('c3', SIMPLE_KIND),
			],
			[
				{ from: 'c1', to: 'c2', branch: 'Then' },
				{ from: 'c1', to: 'c3', branch: 'Else' },
			],
		)
		const layout = layoutOf(['c1', 0, 0], ['c2', 280, 0], ['c3', 280, 140])

		const { onSaveSpy } = renderHarness({
			graph,
			layout,
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR, BRANCHING_DESCRIPTOR),
		})
		await screen.findByTestId('rf__node-c1')

		const { graph: savedGraph, layout: savedLayout } = saved(onSaveSpy)
		expect(savedGraph).toEqual(graph)
		expect([...savedLayout]).toEqual([...layout])
	})
})

describe('WorkflowCanvas — dragging', () => {
	it('snaps a moved node onto the 20px grid and keeps it when saved', async () => {
		const { onSaveSpy } = renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const node = await screen.findByTestId('rf__node-c1')
		dragNodeBy(node, 63, 47)

		await waitFor(() => {
			expect(node.style.transform).toContain('60px')
		})

		expect(saved(onSaveSpy).layout.get('c1')).toEqual({ x: 60, y: 40 })
	})
})

describe('WorkflowCanvas — branches drive handles', () => {
	it('shows one handle per declared branch, and a single handle when unbranched', async () => {
		renderHarness({
			graph: graphWith([
				connector('c1', BRANCHING_KIND),
				connector('c2', SIMPLE_KIND),
			]),
			layout: layoutOf(['c1', 0, 0], ['c2', 280, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR, BRANCHING_DESCRIPTOR),
		})

		const branching = await screen.findByTestId('rf__node-c1')
		expect(branching.querySelector('[data-handleid="Then"]')).not.toBeNull()
		expect(branching.querySelector('[data-handleid="Else"]')).not.toBeNull()
		expect(
			branching.querySelectorAll('.react-flow__handle.source').length,
		).toBe(2)

		const simple = await screen.findByTestId('rf__node-c2')
		const handles = simple.querySelectorAll('.react-flow__handle.source')
		expect(handles.length).toBe(1)
		expect(handles[0]?.getAttribute('data-handleid')).toBeNull()
	})
})

describe('WorkflowCanvas — triggers read as nodes', () => {
	it('names a manual and an event trigger differently, and counts several events', async () => {
		renderHarness({
			graph: graphWith(
				[],
				[],
				[
					trigger('t1', 'Manual'),
					trigger('t2', { Events: ['quote.accepted'] }),
					trigger('t3', { Events: ['quote.accepted', 'quote.sent'] }),
				],
			),
			layout: layoutOf(['t1', 0, 0], ['t2', 0, 200], ['t3', 0, 400]),
			descriptors: descriptorMap(),
			events: [event('quote.accepted'), event('quote.sent')],
		})

		await screen.findByTestId('rf__node-t1')
		expect(screen.getByText('Déclenchement manuel')).toBeDefined()
		expect(screen.getByText('Event Mestier · quote.accepted')).toBeDefined()
		expect(screen.getByText('2 événements')).toBeDefined()
	})

	it('asks to set up an event trigger with nothing selected, never a manual one', async () => {
		renderHarness({
			graph: graphWith(
				[],
				[],
				[trigger('t1', 'Manual'), trigger('t2', { Events: [] })],
			),
			layout: layoutOf(['t1', 0, 0], ['t2', 0, 200]),
			descriptors: descriptorMap(),
			events: [event('quote.accepted')],
		})

		await screen.findByTestId('rf__node-t1')
		expect(
			within(screen.getByTestId('rf__node-t1')).queryByRole('img', {
				name: 'À configurer',
			}),
		).toBeNull()
		expect(
			within(screen.getByTestId('rf__node-t2')).getByRole('img', {
				name: 'À configurer',
			}),
		).toBeDefined()
	})

	it('picks the event of a trigger that was never set up with one click', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith([], [], [trigger('t1', { Events: [] })]),
			layout: layoutOf(['t1', 0, 0]),
			descriptors: descriptorMap(),
			events: [event('quote.accepted'), event('quote.sent')],
		})

		fireEvent.click(await screen.findByTestId('rf__node-t1'))
		await user.click(await screen.findByRole('button', { name: /quote\.sent/ }))
		await user.click(
			screen.getByRole('button', { name: 'Utiliser cet événement' }),
		)

		expect(saved(onSaveSpy).graph.triggers).toEqual([
			{ id: 't1', kind: { Events: ['quote.sent'] } },
		])
	})
})

describe('WorkflowCanvas — adding nodes', () => {
	it('opens the creator on the triggers when the canvas is empty', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith([]),
			layout: new Map(),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		await user.click(
			await screen.findByRole('button', { name: 'Ajouter la première étape' }),
		)
		expect(
			await screen.findByText('Qu’est-ce qui déclenche ce workflow ?'),
		).toBeDefined()

		await user.click(screen.getByRole('button', { name: /Clic/ }))

		expect(saved(onSaveSpy).graph.triggers).toEqual([
			{ id: 't1', kind: 'Manual' },
		])
	})

	it('wires the new connector on the branch whose + was pressed', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith(
				[connector('c1', BRANCHING_KIND)],
				[{ from: 't1', to: 'c1', branch: null }],
				[trigger('t1')],
			),
			layout: layoutOf(['t1', -220, 0], ['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR, BRANCHING_DESCRIPTOR),
		})

		await screen.findByTestId('rf__node-c1')
		await user.click(
			screen.getByRole('button', {
				name: 'Ajouter un nœud après « Étape à embranchements » (Else)',
			}),
		)
		await pickInCreator(user, 'simple', /Étape simple/)

		const { graph, layout } = saved(onSaveSpy)
		expect(graph.edges).toEqual([
			{ from: 't1', to: 'c1', branch: null },
			{ from: 'c1', to: 'c2', branch: 'Else' },
		])
		expect(layout.get('c2')?.x).toBe(220)
	})

	it("wires the first connector of a workflow from its trigger's +", async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith([], [], [trigger('t1')]),
			layout: layoutOf(['t1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		await screen.findByTestId('rf__node-t1')
		await user.click(
			screen.getByRole('button', { name: 'Ajouter un nœud après « Clic »' }),
		)
		await pickInCreator(user, 'simple', /Étape simple/)

		expect(saved(onSaveSpy).graph.edges).toEqual([
			{ from: 't1', to: 'c1', branch: null },
		])
	})

	it('adds a node at the point right-clicked on the pane', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)], [], [trigger('t1')]),
			layout: layoutOf(['t1', -220, 0], ['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		await screen.findByTestId('rf__node-c1')
		const pane = document.querySelector('.react-flow__pane') as HTMLElement
		fireEvent.contextMenu(pane, { clientX: 640, clientY: 260 })
		const expected = flowPositionOf(640, 260)
		await user.click(
			await screen.findByRole('menuitem', { name: /Ajouter un nœud/ }),
		)
		await pickInCreator(user, 'simple', /Étape simple/)

		const placed = saved(onSaveSpy).layout.get('c2')
		const grid = (value: number) => Math.round(value / 20) * 20
		expect(placed).toEqual({
			x: grid(grid(expected.x) - 50),
			y: grid(grid(expected.y) - 50),
		})
	})

	it('finds a node across categories with a search, ignoring accents', async () => {
		const user = userEvent.setup()
		renderHarness({
			graph: graphWith([], [], [trigger('t1')]),
			layout: layoutOf(['t1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR, OTHER_DESCRIPTOR),
		})

		await screen.findByTestId('rf__node-t1')
		await user.click(screen.getByRole('button', { name: 'Ajouter un nœud' }))
		const panel = await screen.findByRole('complementary', {
			name: 'Créateur de nœuds',
		})
		await user.type(
			within(panel).getByRole('searchbox', { name: 'Rechercher un nœud' }),
			'etape s',
		)

		expect(
			within(panel).getByRole('button', { name: /Étape simple/ }),
		).toBeDefined()
		expect(
			within(panel).queryByRole('button', { name: /Autre étape/ }),
		).toBeNull()
	})
})

describe('WorkflowCanvas — a node made from a collection operation', () => {
	const HTTP_DESCRIPTOR = {
		...descriptor(
			'http.request',
			'Requête HTTP',
			[],
			[
				{
					...textField('method', 'Méthode'),
					kind: { Select: { options: [{ value: 'GET', label: 'GET' }] } },
				},
				textField('url', 'URL', true),
			],
		),
		family: 'http',
	}

	it('builds an http connector from the operation and remembers where it came from', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith([], [], [trigger('t1')]),
			layout: layoutOf(['t1', 0, 0]),
			descriptors: descriptorMap(HTTP_DESCRIPTOR),
		})

		await screen.findByTestId('rf__node-t1')
		await user.click(
			screen.getByRole('button', { name: 'Ajouter un nœud après « Clic »' }),
		)
		const panel = await screen.findByRole('complementary', {
			name: 'Créateur de nœuds',
		})
		await user.click(within(panel).getByRole('button', { name: /Collections/ }))
		const collections = within(panel).getAllByRole('button', {
			name: /opération/,
		})
		await user.click(collections[0])
		const operations = within(panel)
			.getAllByRole('button')
			.filter((button) =>
				/^(GET|POST|PUT|PATCH|DELETE)/.test(button.textContent ?? ''),
			)
		await user.click(operations[0])

		const change = saved(onSaveSpy)
		const created = change.graph.connectors[0]
		expect(created.kind).toBe('http.request')
		expect(created.config.url).toMatch(/^https?:\/\//)
		expect(change.meta.c1?.operation).toBeDefined()

		const dialog = await openDetails('c1')
		expect(within(dialog).getByText('Opération')).toBeDefined()
	})
})

describe('WorkflowCanvas — deleting', () => {
	it('removes a trigger and every edge it fed, with no confirmation', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith(
				[connector('c1', SIMPLE_KIND)],
				[{ from: 't1', to: 'c1', branch: null }],
				[trigger('t1')],
			),
			layout: layoutOf(['t1', -220, 0], ['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		fireEvent.contextMenu(await screen.findByTestId('rf__node-t1'), {
			clientX: 100,
			clientY: 100,
		})
		await user.click(await screen.findByRole('menuitem', { name: /Supprimer/ }))

		await waitFor(() => {
			expect(screen.queryByTestId('rf__node-t1')).toBeNull()
		})
		const { graph } = saved(onSaveSpy)
		expect(graph.triggers).toEqual([])
		expect(graph.edges).toEqual([])
		expect(screen.queryByRole('alertdialog')).toBeNull()
	})

	it('stitches the neighbours of a deleted middle node back together, and undoes it', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith(
				[connector('c1', SIMPLE_KIND), connector('c2', SIMPLE_KIND)],
				[
					{ from: 't1', to: 'c1', branch: null },
					{ from: 'c1', to: 'c2', branch: null },
				],
				[trigger('t1')],
			),
			layout: layoutOf(['t1', -220, 0], ['c1', 0, 0], ['c2', 220, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		fireEvent.contextMenu(await screen.findByTestId('rf__node-c1'), {
			clientX: 100,
			clientY: 100,
		})
		await user.click(await screen.findByRole('menuitem', { name: /Supprimer/ }))

		await waitFor(() => {
			expect(screen.queryByTestId('rf__node-c1')).toBeNull()
		})
		expect(saved(onSaveSpy).graph.edges).toEqual([
			{ from: 't1', to: 'c2', branch: null },
		])

		await user.click(
			screen.getByRole('button', { name: 'Annuler la dernière action' }),
		)

		expect(await screen.findByTestId('rf__node-c1')).toBeDefined()
		expect(saved(onSaveSpy).graph.edges).toEqual([
			{ from: 't1', to: 'c1', branch: null },
			{ from: 'c1', to: 'c2', branch: null },
		])
	})

	it('flags a node whose expression points at a deleted node, instead of asking first', async () => {
		renderHarness({
			graph: graphWith(
				[
					connector('c2', SIMPLE_KIND, {
						url: '{{ connectors.c1.output.id }}',
					}),
				],
				[{ from: 't1', to: 'c2', branch: null }],
				[trigger('t1')],
			),
			layout: layoutOf(['t1', -220, 0], ['c2', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		await screen.findByTestId('rf__node-c2')
		expect(
			within(screen.getByTestId('rf__node-c2')).getByRole('img', {
				name: /À corriger : Le nœud « c1 » n’existe plus\./,
			}),
		).toBeDefined()
	})
})

describe('WorkflowCanvas — the node details', () => {
	it('opens on a double click with the input, parameters and output columns', async () => {
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND, { url: 'https://a' })]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const dialog = await openDetails('c1')

		expect(within(dialog).getByRole('region', { name: 'Entrée' })).toBeDefined()
		expect(
			within(dialog).getByRole('region', { name: 'Paramètres' }),
		).toBeDefined()
		expect(within(dialog).getByRole('region', { name: 'Sortie' })).toBeDefined()
		expect(
			(within(dialog).getByLabelText('URL') as HTMLInputElement).value,
		).toBe('https://a')
		expect(
			within(dialog).getByText('Ce nœud n’a pas été exécuté'),
		).toBeDefined()
	})

	it('does not open on a single click, and closes on request', async () => {
		const user = userEvent.setup()
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		fireEvent.click(await screen.findByTestId('rf__node-c1'))
		expect(screen.queryByRole('dialog')).toBeNull()

		await openDetails('c1')
		await user.click(
			screen.getByRole('button', { name: 'Fermer les détails du nœud' }),
		)
		await waitFor(() => {
			expect(screen.queryByRole('dialog')).toBeNull()
		})
	})

	it('opens from the keyboard with Enter on the node', async () => {
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		await screen.findByTestId('rf__node-c1')
		fireEvent.keyDown(nodeButton('c1'), { key: 'Enter' })

		expect(await screen.findByRole('dialog')).toBeDefined()
	})

	it('routes a field edit through the same pipeline as a structural change', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const dialog = await openDetails('c1')
		await user.type(within(dialog).getByLabelText('URL'), 'abc')
		await user.click(
			screen.getByRole('button', { name: 'Fermer les détails du nœud' }),
		)
		await waitFor(() => {
			expect(screen.queryByRole('dialog')).toBeNull()
		})
		dragNodeBy(screen.getByTestId('rf__node-c1'), 60, 40)

		await waitFor(() => {
			expect(saved(onSaveSpy).layout.get('c1')).toEqual({ x: 60, y: 40 })
		})
		expect(saved(onSaveSpy).graph.connectors[0].config).toEqual({ url: 'abc' })
	})

	it('renames the node inline and keeps the name out of the saved graph', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		const dialog = await openDetails('c1')
		await user.click(
			within(dialog).getByRole('button', {
				name: /Nom du nœud : Étape simple/,
			}),
		)
		const input = within(dialog).getByLabelText('Nom du nœud')
		await user.clear(input)
		await user.type(input, 'Créer le projet{Enter}')

		const change = saved(onSaveSpy)
		expect(change.meta).toEqual({ c1: { label: 'Créer le projet' } })
		expect(change.graph.connectors[0]).toEqual(connector('c1', SIMPLE_KIND))
		expect(screen.getAllByText('Créer le projet').length).toBeGreaterThan(0)
	})

	it('refuses a name another node already carries', async () => {
		const user = userEvent.setup()
		renderHarness({
			graph: graphWith([
				connector('c1', SIMPLE_KIND),
				connector('c2', OTHER_KIND),
			]),
			layout: layoutOf(['c1', 0, 0], ['c2', 220, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR, OTHER_DESCRIPTOR),
		})

		const dialog = await openDetails('c1')
		await user.click(
			within(dialog).getByRole('button', {
				name: /Nom du nœud : Étape simple/,
			}),
		)
		const input = within(dialog).getByLabelText('Nom du nœud')
		await user.clear(input)
		await user.type(input, 'Autre étape{Enter}')

		expect(
			within(dialog).getByText('Un autre nœud porte déjà ce nom.'),
		).toBeDefined()
	})

	it('moves to the next node from the navigation arrows', async () => {
		const user = userEvent.setup()
		renderHarness({
			graph: graphWith(
				[connector('c1', SIMPLE_KIND), connector('c2', OTHER_KIND)],
				[{ from: 'c1', to: 'c2', branch: null }],
			),
			layout: layoutOf(['c1', 0, 0], ['c2', 220, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR, OTHER_DESCRIPTOR),
		})

		await openDetails('c1')
		await user.click(
			screen.getByRole('button', {
				name: /Ouvrir le nœud suivant : Autre étape/,
			}),
		)

		expect(
			await screen.findByRole('button', {
				name: /Ouvrir le nœud précédent : Étape simple/,
			}),
		).toBeDefined()
	})

	it('lists the keys of the nodes upstream, never a sibling branch', async () => {
		renderHarness({
			graph: graphWith(
				[
					connector('c1', BRANCHING_KIND),
					connector('c2', SIMPLE_KIND),
					connector('c3', OTHER_KIND),
					connector('c4', SIMPLE_KIND),
				],
				[
					{ from: 'c1', to: 'c2', branch: 'Then' },
					{ from: 'c1', to: 'c3', branch: 'Else' },
					{ from: 'c2', to: 'c4', branch: null },
				],
			),
			layout: layoutOf(
				['c1', 0, 0],
				['c2', 220, 0],
				['c3', 220, 140],
				['c4', 440, 0],
			),
			descriptors: descriptorMap(
				descriptor(BRANCHING_KIND, 'Branches', ['Then', 'Else'], [], {
					output_example: { from_branching: 1 },
				}),
				descriptor(SIMPLE_KIND, 'Simple', [], [], {
					output_example: { from_simple: 1 },
				}),
				descriptor(OTHER_KIND, 'Voisine', [], [], {
					output_example: { from_sibling: 1 },
				}),
			),
		})

		const dialog = await openDetails('c4')
		const input = within(dialog).getByRole('region', { name: 'Entrée' })

		expect(within(input).getByText('from_branching')).toBeDefined()
		expect(within(input).getByText('from_simple')).toBeDefined()
		expect(within(input).queryByText('from_sibling')).toBeNull()
		expect(
			within(input)
				.getByRole('button', {
					name: /from_simple, texte|from_simple, nombre/,
				})
				.getAttribute('aria-label'),
		).toContain('{{ connectors.c2.output.from_simple }}')
	})

	it('fills the trigger entry from the subscribed event before any run exists', async () => {
		renderHarness({
			graph: graphWith(
				[connector('c1', SIMPLE_KIND)],
				[],
				[trigger('t1', { Events: ['quote.accepted'] })],
			),
			layout: layoutOf(['t1', -220, 0], ['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			events: [event('quote.accepted', { quote_id: 'q-1' })],
		})

		const dialog = await openDetails('c1')

		expect(within(dialog).getByText('quote_id')).toBeDefined()
		expect(
			within(dialog)
				.getByRole('button', { name: /quote_id/ })
				.getAttribute('aria-label'),
		).toContain('{{ trigger.quote_id }}')
	})

	it('shows the real values on the Exécution view once a last run exists', async () => {
		const user = userEvent.setup()
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)], [], [trigger('t1')]),
			layout: layoutOf(['t1', -220, 0], ['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			lastRun: { triggerPayload: { quote_id: 'q-real' }, connectorOutputs: {} },
		})

		const dialog = await openDetails('c1')
		await user.click(within(dialog).getByRole('radio', { name: 'Exécution' }))

		expect(within(dialog).getByText('q-real')).toBeDefined()
	})

	it('offers the variables a mirroring connector actually carries', async () => {
		renderHarness({
			graph: graphWith(
				[
					connector('c1', MIRRORING_KIND, {
						variables: { base_url: 'https://x' },
					}),
					connector('c2', SIMPLE_KIND),
				],
				[
					{ from: 't1', to: 'c1', branch: null },
					{ from: 'c1', to: 'c2', branch: null },
				],
				[trigger('t1')],
			),
			layout: layoutOf(['t1', -220, 0], ['c1', 0, 0], ['c2', 220, 0]),
			descriptors: descriptorMap(MIRRORING_DESCRIPTOR, SIMPLE_DESCRIPTOR),
		})

		const dialog = await openDetails('c2')

		expect(within(dialog).getByText('base_url')).toBeDefined()
		expect(within(dialog).queryByText('generic_example')).toBeNull()
	})

	it('shows the step of the last run in the output column', async () => {
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			runSteps: new Map([
				[
					'c1',
					{
						attempts: 1,
						connector_id: 'c1',
						created_at: '2026-01-01T00:00:00Z',
						id: 's1',
						iteration_path: '',
						status: 'succeeded',
						output: { created_id: 'abc-123' },
					},
				],
			]),
		})

		const dialog = await openDetails('c1')

		expect(
			within(within(dialog).getByRole('region', { name: 'Sortie' })).getByText(
				'abc-123',
			),
		).toBeDefined()
	})
})

describe('WorkflowCanvas — validation', () => {
	it('badges the node an error names, and lands the field error in its details', async () => {
		renderHarness({
			graph: graphWith([
				connector('c1', SIMPLE_KIND),
				connector('c2', OTHER_KIND),
			]),
			layout: layoutOf(['c1', 0, 0], ['c2', 220, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR, OTHER_DESCRIPTOR),
			connectorErrors: new Map([
				['c1', [{ field: 'url', message: 'URL manquante' }]],
			]),
		})

		await screen.findByTestId('rf__node-c1')
		expect(
			within(screen.getByTestId('rf__node-c1')).getByRole('img', {
				name: 'À corriger : URL manquante',
			}),
		).toBeDefined()
		expect(
			within(screen.getByTestId('rf__node-c2')).queryByRole('img', {
				name: /À corriger/,
			}),
		).toBeNull()

		const dialog = await openDetails('c1')
		expect(within(dialog).getByRole('alert').textContent).toContain(
			'URL manquante',
		)
	})

	it('badges a trigger wired to nothing, and names the message in its details', async () => {
		renderHarness({
			graph: graphWith([], [], [trigger('t1')]),
			layout: layoutOf(['t1', 0, 0]),
			descriptors: descriptorMap(),
			connectorErrors: new Map([
				['t1', [{ field: null, message: 'Le déclencheur n’alimente rien.' }]],
			]),
		})

		await screen.findByTestId('rf__node-t1')
		expect(
			within(screen.getByTestId('rf__node-t1')).getByRole('img', {
				name: /À corriger : Le déclencheur n’alimente rien\./,
			}),
		).toBeDefined()

		const dialog = await openDetails('t1')
		expect(within(dialog).getByRole('alert').textContent).toContain(
			'Le déclencheur n’alimente rien.',
		)
	})
})

describe('WorkflowCanvas — a run in progress', () => {
	function frameOf(id: string): string {
		return nodeButton(id).className
	}

	it('borders each connector by the status of its step', async () => {
		renderHarness({
			graph: graphWith(
				[
					connector('c1', SIMPLE_KIND),
					connector('c2', SIMPLE_KIND),
					connector('c3', SIMPLE_KIND),
				],
				[],
				[trigger('t1')],
			),
			layout: layoutOf(
				['t1', -220, 0],
				['c1', 0, 0],
				['c2', 200, 0],
				['c3', 400, 0],
			),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			runStatuses: new Map([
				['c1', 'succeeded'],
				['c2', 'running'],
				['c3', 'failed'],
			]),
		})

		await screen.findByTestId('rf__node-c1')

		expect(frameOf('c1')).toContain('border-(--auto-success)')
		expect(frameOf('c2')).toContain('border-state-today')
		expect(frameOf('c3')).toContain('border-state-alert')
	})

	it('leaves a connector with no step of its own unmarked', async () => {
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			runStatuses: new Map([['c1', 'pending']]),
		})

		await screen.findByTestId('rf__node-c1')

		expect(frameOf('c1')).not.toContain('border-state-today')
		expect(frameOf('c1')).not.toContain('border-(--auto-success)')
		expect(frameOf('c1')).not.toContain('border-state-alert')
	})
})

describe('WorkflowCanvas — the node menu', () => {
	it('lists every action in order, with its shortcut', async () => {
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		fireEvent.contextMenu(await screen.findByTestId('rf__node-c1'), {
			clientX: 200,
			clientY: 150,
		})

		const items = await screen.findAllByRole('menuitem')
		expect(
			items.map((item) => item.textContent?.replace(/\s+/g, ' ').trim()),
		).toEqual([
			expect.stringMatching(/^Ouvrir/),
			expect.stringMatching(/^Renommer\s?F2/),
			expect.stringMatching(/^Désactiver\s?D/),
			expect.stringMatching(/^Copier/),
			expect.stringMatching(/^Dupliquer/),
			expect.stringMatching(/^Supprimer/),
		])
	})

	it('opens the details from the menu', async () => {
		const user = userEvent.setup()
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		fireEvent.contextMenu(await screen.findByTestId('rf__node-c1'), {
			clientX: 200,
			clientY: 150,
		})
		await user.click(await screen.findByRole('menuitem', { name: /Ouvrir/ }))

		expect(await screen.findByRole('dialog')).toBeDefined()
	})

	it('renames from the menu through a dialog', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		fireEvent.contextMenu(await screen.findByTestId('rf__node-c1'), {
			clientX: 200,
			clientY: 150,
		})
		await user.click(await screen.findByRole('menuitem', { name: /Renommer/ }))
		const input = await screen.findByLabelText('Nom du nœud')
		await user.clear(input)
		await user.type(input, 'Nouveau nom')
		await user.click(screen.getByRole('button', { name: 'Renommer' }))

		expect(saved(onSaveSpy).meta).toEqual({ c1: { label: 'Nouveau nom' } })
	})

	it('disables and re-enables a node, marking it on the canvas', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		fireEvent.contextMenu(await screen.findByTestId('rf__node-c1'), {
			clientX: 200,
			clientY: 150,
		})
		await user.click(
			await screen.findByRole('menuitem', { name: /Désactiver/ }),
		)

		expect(screen.getByText('Désactivé')).toBeDefined()
		expect(saved(onSaveSpy).meta).toEqual({ c1: { disabled: true } })

		fireEvent.contextMenu(screen.getByTestId('rf__node-c1'), {
			clientX: 200,
			clientY: 150,
		})
		await user.click(await screen.findByRole('menuitem', { name: /Réactiver/ }))

		expect(screen.queryByText('Désactivé')).toBeNull()
	})

	it('duplicates a node under a new id and a copy name', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND, { url: 'https://a' })]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		fireEvent.contextMenu(await screen.findByTestId('rf__node-c1'), {
			clientX: 200,
			clientY: 150,
		})
		await user.click(await screen.findByRole('menuitem', { name: /Dupliquer/ }))

		const change = saved(onSaveSpy)
		expect(change.graph.connectors.map((c) => c.id)).toEqual(['c1', 'c2'])
		expect(change.graph.connectors[1].config).toEqual({ url: 'https://a' })
		expect(change.meta.c2?.label).toBe('Étape simple (copie)')
	})

	it('copies a node, then offers to paste it from the pane menu', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		fireEvent.contextMenu(await screen.findByTestId('rf__node-c1'), {
			clientX: 200,
			clientY: 150,
		})
		await user.click(await screen.findByRole('menuitem', { name: /Copier/ }))

		const pane = document.querySelector('.react-flow__pane') as HTMLElement
		fireEvent.contextMenu(pane, { clientX: 500, clientY: 300 })
		await user.click(
			await screen.findByRole('menuitem', { name: /Coller « Étape simple »/ }),
		)

		expect(saved(onSaveSpy).graph.connectors.map((c) => c.id)).toEqual([
			'c1',
			'c2',
		])
	})

	it('does not offer to paste before anything was copied', async () => {
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		await screen.findByTestId('rf__node-c1')
		const pane = document.querySelector('.react-flow__pane') as HTMLElement
		fireEvent.contextMenu(pane, { clientX: 500, clientY: 300 })

		expect(
			await screen.findByRole('menuitem', { name: /Ajouter un nœud/ }),
		).toBeDefined()
		expect(screen.queryByRole('menuitem', { name: /Coller/ })).toBeNull()
	})
})

describe('WorkflowCanvas — keyboard shortcuts', () => {
	it('toggles the selected node with D and renames it with F2', async () => {
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		fireEvent.click(await screen.findByTestId('rf__node-c1'))
		fireEvent.keyDown(window, { key: 'd' })
		expect(await screen.findByText('Désactivé')).toBeDefined()

		fireEvent.keyDown(window, { key: 'F2' })
		expect(await screen.findByLabelText('Nom du nœud')).toBeDefined()
	})

	it('undoes and redoes with the usual shortcuts', async () => {
		const { onSaveSpy } = renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		fireEvent.click(await screen.findByTestId('rf__node-c1'))
		fireEvent.keyDown(window, { key: 'd' })
		await screen.findByText('Désactivé')

		fireEvent.keyDown(window, { key: 'z', ctrlKey: true })
		await waitFor(() => {
			expect(screen.queryByText('Désactivé')).toBeNull()
		})

		fireEvent.keyDown(window, { key: 'y', ctrlKey: true })
		expect(await screen.findByText('Désactivé')).toBeDefined()
		expect(saved(onSaveSpy).meta).toEqual({ c1: { disabled: true } })
	})
})

describe('WorkflowCanvas — the controls', () => {
	it('tidies the nodes into columns on the grid', async () => {
		const user = userEvent.setup()
		const { onSaveSpy } = renderHarness({
			graph: graphWith(
				[connector('c1', SIMPLE_KIND), connector('c2', SIMPLE_KIND)],
				[
					{ from: 't1', to: 'c1', branch: null },
					{ from: 'c1', to: 'c2', branch: null },
				],
				[trigger('t1')],
			),
			layout: layoutOf(['t1', 3, 9], ['c1', 500, 333], ['c2', 41, 700]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		await screen.findByTestId('rf__node-c1')
		await user.click(screen.getByRole('button', { name: 'Ranger le workflow' }))

		const { layout } = saved(onSaveSpy)
		expect(layout.get('t1')).toEqual({ x: 0, y: 0 })
		expect(layout.get('c1')).toEqual({ x: 220, y: 0 })
		expect(layout.get('c2')).toEqual({ x: 440, y: 0 })
	})

	it('disables undo and redo until there is something to undo', async () => {
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
		})

		await screen.findByTestId('rf__node-c1')

		expect(
			(
				screen.getByRole('button', {
					name: 'Annuler la dernière action',
				}) as HTMLButtonElement
			).disabled,
		).toBe(true)
		expect(
			(
				screen.getByRole('button', {
					name: 'Rétablir l’action annulée',
				}) as HTMLButtonElement
			).disabled,
		).toBe(true)
	})
})

describe('WorkflowCanvas — testing the workflow', () => {
	it('starts a test from the bottom button', async () => {
		const user = userEvent.setup()
		const onTest = vi.fn()
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			onTest,
		})

		await user.click(
			await screen.findByRole('button', { name: 'Tester le workflow' }),
		)

		expect(onTest).toHaveBeenCalledTimes(1)
	})

	it('shows the progress and refuses a second start while a run is going', async () => {
		renderHarness({
			graph: graphWith([connector('c1', SIMPLE_KIND)]),
			layout: layoutOf(['c1', 0, 0]),
			descriptors: descriptorMap(SIMPLE_DESCRIPTOR),
			running: true,
		})

		const button = await screen.findByRole('button', {
			name: 'Exécution en cours…',
		})
		expect((button as HTMLButtonElement).disabled).toBe(true)
	})

	it('is disabled on an empty canvas', async () => {
		renderHarness({
			graph: graphWith([]),
			layout: new Map(),
			descriptors: descriptorMap(),
		})

		const button = await screen.findByRole('button', {
			name: 'Tester le workflow',
		})
		expect((button as HTMLButtonElement).disabled).toBe(true)
	})
})
