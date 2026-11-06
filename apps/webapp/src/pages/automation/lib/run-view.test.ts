import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	buildRunView,
	graphNodeNames,
	realEdgeId,
	skipReason,
} from '#/pages/automation/lib/run-view'

const GRAPH: Schemas.GraphDto = {
	triggers: [{ id: 't1', kind: { Events: ['quote.accepted'] } }],
	connectors: [
		{ id: 'c1', kind: 'odoo.invoice', version: 1, config: {} },
		{ id: 'c2', kind: 'mestier.mail', version: 1, config: {} },
		{ id: 'c3', kind: 'mestier.mail', version: 1, config: {} },
	],
	edges: [
		{ from: 't1', to: 'c1' },
		{ from: 'c1', to: 'c2' },
		{ from: 'c2', to: 'c3' },
	],
}

const DESCRIPTORS = new Map([
	[
		'odoo.invoice',
		{
			kind: 'odoo.invoice',
			label: 'Facture Odoo',
			family: 'odoo',
			branches: [],
		},
	],
] as unknown as [string, Schemas.ConnectorDescriptorResponse][])

function step(
	overrides: Partial<Schemas.RunStepResponse>,
): Schemas.RunStepResponse {
	return {
		id: 's1',
		connector_id: 'c1',
		status: 'succeeded',
		attempts: 1,
		created_at: '2026-08-01T10:00:00Z',
		started_at: '2026-08-01T10:00:00Z',
		finished_at: '2026-08-01T10:00:02Z',
		iteration_path: '',
		...overrides,
	}
}

function run(
	status: string,
	steps: Schemas.RunStepResponse[],
): Schemas.RunDetailResponse {
	return {
		id: 'r1',
		number: 12,
		organization_id: 'org-1',
		workflow_id: 'w1',
		workflow_version_id: 'v1',
		workflow_version_created_at: '2026-07-31T18:20:00Z',
		steps_done: steps.filter((item) => item.status === 'succeeded').length,
		steps_total: GRAPH.connectors.length,
		status,
		created_at: '2026-08-01T10:00:00Z',
		started_at: '2026-08-01T10:00:00Z',
		finished_at: '2026-08-01T10:00:05Z',
		graph: GRAPH,
		steps,
	}
}

