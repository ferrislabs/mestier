import { screen, within } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { Schemas } from '#/api/api.client'
import { TooltipProvider } from '#/components/ui/tooltip'
import { readCollections } from '#/pages/automation/feature/mocks/collections-store'
import { NodeDetailsFeature } from '#/pages/automation/feature/node-details-feature'
import {
	connectorNode,
	type EditorEdge,
	type EditorLookup,
	type EditorNode,
	triggerNode,
} from '#/pages/automation/lib/editor-nodes'
import type { UpstreamSources } from '#/pages/automation/lib/upstream-entries'
import { renderWithRouter } from '#/test/render-with-router'

const NODE: EditorNode = {
	id: 'c1',
	type: 'automation',
	position: { x: 0, y: 0 },
	data: {
		kind: 'connector',
		label: 'Envoyer l’e-mail',
		subtitle: '',
		disabled: false,
		meta: {},
		typeLabel: 'Envoyer un e-mail',
		branches: [],
		errors: [],
		runStatus: 'failed',
		connector: { id: 'c1', kind: 'mail.send', version: 1, config: {} },
	},
}

const FAILED_STEP: Schemas.RunStepResponse = {
	id: 's1',
	connector_id: 'c1',
	iteration_path: '',
	attempts: 1,
	status: 'failed',
	created_at: '2026-08-05T00:00:01Z',
	error: 'SMTP expiré',
}

describe('NodeDetailsFeature', () => {
	it('sends a failed node to the identifications', async () => {
		await renderWithRouter(
			<NodeDetailsFeature
				node={NODE}
				nodes={[NODE]}
				edges={[]}
				organizationSlug="atelier-bois"
				sources={{
					descriptors: new Map(),
					events: [],
					lastRun: null,
					runSteps: new Map([['c1', FAILED_STEP]]),
				}}
				authSchemes={[]}
				credentials={[]}
				issues={[]}
				testRun={{ running: false, start: vi.fn() }}
				onUpdateConnector={vi.fn()}
				onUpdateTriggerKind={vi.fn()}
				onUpdateMeta={vi.fn()}
				onCreateCredential={vi.fn()}
				onEvaluateExpression={vi.fn()}
				onClose={vi.fn()}
				onNavigate={vi.fn()}
			/>,
		)

		const dialog = await screen.findByRole('dialog')
		expect(
			within(dialog)
				.getByRole('link', { name: 'Ouvrir les identifications' })
				.getAttribute('href'),
		).toBe('/o/atelier-bois/automatisation/identifications')
	})
})

function descriptor(
	kind: string,
	family: string,
	overrides: Partial<Schemas.ConnectorDescriptorResponse> = {},
): Schemas.ConnectorDescriptorResponse {
	return {
		auth: 'None',
		branches: [],
		family,
		fields: [],
		kind,
		label: kind,
		output_example: { ok: true },
		version: 1,
		...overrides,
	}
}

const LOOKUP: EditorLookup = {
	descriptors: new Map([
		[
			'flow.condition',
			descriptor('flow.condition', 'flow', {
				branches: ['Then', 'Else'],
				fields: [
					{
						expression: true,
						kind: 'Text',
						label: 'Condition',
						name: 'predicate',
						required: true,
						secret: false,
						visible_when: null,
					},
				],
			}),
		],
		[
			'http.request',
			descriptor('http.request', 'http', {
				auth: { Optional: ['bearer_token'] },
				fields: [
					{
						expression: false,
						kind: { Select: { options: [{ value: 'GET', label: 'GET' }] } },
						label: 'Méthode',
						name: 'method',
						required: true,
						secret: false,
						visible_when: null,
					},
					{
						expression: true,
						kind: 'Text',
						label: 'URL',
						name: 'url',
						required: true,
						secret: false,
						visible_when: null,
					},
				],
			}),
		],
	]),
	events: [],
}

const SOURCES: UpstreamSources = {
	descriptors: LOOKUP.descriptors,
	events: [],
	lastRun: null,
	runSteps: new Map(),
}

const AT = { x: 0, y: 0 }

function edge(source: string, target: string, handle?: string): EditorEdge {
	return {
		id: `${source}->${target}:${handle ?? ''}`,
		source,
		target,
		sourceHandle: handle,
	}
}

async function openNode(
	nodeId: string,
	nodes: EditorNode[],
	edges: EditorEdge[],
	handlers: { onUpdateConnector?: (patch: unknown) => void } = {},
) {
	const node = nodes.find((candidate) => candidate.id === nodeId)
	if (!node) throw new Error(`no node ${nodeId}`)
	await renderWithRouter(
		<TooltipProvider>
			<NodeDetailsFeature
				node={node}
				nodes={nodes}
				edges={edges}
				organizationSlug="atelier-bois"
				sources={SOURCES}
				authSchemes={[]}
				credentials={[]}
				issues={[]}
				testRun={{ running: false, start: vi.fn() }}
				onUpdateConnector={handlers.onUpdateConnector ?? vi.fn()}
				onUpdateTriggerKind={vi.fn()}
				onUpdateMeta={vi.fn()}
				onCreateCredential={vi.fn()}
				onEvaluateExpression={vi.fn()}
				onClose={vi.fn()}
				onNavigate={vi.fn()}
			/>
		</TooltipProvider>,
	)
	return screen.findByRole('dialog')
}

