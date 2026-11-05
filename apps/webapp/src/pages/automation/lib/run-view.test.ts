import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import {
	buildRunView,
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
		organization_id: 'org-1',
		workflow_id: 'w1',
		workflow_version_id: 'v1',
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
			['c1', 'success'],
			['c2', 'error'],
		])
		expect(view.failed?.detail).toBe('SMTP expiré')
		expect(view.pending.map((s) => [s.nodeId, s.status, s.detail])).toEqual([
			['c3', 'skipped', 'Non appelé'],
		])
		expect(view.status).toBe('error')
		expect(view.durationMs).toBe(5000)
	})

	it('marks unreached connectors as waiting while the run is still going', () => {
		const view = buildRunView(
			{
				...run('running', [step({ connector_id: 'c1', status: 'running' })]),
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

		expect(view.steps[0]?.label).toBe('Facture Odoo · itération 2')
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

		const [first, loop] = view.stepGroups
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

		expect(view.nodes.find((node) => node.id === 'c2')?.order).toBe(2)
		expect(view.nodes.find((node) => node.id === 'c3')?.order).toBeNull()
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
