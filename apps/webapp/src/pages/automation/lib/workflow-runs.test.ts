import { describe, expect, it } from 'vitest'
import type { Schemas } from '#/api/api.client'
import type { Run } from '#/hooks/use-automation'
import {
	aggregateConnectorStatuses,
	canReplay,
	connectorOutputsFromSteps,
	elapsedMsSince,
	groupRunSteps,
	isTerminalRunStatus,
	latestRunByWorkflow,
	latestRunId,
	representativeStepByConnector,
	runDurationMs,
	runElapsedMs,
	runTriggerLabel,
	triggerEventLabel,
} from '#/pages/automation/lib/workflow-runs'

function run(overrides: Partial<Run> = {}): Run {
	return {
		id: 'run-1',
		organization_id: 'org-1',
		workflow_id: 'workflow-1',
		workflow_version_id: 'v1',
		status: 'succeeded',
		created_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}

describe('latestRunByWorkflow', () => {
	it('returns nothing for a workflow with no run', () => {
		expect(latestRunByWorkflow([]).size).toBe(0)
	})

	it('picks the only run for a workflow that ran once', () => {
		const result = latestRunByWorkflow([run({ status: 'failed' })])

		expect(result.get('workflow-1')).toEqual({
			status: 'failed',
			at: '2026-08-01T00:00:00Z',
		})
	})

	it('picks the most recent run when a workflow ran several times, regardless of array order', () => {
		const result = latestRunByWorkflow([
			run({
				id: 'run-1',
				status: 'failed',
				created_at: '2026-08-01T00:00:00Z',
			}),
			run({
				id: 'run-2',
				status: 'succeeded',
				created_at: '2026-08-03T00:00:00Z',
			}),
			run({
				id: 'run-3',
				status: 'running',
				created_at: '2026-08-02T00:00:00Z',
			}),
		])

		expect(result.get('workflow-1')).toEqual({
			status: 'succeeded',
			at: '2026-08-03T00:00:00Z',
		})
	})

	it('keeps each workflow’s own latest run separate', () => {
		const result = latestRunByWorkflow([
			run({ id: 'run-1', workflow_id: 'workflow-1', status: 'succeeded' }),
			run({ id: 'run-2', workflow_id: 'workflow-2', status: 'failed' }),
		])

		expect(result.get('workflow-1')?.status).toBe('succeeded')
		expect(result.get('workflow-2')?.status).toBe('failed')
	})
})

describe('latestRunId', () => {
	it('is null when the workflow never ran', () => {
		expect(latestRunId([], 'workflow-1')).toBeNull()
	})

	it('names the most recent run for that workflow only', () => {
		const result = latestRunId(
			[
				run({
					id: 'run-1',
					workflow_id: 'workflow-1',
					created_at: '2026-08-01T00:00:00Z',
				}),
				run({
					id: 'run-2',
					workflow_id: 'workflow-1',
					created_at: '2026-08-03T00:00:00Z',
				}),
				run({
					id: 'run-3',
					workflow_id: 'workflow-2',
					created_at: '2026-08-04T00:00:00Z',
				}),
			],
			'workflow-1',
		)

		expect(result).toBe('run-2')
	})
})

function step(
	overrides: Partial<Schemas.RunStepResponse> = {},
): Schemas.RunStepResponse {
	return {
		id: 'step-1',
		connector_id: 'c1',
		iteration_path: '',
		attempts: 1,
		status: 'succeeded',
		created_at: '2026-08-01T00:00:00Z',
		...overrides,
	}
}

describe('connectorOutputsFromSteps', () => {
	it('maps each connector id to its output', () => {
		const outputs = connectorOutputsFromSteps([
			step({ connector_id: 'c1', output: { id: 1 } }),
			step({ connector_id: 'c2', output: { id: 2 } }),
		])

		expect(outputs).toEqual({ c1: { id: 1 }, c2: { id: 2 } })
	})

	it('ignores a step with no output yet', () => {
		const outputs = connectorOutputsFromSteps([
			step({ connector_id: 'c1', output: undefined }),
		])

		expect(outputs).toEqual({})
	})

	it('prefers the top-level iteration over a nested loop iteration', () => {
		const outputs = connectorOutputsFromSteps([
			step({
				connector_id: 'c1',
				iteration_path: 'c2[0]',
				output: { id: 'a' },
			}),
			step({ connector_id: 'c1', iteration_path: '', output: { id: 'b' } }),
		])

		expect(outputs).toEqual({ c1: { id: 'b' } })
	})
})

describe('isTerminalRunStatus', () => {
	it('is terminal for succeeded, failed and cancelled', () => {
		expect(isTerminalRunStatus('succeeded')).toBe(true)
		expect(isTerminalRunStatus('failed')).toBe(true)
		expect(isTerminalRunStatus('cancelled')).toBe(true)
	})

	it('is not terminal for pending or running', () => {
		expect(isTerminalRunStatus('pending')).toBe(false)
		expect(isTerminalRunStatus('running')).toBe(false)
	})
})

describe('runDurationMs', () => {
	it('is null when the run has not started', () => {
		expect(runDurationMs({ started_at: null, finished_at: null })).toBeNull()
	})

	it('is null when the run has not finished', () => {
		expect(
			runDurationMs({
				started_at: '2026-08-01T00:00:00.000Z',
				finished_at: null,
			}),
		).toBeNull()
	})

	it('is the elapsed milliseconds between start and finish', () => {
		expect(
			runDurationMs({
				started_at: '2026-08-01T00:00:00.000Z',
				finished_at: '2026-08-01T00:00:01.500Z',
			}),
		).toBe(1500)
	})
})

describe('elapsedMsSince', () => {
	const NOW = new Date('2026-08-01T00:00:03.200Z').getTime()

	it('is the time elapsed since the start', () => {
		expect(elapsedMsSince('2026-08-01T00:00:01.000Z', NOW)).toBe(2200)
	})

	it('is null without a start', () => {
		expect(elapsedMsSince(null, NOW)).toBeNull()
		expect(elapsedMsSince(undefined, NOW)).toBeNull()
	})

	it('never goes negative when the clock is behind the server', () => {
		expect(elapsedMsSince('2026-08-01T00:00:09.000Z', NOW)).toBe(0)
	})

	it('is null for an unreadable date', () => {
		expect(elapsedMsSince('not a date', NOW)).toBeNull()
	})
})

describe('runElapsedMs', () => {
	const NOW = new Date('2026-08-01T00:00:03.200Z').getTime()

	it('is the final duration of a finished run, whatever the clock says', () => {
		expect(
			runElapsedMs(
				{
					status: 'succeeded',
					started_at: '2026-08-01T00:00:00.000Z',
					finished_at: '2026-08-01T00:00:01.500Z',
				},
				NOW,
			),
		).toBe(1500)
	})

	it('counts from the start while the run is running', () => {
		expect(
			runElapsedMs(
				{
					status: 'running',
					started_at: '2026-08-01T00:00:01.000Z',
					finished_at: null,
				},
				NOW,
			),
		).toBe(2200)
	})

	it('has no duration for a run that has not started', () => {
		expect(
			runElapsedMs(
				{ status: 'pending', started_at: null, finished_at: null },
				NOW,
			),
		).toBeNull()
	})

	it('has no duration for a pending run waiting for its next attempt', () => {
		expect(
			runElapsedMs(
				{
					status: 'pending',
					started_at: '2026-08-01T00:00:01.000Z',
					finished_at: null,
				},
				NOW,
			),
		).toBeNull()
	})

	it('has no duration for a settled run that never recorded its end', () => {
		expect(
			runElapsedMs(
				{
					status: 'failed',
					started_at: '2026-08-01T00:00:01.000Z',
					finished_at: null,
				},
				NOW,
			),
		).toBeNull()
	})
})

describe('runTriggerLabel', () => {
	it('names a run with no trigger event as a manual test', () => {
		expect(runTriggerLabel({ trigger_event_id: null })).toBe('Test manuel')
	})

	it('names an event-triggered run after its event when the label is known', () => {
		expect(
			runTriggerLabel({ trigger_event_id: 'evt-1' }, 'Devis accepté'),
		).toBe('Devis accepté')
	})

	it('names a run carrying a trigger event as event-triggered', () => {
		expect(runTriggerLabel({ trigger_event_id: 'evt-1' })).toBe('Événement')
	})
})

describe('aggregateConnectorStatuses', () => {
	it('maps a connector with a single step to that step’s status', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', status: 'succeeded' }),
		])

		expect(statuses.get('c1')).toBe('succeeded')
	})

	it('surfaces a failure over a success across loop iterations', () => {
		const statuses = aggregateConnectorStatuses([
			step({
				connector_id: 'c1',
				iteration_path: 'c2[0]',
				status: 'succeeded',
			}),
			step({ connector_id: 'c1', iteration_path: 'c2[1]', status: 'failed' }),
			step({ connector_id: 'c1', iteration_path: 'c2[2]', status: 'pending' }),
		])

		expect(statuses.get('c1')).toBe('failed')
	})

	it('surfaces in_flight over pending when nothing has failed', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', iteration_path: 'c2[0]', status: 'pending' }),
			step({
				connector_id: 'c1',
				iteration_path: 'c2[1]',
				status: 'in_flight',
			}),
		])

		expect(statuses.get('c1')).toBe('in_flight')
	})

	it('surfaces a dead step over every other status', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', iteration_path: 'c2[0]', status: 'failed' }),
			step({ connector_id: 'c1', iteration_path: 'c2[1]', status: 'dead' }),
			step({
				connector_id: 'c1',
				iteration_path: 'c2[2]',
				status: 'succeeded',
			}),
			step({
				connector_id: 'c1',
				iteration_path: 'c2[3]',
				status: 'in_flight',
			}),
		])

		expect(statuses.get('c1')).toBe('dead')
	})

	it('lets a step that ran win over a skipped one', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', iteration_path: 'c2[0]', status: 'skipped' }),
			step({
				connector_id: 'c1',
				iteration_path: 'c2[1]',
				status: 'succeeded',
			}),
		])

		expect(statuses.get('c1')).toBe('succeeded')
	})

	it('keeps a connector that was only skipped as skipped', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', status: 'skipped' }),
		])

		expect(statuses.get('c1')).toBe('skipped')
	})

	it('has no entry for a connector with no step yet', () => {
		const statuses = aggregateConnectorStatuses([
			step({ connector_id: 'c1', status: 'succeeded' }),
		])

		expect(statuses.has('c2')).toBe(false)
	})
})