describe('buildRunView', () => {
	it('stops on the failed step and leaves the rest unreached', () => {
		const view = buildRunView(
			run('failed', [
				step({ id: 's1', connector_id: 'c1' }),
				step({
					id: 's2',
					connector_id: 'c2',
					status: 'failed',
					error: 'SMTP expiré',
					started_at: '2026-08-01T10:00:02Z',
					finished_at: '2026-08-01T10:00:03Z',
				}),
			]),
			DESCRIPTORS,
		)

		expect(view.steps.map((s) => [s.nodeId, s.status])).toEqual([
			['t1', 'success'],
			['c1', 'success'],
			['c2', 'error'],
		])
		expect(view.failed?.detail).toBe('SMTP expiré')
		expect(view.pending.map((s) => [s.nodeId, s.status, s.detail])).toEqual([
			['c3', 'skipped', 'Arrêt sur « c2 »'],
		])
		expect(view.status).toBe('error')
		expect(view.durationMs).toBe(5000)
	})

	it('marks unreached connectors as waiting while the run is still going', () => {
		const view = buildRunView(
			{
				...run('running', [
					step({ connector_id: 'c1', status: 'in_flight', finished_at: null }),
				]),
				finished_at: null,
			},
			DESCRIPTORS,
		)

		expect(view.pending.map((s) => s.status)).toEqual(['waiting', 'waiting'])
		expect(view.pending[0]?.detail).toBe('Pas encore atteint')
		expect(view.nodes.find((n) => n.id === 'c1')?.note).toBe('En cours…')
	})

	it('marks an edge as traversed when both ends were reached', () => {
		const view = buildRunView(
			run('failed', [
				step({ id: 's1', connector_id: 'c1' }),
				step({ id: 's2', connector_id: 'c2', status: 'failed' }),
			]),
			DESCRIPTORS,
		)

		expect([...view.traversedEdges]).toEqual([
			realEdgeId({ from: 't1', to: 'c1' }),
			realEdgeId({ from: 'c1', to: 'c2' }),
		])
	})

	it('uses the descriptor label and numbers loop iterations', () => {
		const view = buildRunView(
			run('succeeded', [
				step({ id: 's1', connector_id: 'c1', iteration_path: 'loop[1]' }),
			]),
			DESCRIPTORS,
		)

		expect(view.steps.find((s) => s.nodeId === 'c1')?.label).toBe(
			'Facture Odoo · itération 2',
		)
		expect(view.nodes.find((n) => n.kind === 'trigger')?.subtitle).toBe(
			'quote.accepted',
		)
	})

	it('keeps an empty view when the version graph is gone', () => {
		const view = buildRunView(
			{ ...run('succeeded', []), graph: null },
			DESCRIPTORS,
		)

		expect(view.nodes).toEqual([])
		expect(view.steps).toEqual([])
	})

	it('counts the items carried by each traversed edge', () => {
		const view = buildRunView(
			{
				...run('succeeded', [
					step({ id: 's1', connector_id: 'c1', output: [{ a: 1 }, { a: 2 }] }),
					step({ id: 's2', connector_id: 'c2', output: { sent: true } }),
				]),
				trigger_payload: { number: 'DV-1' },
			},
			DESCRIPTORS,
		)

		expect(view.edgeCounts.get(realEdgeId({ from: 't1', to: 'c1' }))).toBe(1)
		expect(view.edgeCounts.get(realEdgeId({ from: 'c1', to: 'c2' }))).toBe(2)
		expect(view.edgeCounts.has(realEdgeId({ from: 'c2', to: 'c3' }))).toBe(
			false,
		)
	})

	it('groups looped steps by connector and iteration, in execution order', () => {
		const view = buildRunView(
			run('succeeded', [
				step({ id: 's0', connector_id: 'c1' }),
				step({
					id: 's1',
					connector_id: 'c2',
					iteration_path: 'c1[0]',
					started_at: '2026-08-01T10:00:01Z',
				}),
				step({
					id: 's2',
					connector_id: 'c2',
					iteration_path: 'c1[1]',
					started_at: '2026-08-01T10:00:02Z',
				}),
			]),
			DESCRIPTORS,
		)

		const [, first, loop] = view.stepGroups
		expect(first?.kind).toBe('step')
		expect(loop?.kind).toBe('loop')
		if (loop?.kind !== 'loop') return
		expect(loop.label).toBe('Facture Odoo')
		expect(loop.iterations.map((iteration) => iteration.index)).toEqual([0, 1])
		expect(
			loop.iterations[1]?.children.map((child) =>
				child.kind === 'step' ? child.step.plainLabel : null,
			),
		).toEqual(['c2'])
	})

	it('numbers nodes by the order of their first step', () => {
		const view = buildRunView(
			run('succeeded', [
				step({ id: 's1', connector_id: 'c1' }),
				step({
					id: 's2',
					connector_id: 'c2',
					started_at: '2026-08-01T10:00:02Z',
				}),
			]),
			DESCRIPTORS,
		)

		expect(view.nodes.find((node) => node.id === 't1')?.order).toBe(1)
		expect(view.nodes.find((node) => node.id === 'c1')?.order).toBe(2)
		expect(view.nodes.find((node) => node.id === 'c2')?.order).toBe(3)
		expect(view.nodes.find((node) => node.id === 'c3')?.order).toBeNull()
	})
})

const RUN_START = '2026-08-01T10:00:00Z'
const NOW = new Date('2026-08-01T10:00:03.200Z').getTime()

function liveRun(
	steps: Schemas.RunStepResponse[],
	status = 'running',
): Schemas.RunDetailResponse {
	return { ...run(status, steps), started_at: RUN_START, finished_at: null }
}