function chain(): { nodes: EditorNode[]; edges: EditorEdge[] } {
	return {
		nodes: [
			triggerNode({ id: 't1', kind: 'Manual' }, AT, LOOKUP),
			connectorNode(
				{ id: 'c1', kind: 'flow.condition', version: 1, config: {} },
				AT,
				LOOKUP,
			),
			connectorNode(
				{
					id: 'c2',
					kind: 'http.request',
					version: 1,
					config: { method: 'POST', url: 'https://a' },
				},
				{ x: 300, y: 0 },
				LOOKUP,
				{ label: 'Appeler le fournisseur' },
			),
		],
		edges: [edge('t1', 'c1'), edge('c1', 'c2', 'Then')],
	}
}

describe('NodeDetailsFeature — what the header says of the node', () => {
	it('shows the label of the type in the badge', async () => {
		const { nodes, edges } = chain()
		const renamed = nodes.map((node) =>
			node.id === 'c1'
				? { ...node, data: { ...node.data, label: 'Vérifier le montant' } }
				: node,
		)

		const dialog = await openNode('c1', renamed, edges)

		expect(within(dialog).getByText('Si')).toBeDefined()
	})

	it('describes a node with no description by its type and its subtitle, never its family', async () => {
		const { nodes, edges } = chain()
		const cleared = nodes.map((node) =>
			node.id === 'c1'
				? { ...node, data: { ...node.data, subtitle: '' } }
				: node,
		)

		const dialog = await openNode('c1', cleared, edges)

		const described = dialog.getAttribute('aria-describedby')
		expect(document.getElementById(described ?? '')?.textContent).toBe(
			'Si · Condition',
		)
	})
})

describe('NodeDetailsFeature — the neighbours', () => {
	it('names the branch of a next node in French, and none for a previous one', async () => {
		const { nodes, edges } = chain()

		const dialog = await openNode('c1', nodes, edges)

		expect(
			within(dialog).getByRole('button', {
				name: 'Ouvrir le nœud suivant : Appeler le fournisseur (branche vrai)',
			}),
		).toBeDefined()
		expect(
			within(dialog).getByRole('button', {
				name: 'Ouvrir le nœud précédent : Clic',
			}),
		).toBeDefined()
	})

	it('names the branch the data comes from in the variables', async () => {
		const { nodes, edges } = chain()

		const dialog = await openNode('c2', nodes, edges)

		expect(
			within(dialog).getByText(/entrée directe, branche vrai/),
		).toBeDefined()
	})
})

describe('NodeDetailsFeature — the parameters', () => {
	it('leaves the parameters of the manual trigger empty', async () => {
		const { nodes, edges } = chain()

		const dialog = await openNode('t1', nodes, edges)

		const parameters = within(dialog).getByRole('region', {
			name: 'Paramètres',
		})
		expect(parameters.textContent).toBe('Paramètres')
		expect(within(parameters).queryByRole('button')).toBeNull()
		expect(within(parameters).queryByRole('textbox')).toBeNull()
	})

	it('seeds the defaults of a node that has none yet', async () => {
		const onUpdateConnector = vi.fn()
		const nodes = [
			triggerNode({ id: 't1', kind: 'Manual' }, AT, LOOKUP),
			connectorNode(
				{ id: 'c1', kind: 'http.request', version: 1, config: {} },
				AT,
				LOOKUP,
			),
		]

		await openNode('c1', nodes, [edge('t1', 'c1')], { onUpdateConnector })

		expect(onUpdateConnector).toHaveBeenCalledWith({
			config: { method: 'GET' },
		})
	})

	it('tells what the spec of a collection declares under the identification', async () => {
		const collection = readCollections().find(
			(candidate) => candidate.operations.length > 0,
		)
		if (!collection) throw new Error('no preset collection')
		const operation = collection.operations[0]
		const base = chain()
		const nodes = base.nodes.map((node) =>
			node.id === 'c2'
				? {
						...node,
						data: {
							...node.data,
							meta: {
								operation: {
									collectionId: collection.id,
									operationId: operation.id,
								},
							},
						},
					}
				: node,
		)

		const dialog = await openNode('c2', nodes, base.edges)

		const declared =
			collection.security.length > 0
				? `La spec déclare : ${collection.security.join(', ')}.`
				: 'La spec ne déclare pas d’authentification.'
		expect(within(dialog).getByText(new RegExp(declared))).toBeDefined()
	})
})