describe('groupRunSteps', () => {
	it('lists top-level steps as a flat sequence when there is no loop', () => {
		const tree = groupRunSteps([
			step({ id: 's1', connector_id: 'c1', iteration_path: '' }),
			step({ id: 's2', connector_id: 'c2', iteration_path: '' }),
		])

		expect(tree).toEqual([
			{ kind: 'step', step: step({ id: 's1', connector_id: 'c1' }) },
			{ kind: 'step', step: step({ id: 's2', connector_id: 'c2' }) },
		])
	})

	it('groups a five-item loop into one loop node with five iterations, not five flat rows', () => {
		const steps = Array.from({ length: 5 }, (_, index) =>
			step({
				id: `s${index}`,
				connector_id: 'c2',
				iteration_path: `c1[${index}]`,
			}),
		)

		const tree = groupRunSteps(steps)

		expect(tree).toHaveLength(1)
		const loop = tree[0]
		if (loop.kind !== 'loop') throw new Error('expected a loop node')
		expect(loop.connectorId).toBe('c1')
		expect(loop.iterations.map((iteration) => iteration.index)).toEqual([
			0, 1, 2, 3, 4,
		])
		expect(
			loop.iterations.every((iteration) => iteration.children.length === 1),
		).toBe(true)
	})

	it('nests a loop inside a loop, following the iteration_path', () => {
		const steps = [
			step({ id: 's1', connector_id: 'c7', iteration_path: 'c2[3].c5[0]' }),
		]

		const tree = groupRunSteps(steps)

		expect(tree).toHaveLength(1)
		const outerLoop = tree[0]
		if (outerLoop.kind !== 'loop') throw new Error('expected a loop node')
		expect(outerLoop.connectorId).toBe('c2')
		expect(outerLoop.iterations).toHaveLength(1)
		expect(outerLoop.iterations[0]?.index).toBe(3)
		expect(outerLoop.iterations[0]?.path).toBe('c2[3]')

		const innerLoop = outerLoop.iterations[0]?.children[0]
		if (!innerLoop || innerLoop.kind !== 'loop') {
			throw new Error('expected a nested loop node')
		}
		expect(innerLoop.connectorId).toBe('c5')
		expect(innerLoop.iterations).toHaveLength(1)
		expect(innerLoop.iterations[0]?.index).toBe(0)
		expect(innerLoop.iterations[0]?.path).toBe('c2[3].c5[0]')
		expect(innerLoop.iterations[0]?.children).toEqual([
			{
				kind: 'step',
				step: step({
					id: 's1',
					connector_id: 'c7',
					iteration_path: 'c2[3].c5[0]',
				}),
			},
		])
	})

	it('merges every iteration of the same loop into one group even when steps outside it fall in between', () => {
		const tree = groupRunSteps([
			step({ id: 's1', connector_id: 'c1', iteration_path: '' }),
			step({ id: 's2', connector_id: 'c3', iteration_path: 'c2[0]' }),
			step({ id: 's3', connector_id: 'c4', iteration_path: '' }),
			step({ id: 's4', connector_id: 'c3', iteration_path: 'c2[1]' }),
		])

		expect(tree.map((node) => node.kind)).toEqual(['step', 'loop', 'step'])
		const loop = tree[1]
		if (loop.kind !== 'loop') throw new Error('expected a loop node')
		expect(loop.iterations.map((iteration) => iteration.index)).toEqual([0, 1])
	})
})