describe('buildRunView — the statuses of a step', () => {
	it('shows an in_flight step as running, with how long it has been running', () => {
		const view = buildRunView(
			liveRun([
				step({
					id: 's1',
					connector_id: 'c1',
					status: 'in_flight',
					started_at: '2026-08-01T10:00:01Z',
					finished_at: null,
				}),
			]),
			DESCRIPTORS,
			NOW,
		)

		expect(view.nodes.find((n) => n.id === 'c1')).toMatchObject({
			status: 'running',
			note: 'En cours…',
			order: 2,
		})
		expect(view.steps.find((s) => s.nodeId === 'c1')).toMatchObject({
			status: 'running',
			detail: 'En cours · 2,2 s',
		})
		expect(view.status).toBe('running')
		expect(view.durationMs).toBe(3200)
	})

	it('only says the step is running when it has no start to count from', () => {
		const view = buildRunView(
			liveRun([
				step({
					connector_id: 'c1',
					status: 'in_flight',
					started_at: null,
					finished_at: null,
				}),
			]),
			DESCRIPTORS,
			NOW,
		)

		expect(view.steps.find((s) => s.nodeId === 'c1')?.detail).toBe('En cours')
	})

	it('treats a dead step as a failure with its message', () => {
		const view = buildRunView(
			run('failed', [
				step({ id: 's1', connector_id: 'c1' }),
				step({
					id: 's2',
					connector_id: 'c2',
					status: 'dead',
					error: 'SMTP expiré',
					started_at: '2026-08-01T10:00:02Z',
					finished_at: '2026-08-01T10:00:03Z',
				}),
			]),
			DESCRIPTORS,
		)

		expect(view.nodes.find((n) => n.id === 'c2')).toMatchObject({
			status: 'error',
			note: 'Échec après 1 s',
		})
		expect(view.failed).toMatchObject({
			nodeId: 'c2',
			status: 'error',
			detail: 'SMTP expiré',
		})
	})

	it('treats a failed step as a failure too', () => {
		const view = buildRunView(
			run('failed', [
				step({ connector_id: 'c1', status: 'failed', error: 'timeout' }),
			]),
			DESCRIPTORS,
		)

		expect(view.failed).toMatchObject({ nodeId: 'c1', detail: 'timeout' })
	})

	it('keeps a skipped step out of the numbered steps and lists it as not called', () => {
		const view = buildRunView(
			run('succeeded', [
				step({ id: 's1', connector_id: 'c1' }),
				step({
					id: 's2',
					connector_id: 'c2',
					status: 'skipped',
					attempts: 0,
					started_at: '2026-08-01T10:00:02Z',
					finished_at: '2026-08-01T10:00:02Z',
				}),
			]),
			DESCRIPTORS,
		)

		expect(view.nodes.find((n) => n.id === 'c2')).toMatchObject({
			status: 'skipped',
			note: 'Non appelé',
			order: null,
		})
		expect(view.steps.map((s) => s.nodeId)).toEqual(['t1', 'c1'])
		expect(
			view.stepGroups.map((group) =>
				group.kind === 'step' ? group.step.nodeId : null,
			),
		).toEqual(['t1', 'c1'])
		expect(view.pending.map((s) => [s.nodeId, s.status, s.detail])).toEqual([
			['c2', 'skipped', 'Aucun élément reçu'],
			['c3', 'skipped', 'Aucun élément reçu'],
		])
	})

	it('does not draw the edges that lead to a skipped step as followed', () => {
		const view = buildRunView(
			run('succeeded', [
				step({ id: 's1', connector_id: 'c1' }),
				step({ id: 's2', connector_id: 'c2', status: 'skipped' }),
			]),
			DESCRIPTORS,
		)

		expect([...view.traversedEdges]).toEqual([
			realEdgeId({ from: 't1', to: 'c1' }),
		])
	})

	it('lists a pending step as not reached yet while the run goes on', () => {
		const view = buildRunView(
			liveRun([
				step({ id: 's1', connector_id: 'c1' }),
				step({
					id: 's2',
					connector_id: 'c2',
					status: 'pending',
					attempts: 0,
					started_at: null,
					finished_at: null,
				}),
			]),
			DESCRIPTORS,
			NOW,
		)

		expect(view.nodes.find((n) => n.id === 'c2')).toMatchObject({
			status: 'waiting',
			note: 'En attente',
			order: null,
		})
		expect(view.steps.map((s) => s.nodeId)).toEqual(['t1', 'c1'])
		expect(view.pending.map((s) => [s.nodeId, s.status, s.detail])).toEqual([
			['c2', 'waiting', 'Pas encore atteint'],
			['c3', 'waiting', 'Pas encore atteint'],
		])
		expect([...view.traversedEdges]).toEqual([
			realEdgeId({ from: 't1', to: 'c1' }),
		])
	})

	it('no longer waits for a pending step once the run has settled', () => {
		const view = buildRunView(
			run('cancelled', [
				step({ id: 's1', connector_id: 'c1' }),
				step({ id: 's2', connector_id: 'c2', status: 'pending' }),
			]),
			DESCRIPTORS,
		)

		expect(view.status).toBe('cancelled')
		expect(view.nodes.find((n) => n.id === 'c2')?.status).toBe('skipped')
	})

	it('lets a step that ran decide the node when another iteration was skipped', () => {
		const view = buildRunView(
			run('succeeded', [
				step({
					id: 's1',
					connector_id: 'c2',
					iteration_path: 'c1[0]',
					status: 'skipped',
				}),
				step({
					id: 's2',
					connector_id: 'c2',
					iteration_path: 'c1[1]',
					status: 'succeeded',
					started_at: '2026-08-01T10:00:01Z',
				}),
			]),
			DESCRIPTORS,
		)

		expect(view.nodes.find((n) => n.id === 'c2')?.status).toBe('success')
		expect(view.pending.map((s) => s.nodeId)).not.toContain('c2')
		expect(view.steps.map((s) => s.stepId)).toEqual(['trigger:t1', 's2'])
	})

	it('does not list a node twice when one of its iterations is still pending', () => {
		const view = buildRunView(
			liveRun([
				step({
					id: 's1',
					connector_id: 'c2',
					iteration_path: 'c1[0]',
					started_at: '2026-08-01T10:00:01Z',
				}),
				step({
					id: 's2',
					connector_id: 'c2',
					iteration_path: 'c1[1]',
					status: 'pending',
					attempts: 0,
					started_at: null,
					finished_at: null,
				}),
			]),
			DESCRIPTORS,
			NOW,
		)

		expect(view.steps.map((s) => s.stepId)).toEqual(['trigger:t1', 's1'])
		expect(view.pending.map((s) => s.nodeId)).not.toContain('c2')
		expect(view.nodes.find((n) => n.id === 'c2')?.order).toBe(2)
	})

	it('names the node the run stopped on for everything that came after it', () => {
		const view = buildRunView(
			run('failed', [
				step({
					id: 's1',
					connector_id: 'c1',
					status: 'dead',
					error: 'boom',
				}),
			]),
			DESCRIPTORS,
		)

		expect(view.pending.map((s) => [s.nodeId, s.detail])).toEqual([
			['c2', 'Arrêt sur « Facture Odoo »'],
			['c3', 'Arrêt sur « Facture Odoo »'],
		])
	})

	it('says no item came when the node is not downstream of the failure', () => {
		const view = buildRunView(
			{
				...run('failed', [
					step({ id: 's1', connector_id: 'c1', status: 'dead', error: 'boom' }),
				]),
				graph: {
					...GRAPH,
					edges: [
						{ from: 't1', to: 'c1' },
						{ from: 't1', to: 'c2' },
						{ from: 'c2', to: 'c3' },
					],
				},
			},
			DESCRIPTORS,
		)

		expect(view.pending.map((s) => [s.nodeId, s.detail])).toEqual([
			['c2', 'Aucun élément reçu'],
			['c3', 'Aucun élément reçu'],
		])
	})
})