describe('canReplay', () => {
	it('allows replay once a run has settled', () => {
		expect(canReplay('succeeded')).toBe(true)
		expect(canReplay('failed')).toBe(true)
		expect(canReplay('cancelled')).toBe(true)
	})

	it('refuses replay while a run is still pending or running', () => {
		expect(canReplay('pending')).toBe(false)
		expect(canReplay('running')).toBe(false)
	})
})

describe('representativeStepByConnector', () => {
	function step(
		connectorId: string,
		status: string,
		overrides: Partial<Schemas.RunStepResponse> = {},
	): Schemas.RunStepResponse {
		return {
			attempts: 1,
			connector_id: connectorId,
			created_at: '2026-09-17T10:00:00Z',
			id: `${connectorId}-${status}`,
			iteration_path: '',
			status,
			...overrides,
		}
	}

	it('keeps the step whose status the node border shows', () => {
		const steps = [
			step('c1', 'succeeded', { output: { ok: true } }),
			step('c1', 'failed', { error: 'connection refused' }),
		]

		expect(representativeStepByConnector(steps).get('c1')?.error).toBe(
			'connection refused',
		)
	})

	it('agrees with the status the border is painted from', () => {
		const steps = [
			step('c1', 'succeeded'),
			step('c1', 'failed', { error: 'boom' }),
			step('c2', 'in_flight'),
			step('c3', 'dead', { error: 'définitif' }),
			step('c3', 'skipped'),
		]
		const statuses = aggregateConnectorStatuses(steps)
		const representative = representativeStepByConnector(steps)

		for (const [id, status] of statuses) {
			expect(representative.get(id)?.status).toBe(status)
		}
	})

	it('has nothing to say about a connector with no step', () => {
		expect(representativeStepByConnector([]).get('c1')).toBeUndefined()
	})
})

describe('triggerEventLabel', () => {
	const labels = new Map([['quote.accepted', 'Devis accepté']])

	it('resolves the label of a single-event trigger', () => {
		expect(
			triggerEventLabel(
				[{ id: 't1', kind: { Events: ['quote.accepted'] } }],
				labels,
			),
		).toBe('Devis accepté')
	})

	it('falls back to the raw event name when it has no label', () => {
		expect(
			triggerEventLabel(
				[{ id: 't1', kind: { Events: ['task.done'] } }],
				labels,
			),
		).toBe('task.done')
	})

	it('has no label for a manual or multi-event trigger', () => {
		expect(triggerEventLabel([{ id: 't1', kind: 'Manual' }], labels)).toBeNull()
		expect(
			triggerEventLabel([{ id: 't1', kind: { Events: ['a', 'b'] } }], labels),
		).toBeNull()
	})
})