describe('buildRunView — the live duration', () => {
	it('counts the run from its start while it is running', () => {
		const view = buildRunView(liveRun([]), DESCRIPTORS, NOW)

		expect(view.durationMs).toBe(3200)
	})

	it('has no duration for a run that has not started', () => {
		const view = buildRunView(
			{ ...liveRun([], 'pending'), started_at: null },
			DESCRIPTORS,
			NOW,
		)

		expect(view.durationMs).toBeNull()
	})
})

describe('buildRunView — the presentation of a connector', () => {
	const KINDS_GRAPH: Schemas.GraphDto = {
		triggers: [],
		connectors: [
			{ id: 'c1', kind: 'flow.condition', version: 1, config: {} },
			{ id: 'c2', kind: 'odoo.invoice', version: 1, config: {} },
			{ id: 'c3', kind: 'mystery.thing', version: 1, config: {} },
		],
		edges: [],
	}

	it('takes the subtitle of the kind, as the editor does', () => {
		const view = buildRunView(
			{ ...run('succeeded', []), graph: KINDS_GRAPH },
			DESCRIPTORS,
		)

		expect(view.nodes.find((n) => n.id === 'c1')?.subtitle).toBe('Condition')
	})

	it('says “Connecteur” for a kind without a copy, whatever its family', () => {
		const view = buildRunView(
			{ ...run('succeeded', []), graph: KINDS_GRAPH },
			DESCRIPTORS,
		)

		expect(view.nodes.find((n) => n.id === 'c2')?.subtitle).toBe('Connecteur')
		expect(view.nodes.find((n) => n.id === 'c3')?.subtitle).toBe('Connecteur')
	})

	it('names a connector after its kind first, as the editor does', () => {
		const view = buildRunView(
			{ ...run('succeeded', []), graph: KINDS_GRAPH },
			new Map([
				[
					'flow.condition',
					{
						kind: 'flow.condition',
						label: 'Condition du serveur',
						family: 'flow',
						branches: [],
					},
				],
			] as unknown as [string, Schemas.ConnectorDescriptorResponse][]),
		)

		expect(view.nodes.find((n) => n.id === 'c1')?.label).toBe('Si')
	})

	it('uses that name in the steps, the loops and the unreached nodes', () => {
		const view = buildRunView(
			{
				...run('failed', [
					step({ id: 's1', connector_id: 'c1', iteration_path: 'c1[0]' }),
					step({
						id: 's2',
						connector_id: 'c1',
						iteration_path: 'c1[1]',
						status: 'dead',
						error: 'boom',
					}),
				]),
				graph: {
					...KINDS_GRAPH,
					edges: [{ from: 'c1', to: 'c3' }],
				},
			},
			DESCRIPTORS,
		)

		expect(view.steps.map((s) => s.label)).toEqual([
			'Si · itération 1',
			'Si · itération 2',
		])
		expect(view.failed?.plainLabel).toBe('Si')
		const loop = view.stepGroups[0]
		expect(loop?.kind === 'loop' ? loop.label : null).toBe('Si')
		expect(view.pending.find((s) => s.nodeId === 'c3')?.detail).toBe(
			'Arrêt sur « Si »',
		)
	})

	it('keeps the label of the descriptor', () => {
		const view = buildRunView(
			{ ...run('succeeded', []), graph: KINDS_GRAPH },
			DESCRIPTORS,
		)

		expect(view.nodes.find((n) => n.id === 'c2')?.label).toBe('Facture Odoo')
		expect(view.nodes.find((n) => n.id === 'c3')?.label).toBe('c3')
	})

	it('carries the kind of the connector on its steps, so they look like their node', () => {
		const view = buildRunView(
			{
				...run('succeeded', [
					step({ id: 's1', connector_id: 'c1' }),
					step({ id: 's2', connector_id: 'ghost' }),
				]),
				graph: KINDS_GRAPH,
			},
			DESCRIPTORS,
		)

		expect(view.steps.map((s) => [s.nodeId, s.connectorKind])).toEqual([
			['c1', 'flow.condition'],
			['ghost', undefined],
		])
	})

	it('carries it on the nodes that were not reached too', () => {
		const view = buildRunView(
			{ ...run('succeeded', []), graph: KINDS_GRAPH },
			DESCRIPTORS,
		)

		expect(view.pending.map((s) => s.connectorKind)).toEqual([
			'flow.condition',
			'odoo.invoice',
			'mystery.thing',
		])
	})
})

describe('skipReason', () => {
	it('blames the failed node for everything downstream of it', () => {
		expect(skipReason('c3', GRAPH, 'c2')).toEqual({
			type: 'after-error',
			failedNodeId: 'c2',
		})
		expect(skipReason('c1', GRAPH, 'c2')).toEqual({ type: 'no-items' })
		expect(skipReason('c3', GRAPH, null)).toEqual({ type: 'no-items' })
	})
})

describe('buildRunView — the names saved in the graph', () => {
	const EVENTS = [
		{ name: 'quote.accepted', label: 'Devis accepté' },
	] as unknown as Schemas.EventDescriptorResponse[]
	const NAMED: Schemas.GraphDto = {
		triggers: [
			{
				id: 't1',
				kind: { Events: ['quote.accepted'] },
				name: 'Quand le client accepte',
				description: 'Démarre la relance',
			},
		],
		connectors: [
			{
				id: 'c1',
				kind: 'flow.condition',
				version: 1,
				config: {},
				name: 'Montant élevé ?',
				description: 'Plus de 5 000 €',
			},
			{ id: 'c2', kind: 'flow.condition', version: 1, config: {} },
		],
		edges: [
			{ from: 't1', to: 'c1' },
			{ from: 'c1', to: 'c2', branch: 'Then' },
		],
	}

	it('shows the name and the description of every node on the canvas', () => {
		const view = buildRunView(
			{ ...run('succeeded', []), graph: NAMED },
			DESCRIPTORS,
			undefined,
			EVENTS,
		)

		expect(
			view.nodes.map((node) => [node.id, node.label, node.subtitle]),
		).toEqual([
			['t1', 'Quand le client accepte', 'Démarre la relance'],
			['c1', 'Montant élevé ?', 'Plus de 5 000 €'],
			['c2', 'Si', 'Condition'],
		])
	})

	it('carries the names into the steps, the loops, the failure and the unreached nodes', () => {
		const view = buildRunView(
			{
				...run('failed', [
					step({
						id: 's1',
						connector_id: 'c1',
						status: 'dead',
						error: 'boom',
						iteration_path: 'c1[0]',
					}),
				]),
				graph: NAMED,
			},
			DESCRIPTORS,
			undefined,
			EVENTS,
		)

		expect(view.steps.map((s) => s.label)).toEqual([
			'Quand le client accepte',
			'Montant élevé ? · itération 1',
		])
		expect(view.failed?.plainLabel).toBe('Montant élevé ?')
		expect(view.pending.map((s) => [s.nodeId, s.label, s.detail])).toEqual([
			['c2', 'Si', 'Arrêt sur « Montant élevé ? »'],
		])
		const loop = view.stepGroups[1]
		expect(loop?.kind === 'loop' ? loop.label : null).toBe('Montant élevé ?')
	})

	it('lists the name of every node of a graph, the default one when it has none', () => {
		expect(graphNodeNames(NAMED, DESCRIPTORS, EVENTS)).toEqual(
			new Map([
				['t1', 'Quand le client accepte'],
				['c1', 'Montant élevé ?'],
				['c2', 'Si'],
			]),
		)
	})

	it('names a webhook and a scheduler after their kind, or after the name saved in the graph', () => {
		const graph: Schemas.GraphDto = {
			...NAMED,
			triggers: [
				{
					id: 't1',
					kind: {
						Webhook: {
							method: 'POST',
							path: 'paiement-recu',
							credential_id: null,
							contract: [],
							on_invalid: 'Reject',
						},
					},
				},
				{
					id: 't2',
					kind: { Schedule: { frequency: 'Daily', time: '09:00' } },
				},
				{
					id: 't3',
					name: 'Paiement Stripe',
					kind: {
						Webhook: { method: 'GET', path: 'stripe', credential_id: null },
					},
				},
			],
		}

		expect(graphNodeNames(graph, DESCRIPTORS, EVENTS)).toEqual(
			new Map([
				['t1', 'Webhook'],
				['t2', 'Planificateur'],
				['t3', 'Paiement Stripe'],
				['c1', 'Montant élevé ?'],
				['c2', 'Si'],
			]),
		)
	})

	it('shows the subtitle of a webhook and the rule of a scheduler on the canvas', () => {
		const view = buildRunView(
			{
				...run('succeeded', []),
				graph: {
					...NAMED,
					triggers: [
						{
							id: 't1',
							kind: { Webhook: { method: 'POST', path: 'x' } },
						},
						{
							id: 't2',
							kind: {
								Schedule: {
									frequency: 'Weekly',
									time: '14:30',
									weekday: 'Friday',
								},
							},
						},
					],
				},
			},
			DESCRIPTORS,
			undefined,
			EVENTS,
		)

		expect(view.nodes.slice(0, 2)).toMatchObject([
			{ label: 'Webhook', subtitle: 'Déclencheur HTTP' },
			{
				label: 'Planificateur',
				subtitle: 'toutes les semaines le vendredi à 14:30',
			},
		])
	})

	it('names a trigger that has no name of its own after its event', () => {
		const view = buildRunView(
			{
				...run('succeeded', []),
				graph: {
					...NAMED,
					triggers: [{ id: 't1', kind: { Events: ['quote.accepted'] } }],
				},
			},
			DESCRIPTORS,
			undefined,
			EVENTS,
		)

		expect(view.nodes[0]).toMatchObject({
			label: 'Devis accepté',
			subtitle: 'quote.accepted',
		})
	})
})

describe('buildRunView — the trigger of the run', () => {
	const TWO_TRIGGERS: Schemas.GraphDto = {
		triggers: [
			{ id: 't1', kind: { Events: ['quote.accepted'] }, name: 'Devis accepté' },
			{ id: 't2', kind: 'Manual', name: 'Lancement à la main' },
		],
		connectors: [{ id: 'c1', kind: 'mestier.mail', version: 1, config: {} }],
		edges: [
			{ from: 't1', to: 'c1' },
			{ from: 't2', to: 'c1' },
		],
	}

	it('numbers the trigger first, under its own name, before the connectors', () => {
		const view = buildRunView(
			{
				...run('succeeded', [step({ id: 's1', connector_id: 'c1' })]),
				trigger_id: 't1',
				graph: TWO_TRIGGERS,
			},
			DESCRIPTORS,
		)

		expect(
			view.steps.map((s) => [s.nodeId, s.label, s.status, s.detail]),
		).toEqual([
			['t1', 'Devis accepté', 'success', 'Démarré'],
			['c1', 'c1', 'success', '2 s'],
		])
		expect(view.nodes.find((n) => n.id === 't1')).toMatchObject({
			status: 'success',
			order: 1,
		})
		expect(view.nodes.find((n) => n.id === 'c1')?.order).toBe(2)
		const [first] = view.stepGroups
		expect(first?.kind === 'step' ? first.step.nodeId : null).toBe('t1')
	})

	it('only counts the trigger that started the run, the other one was not called', () => {
		const view = buildRunView(
			{
				...run('succeeded', [step({ id: 's1', connector_id: 'c1' })]),
				trigger_id: 't1',
				graph: TWO_TRIGGERS,
			},
			DESCRIPTORS,
		)

		expect(view.nodes.find((n) => n.id === 't2')).toMatchObject({
			status: 'skipped',
			note: 'Non appelé',
			order: null,
		})
		expect([...view.traversedEdges]).toEqual([
			realEdgeId({ from: 't1', to: 'c1' }),
		])
		expect(view.pending.map((s) => s.nodeId)).toEqual([])
	})

	it('takes the manual trigger for a run that no trigger started', () => {
		const view = buildRunView(
			{
				...run('succeeded', [step({ id: 's1', connector_id: 'c1' })]),
				graph: TWO_TRIGGERS,
			},
			DESCRIPTORS,
		)

		expect(view.nodes.find((n) => n.id === 't2')?.status).toBe('success')
		expect(view.nodes.find((n) => n.id === 't1')?.status).toBe('skipped')
		expect(view.steps[0]?.nodeId).toBe('t2')
	})

	it('keeps the trigger out of the steps until the run has started', () => {
		const view = buildRunView(
			{
				...run('pending', []),
				started_at: null,
				finished_at: null,
				trigger_id: 't1',
				graph: TWO_TRIGGERS,
			},
			DESCRIPTORS,
		)

		expect(view.steps).toEqual([])
		expect(view.nodes.find((n) => n.id === 't1')).toMatchObject({
			status: 'waiting',
			note: 'En attente',
			order: null,
		})
	})
})
